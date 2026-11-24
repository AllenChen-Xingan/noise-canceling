"""gpu_mix（torch 批量版）对拍 mix.render（CPU 参考版）：同种子同输入，逐样本差 ≤ 1e-4；有声区间逐个相同。
有 CUDA 就在 GPU 上跑（云上），没有就用 torch 的 CPU 后端（本机）——两者走同一份代码。夹具全是现造的合成波形。"""
import copy
import inspect
import json
import sys
import unittest
from pathlib import Path

import numpy as np

try:
    import torch
except ImportError:  # 本机没装 torch 的环境：整组跳过并写明（云上与 .venv 里都装了）
    torch = None

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import mix  # noqa: E402

SR = 16000
N = 4 * SR
TOL = 1e-4
RECIPE = json.loads((HERE / "recipe.json").read_text())


def knobs(**over):
    k = copy.deepcopy(RECIPE["knobs"])
    for name, v in over.items():
        k[name].update(v)
    return k


def speech(n, seed):
    rng = np.random.default_rng(seed)
    t = np.arange(n) / SR
    env = (np.sin(2 * np.pi * (1.5 + seed % 3) * t) > 0.2).astype(float)
    return 0.1 * env * np.sin(2 * np.pi * (180 + 20 * seed) * t) + 0.002 * rng.standard_normal(n)


def rir(seed, n=SR // 2):
    rng = np.random.default_rng(100 + seed)
    h = rng.standard_normal(n) * np.exp(-6.9 * np.arange(n) / ((0.2 + 0.1 * (seed % 5)) * SR)) * 0.05
    d = 40 + 7 * seed
    h[:d] = 0
    h[d] = 1.0
    return h


def plans(K, seeds, with_rir=True):
    out = []
    for s in seeds:
        rng = np.random.default_rng([7, s])
        ns = [(np.random.default_rng(200 + s + j).standard_normal(int(SR * (2 + j))) * 0.05, j == 2, f"n{j}") for j in range(3)]
        sp_len = int(SR * (2.5 + (s % 4)))  # 有的比 4 s 短（补零），有的比 4 s 长（随机截）
        out.append(mix.plan(rng, speech(sp_len, s), ns, rir(s) if with_rir else None, K, N))
    return out


@unittest.skipIf(torch is None, "没装 torch：用仓库 .venv 跑（README 数据一节有安装命令）")
class Parity(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        import gpu_mix

        cls.g = gpu_mix
        cls.dev = "cuda" if torch.cuda.is_available() else "cpu"

    def check(self, ps, dtype=None, ivs=True):
        kw = {"dtype": dtype} if dtype is not None else {}
        out = self.g.render_batch(self.g.collate_plans(ps), device=self.dev, **kw)
        for i, p in enumerate(ps):
            noisy, clean, meta = mix.render(p, N)
            dn = float(np.max(np.abs(out["noisy"][i].cpu().numpy() - noisy)))
            dc = float(np.max(np.abs(out["clean"][i].cpu().numpy() - clean)))
            self.assertLessEqual(dn, TOL, f"样本 {i} noisy 差 {dn}")
            self.assertLessEqual(dc, TOL, f"样本 {i} clean 差 {dc}")
            if ivs:
                self.assertEqual(out["owner_ivs"][i], meta["owner_ivs"], f"样本 {i} 有声区间不同")
            self.assertEqual(out["clip_thr"][i] is None, meta["clip_thr"] is None)
            if meta["clip_thr"] is not None:
                self.assertAlmostEqual(out["clip_thr"][i], meta["clip_thr"], places=4)

    def test_default_recipe(self):
        self.check(plans(knobs(), range(12)))

    def test_every_knob_forced_on(self):
        K = knobs(clip={"prob": 1.0}, eq={"prob": 1.0}, n_noises={"p": {"3": 1.0}}, reverb={"prob": 1.0})
        self.check(plans(K, range(8)))

    def test_near_silent_and_no_reverb(self):
        K = knobs(near_silent={"on": True, "frac": 0.5}, reverb={"on": False})
        ps = plans(K, range(10), with_rir=False)
        self.assertTrue(any(p.silent for p in ps) and not all(p.silent for p in ps))
        self.check(ps)

    def test_all_knobs_off(self):
        self.check(plans(knobs(**{k: {"on": False} for k in RECIPE["knobs"]}), range(6)))

    def test_mixed_rir_lengths_in_batch(self):
        ps = plans(knobs(), range(4)) + plans(knobs(reverb={"on": False}), range(4, 8), with_rir=False)
        self.check(ps)

    def test_float32_waveform(self):
        """float32 下波形仍 ≤ 1e-4（有声区间在帧门限边上可能翻一帧，不在这里比）。"""
        self.check(plans(knobs(), range(8)), dtype=torch.float32, ivs=False)

    def test_level_and_intervals_match_dsp(self):
        """gpu_mix 的电平 / 区间是 exp013 dsp.py 的第二处表示：默认参数相同，随机信号上逐个相同。"""
        import dsp

        self.assertEqual(inspect.signature(dsp.gated_level_db).parameters["rel_db"].default, self.g.REL_DB)
        self.assertEqual(inspect.signature(dsp.intervals).parameters["rel_db"].default, self.g.REL_DB)
        self.assertEqual(inspect.signature(dsp.intervals).parameters["guard"].default, self.g.GUARD)
        rng = np.random.default_rng(0)
        xs = np.stack([speech(N, s) * rng.uniform(0.01, 1) for s in range(6)] + [np.zeros(N)])
        t = torch.as_tensor(xs, dtype=torch.float64)
        lv = self.g.gated_level_db(t).numpy()
        act = self.g.activity(t).numpy()
        for i, x in enumerate(xs):
            self.assertAlmostEqual(float(lv[i]), dsp.gated_level_db(x), places=9)
            self.assertEqual(self.g.runs(act[i], N), dsp.intervals(x))

    def test_biquad_matches_lfilter(self):
        from scipy.signal import lfilter

        rng = np.random.default_rng(3)
        x = rng.standard_normal(4000)
        for _ in range(20):
            c = mix.rand_biquad(rng, 0.375)
            Nf = self.g._nfft(len(x) + self.g.EQ_TAIL)
            H = self.g._biquad_freq(torch.as_tensor(c), Nf)
            y = torch.fft.irfft(torch.fft.rfft(torch.as_tensor(x), Nf) * H, Nf)[: len(x)].numpy()
            np.testing.assert_allclose(y, lfilter(c[:3], c[3:], x), atol=1e-9)


class PlanShape(unittest.TestCase):
    def test_plan_is_pure_input(self):
        """plan 之后不再抽随机数：同一个 plan 渲染两次结果相同。"""
        p = plans(knobs(), [1])[0]
        a, b = mix.render(p, N), mix.render(p, N)
        np.testing.assert_array_equal(a[0], b[0])
        self.assertEqual(p.noises.shape, (mix.MAX_NOISES, N))

    def test_max_noises_covers_recipe(self):
        self.assertGreaterEqual(mix.MAX_NOISES, max(int(k) for k in RECIPE["knobs"]["n_noises"]["p"]))


if __name__ == "__main__":
    unittest.main()
