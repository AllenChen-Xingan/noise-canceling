"""exp019 的测试：结构候选的正确性与计数口径。

跑法（在 exp019 目录下）：python tests/test_exp019.py -v
需要主目录的 `resources/gtcrn/`（官方代码与检查点）；没有时整组跳过。
"""
import sys
import unittest
from pathlib import Path

import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

try:
    import gtcrn_src
    gtcrn_src.stream_dir()
    HAVE = True
except Exception as e:  # 没有官方代码就跳过，并说明原因
    HAVE, WHY = False, str(e)

if HAVE:
    import macs
    import model
    import variants

# 官方 README / 源码头注释给的数（外部第二处表示，和我们的计数比对）
OFFICIAL_PARAMS = 23_670     # "23.67 K params"（可训练参数）
OFFICIAL_MMACS_PER_S = 33.0  # "33.0 MMACs"，ptflops 在 63 帧（约 1 s）上量
FRAMES_PER_S = 16000 / 256


def frames(n, seed=0):
    g = torch.Generator().manual_seed(seed)
    return torch.randn(n, 1, 257, 1, 2, generator=g) * 0.1


def run(m, xs, cond=None):
    st, ys = m.init_state(1), []
    with torch.no_grad():
        for x in xs:
            y, st = m(x, st, cond)
            ys.append(y)
    return torch.cat(ys, dim=2)


@unittest.skipUnless(HAVE, "缺官方 GTCRN 源码，人工跑")
class Exp019(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        torch.manual_seed(0)
        cls.xs = frames(40)
        cls.models = {n: model.make(n) for n in variants.NAMES}
        cls.ref = run(cls.models["base"], cls.xs)

    def test_base_equals_official_stream(self):
        """包装后的 base 与官方 StreamGTCRN 逐帧逐值相同（串接顺序没写错）。"""
        _, st = gtcrn_src.load_stream_model()
        conv = torch.zeros(2, 1, 16, 16, 33)
        tra = torch.zeros(2, 3, 1, 1, 16)
        inter = torch.zeros(2, 1, 33, 16)
        ys = []
        with torch.no_grad():
            for x in self.xs:
                y, conv, tra, inter = st(x, conv, tra, inter)
                ys.append(y)
        torch.testing.assert_close(self.ref, torch.cat(ys, dim=2), rtol=0, atol=1e-6)

    def test_zero_init_equals_base(self):
        """每个候选刚接上时输出与原模型相同 → 能从现有权重热启动。"""
        cond = torch.randn(1, 8)
        for n in variants.NAMES:
            with self.subTest(n=n):
                y = run(self.models[n], self.xs, cond if n.startswith("film") else None)
                torch.testing.assert_close(y, self.ref, rtol=0, atol=1e-6)

    def _perturbed(self, n):
        m = model.make(n)
        torch.manual_seed(1)
        with torch.no_grad():
            for p in m.extra.parameters():
                p.add_(torch.randn_like(p) * 0.3)
        return m

    def test_perturbed_changes_output(self):
        """反向对照：候选权重不为零时输出确实变了（证明上一条不是空测）。"""
        for n in variants.NAMES[1:]:
            with self.subTest(n=n):
                m = self._perturbed(n)
                cond = torch.randn(1, 8) if n.startswith("film") else None
                y = run(m, self.xs, cond)
                self.assertGreater((y - self.ref).abs().max().item(), 1e-4)

    def test_causal_no_lookahead(self):
        """改第 t 帧之后的输入，第 t 帧及之前的输出不变 → 不看未来帧，不加延迟。"""
        t = 25
        xs2 = self.xs.clone()
        xs2[t + 1:] = frames(len(xs2) - t - 1, seed=9)
        for n in variants.NAMES:
            with self.subTest(n=n):
                m = self._perturbed(n) if n != "base" else self.models[n]
                a, b = run(m, self.xs), run(m, xs2)
                torch.testing.assert_close(a[:, :, : t + 1], b[:, :, : t + 1], rtol=0, atol=0)
                self.assertGreater((a[:, :, t + 1:] - b[:, :, t + 1:]).abs().max().item(), 0)

    @staticmethod
    def _module_outputs(mod, seq, cond=None):
        st, ys = mod.init_state(1), []
        with torch.no_grad():
            for x in seq:
                y, st = mod(x, st, cond)
                ys.append(y)
        return ys

    def test_branch_uses_current_frame(self):
        """没有延后一帧：只改模块第 t 帧的输入，模块支路（输出 − 输入）在第 t 帧就变。

        直接测候选模块（整模型的掩码乘当前频谱，会掩盖内部延后）。纯延后一帧的模块过不了。
        """
        t = 6
        g = torch.Generator().manual_seed(5)
        seq = torch.randn(10, 1, 16, 1, 33, generator=g)
        seq2 = seq.clone()
        seq2[t] = torch.randn(1, 16, 1, 33, generator=g)
        for n in variants.NAMES[1:]:
            with self.subTest(n=n):
                mod = self._perturbed(n).extra
                cond = torch.randn(1, 8) if n.startswith("film") else None
                a = self._module_outputs(mod, seq, cond)
                b = self._module_outputs(mod, seq2, cond)
                da, db = a[t] - seq[t], b[t] - seq2[t]
                self.assertGreater((da - db).abs().max().item(), 1e-6)
                torch.testing.assert_close(a[t - 1], b[t - 1], rtol=0, atol=0)

    def test_tattn_window_is_k_frames(self):
        """时间注意力正好看最近 K 帧：最老的窗内帧改了，输出变；紧邻窗外那帧改了，输出不变。

        直接测模块本身（不经 GRU，GRU 记得更久）。K = 16 / 32 / 64 各测一遍。
        """
        g = torch.Generator().manual_seed(3)
        for k in (16, 32, 64):
            with self.subTest(k=k):
                att = self._perturbed(f"tattn-k{k}").extra
                n = k + 5
                seq = torch.randn(n, 1, 16, 1, 33, generator=g)
                last = self._module_outputs(att, seq)[-1]
                for idx, should_change in ((n - k, True), (n - k - 1, False)):
                    s2 = seq.clone()
                    s2[idx] = torch.randn(1, 16, 1, 33, generator=g)
                    y = self._module_outputs(att, s2)[-1]
                    diff = (y - last).abs().max().item()
                    if should_change:
                        self.assertGreater(diff, 1e-6, f"窗内最老帧 {idx} 没起作用")
                    else:
                        self.assertEqual(diff, 0.0, f"窗外帧 {idx} 影响了输出")
                self.assertEqual(tuple(att.init_state(1)[0].shape), (1, 33, k - 1, 16))

    def test_film_zero_cond_is_bypass_after_training(self):
        """场景条件模块权重已训（非零）时，输入全零向量仍与不加条件的基座逐值相同（无偏置）。"""
        m = self._perturbed("film-d8")
        y = run(m, self.xs, torch.zeros(1, 8))
        torch.testing.assert_close(y, self.ref, rtol=0, atol=0)

    def test_params_match_official(self):
        """可训练参数与官方 23.67 K 一致（误差 ≤ 10 个）。"""
        n = macs.count_params(self.models["base"], trainable_only=True)
        self.assertLessEqual(abs(n - OFFICIAL_PARAMS), 10, n)

    def test_macs_near_official(self):
        """每帧乘加与官方量级一致：我们不计归一化 / 激活 / 逐元素，故落在官方的 75%–100%。"""
        ours = macs.count_frame(self.models["base"])["total"]
        theirs = OFFICIAL_MMACS_PER_S * 1e6 / 63
        self.assertTrue(0.75 <= ours / theirs <= 1.0, ours / theirs)

    def test_extra_macs_formulas(self):
        """候选的乘加与手算公式一致（第二处表示：比较卡里的公式）。"""
        C, F = 16, 33
        expect = {
            "film-d8": 8 * 2 * C,
            "fattn": F * C * 3 * C + F * C * C + 2 * F * F * C,
        }
        for k in (16, 32, 64):
            expect[f"tattn-k{k}"] = F * C * 3 * C + F * C * C + 2 * F * k * C
        for n, v in expect.items():
            with self.subTest(n=n):
                self.assertEqual(macs.count_frame(self.models[n])["extra"], v)

    def test_dpgrnn3_costs_one_block(self):
        """多一层双路 RNN 的乘加 = 原来第一层双路 RNN 的乘加（同形，钩子只挂在那一层上数）。"""
        one = macs.count_frame(self.models["dpgrnn3"])["extra"]
        m = self.models["base"]
        tally, hooks = [0], []
        for mod in m.base.dpgrnn1.modules():
            for types, fn in macs.RULES:
                if isinstance(mod, types):
                    hooks.append(mod.register_forward_hook(
                        lambda mm, i, o, fn=fn: tally.__setitem__(
                            0, tally[0] + fn(mm, i, o[0] if isinstance(o, tuple) else o))))
        try:
            with torch.no_grad():
                m(self.xs[0], m.init_state(1))
        finally:
            for h in hooks:
                h.remove()
        self.assertEqual(one, tally[0])


if __name__ == "__main__":
    unittest.main()
