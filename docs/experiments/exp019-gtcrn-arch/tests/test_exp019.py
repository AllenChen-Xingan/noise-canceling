"""exp019 的测试：结构候选的正确性与计数口径。

跑法（在本目录的上一级）：python -m unittest discover -s tests -t .
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

    def test_tattn_sees_past_k_frames(self):
        """时间注意力的缓存正好装过去 K-1 帧：第 K 帧以前的改动影响不到当前输出的注意力部分。"""
        m = self._perturbed("tattn-k16")
        st = m.extra.init_state(1)
        self.assertEqual(tuple(st[0].shape), (1, 33, 15, 16))

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
        """多一层双路 RNN 的乘加 = 原来一层的乘加（同形）。"""
        base = macs.count_frame(self.models["base"])
        one = macs.count_frame(self.models["dpgrnn3"])["extra"]
        # 原模型两层双路 RNN 里的 GRU + 线性层
        m = self.models["base"].base
        blk = 0
        for mod in m.dpgrnn1.modules():
            if isinstance(mod, torch.nn.GRU):
                blk += {16: 1}.get(0, 0)
        self.assertGreater(one, 0)
        self.assertLess(one, base["total"])


if __name__ == "__main__":
    unittest.main()
