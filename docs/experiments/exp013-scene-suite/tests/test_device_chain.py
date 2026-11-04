"""设备链：参数文件与 022 文档交叉校验 + 行为测试。"""
import sys
import unittest
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import device_chain as dc  # noqa: E402
from dsp import SR  # noqa: E402
from rooms import make_rir  # noqa: E402

ROOT = HERE.parents[2]


def num(v):
    """数字按 022 文档写法：负号用 −，整数不带小数点。"""
    s = f"{abs(v):g}"
    return ("−" if v < 0 else "") + s


def signed(v):
    return "0" if v == 0 else (("+" if v > 0 else "−") + f"{abs(v):g}")


def freq(lo, hi):
    if hi >= 1000:
        return (f"{lo / 1000:g}–{hi / 1000:g} kHz" if lo != hi else f"{lo / 1000:g} kHz")
    return f"{lo:g}–{hi:g} Hz" if lo != hi else f"{lo:g} Hz"


def rng_txt(r, sep="–"):
    return f"{num(r[0])}{sep}{num(r[1])}"


def expected_fragments(P):
    s1, s2, s3, s4, s6, s7, s8 = (P[k] for k in ("step1_room", "step2_direction", "step3_response", "step4_floor", "step6_level", "step7_vpio", "step8_clip"))
    d = s1["owner_dist_m"]
    st = s2["strength"]
    b1, b2 = s2["bands_hz"]
    dz = s2["diffuse_db"]
    curve = "、".join(f"{freq(lo, hi)} {signed(v)}" for lo, hi, v in s3["curve_db"]) + " dB"
    pk = s3["peak"]
    t90, t180 = s2["azimuth_table"]["90"], s2["azimuth_table"]["180"]

    def cell(r):  # 文档写「−3.4 到 −4.2」：绝对值小的在前；两端相同写一个
        a, b = sorted(r, key=abs)
        one = lambda v: ("−" if v < 0 else "") + f"{abs(v):.1f}"
        return one(a) if a == b else f"{one(a)} 到 {one(b)}"

    tilt = s3["tilt_db_per_oct"]
    return [
        f"说话人距屏幕 {d['range'][0]:g}–{d['range'][1]:g} m（{d['default']:g}）",
        f"直达峰前后约 {s2['direct_window_ms']:g} ms",
        f"低频（< {b1} Hz）、中频（{b1}–{b2} Hz）、高频（> {b2 / 1000:g} kHz）",
        f"强度系数 {st['range'][0]:.1f}–{st['range'][1]:.1f}（{st['default']:.1f}）",
        f"中频 {num(dz['mid'])} dB、高频 {num(dz['high'])} dB",
        f"混响压力」分支（概率 {s2['reverb_stress_prob']:g}）",
        f"前方 ±{s2['front_flat_deg']}° 不衰减",
        f"另以 {s3['flat_prob']:g} 的概率改用平坦曲线（只留 {s3['flat_highpass_hz']} Hz 高通）",
        f"目标曲线（相对 1 kHz）：{curve}",
        f"±{abs(tilt[0]):g} dB/倍频程",
        f"1 个峰值滤波（{pk['freq_hz'][0] / 1000:g}–{pk['freq_hz'][1] / 1000:g} kHz，±{abs(pk['gain_db'][0]):g} dB，Q {pk['q'][0]:g}–{pk['q'][1]:g}）",
        f"处理后底噪之比 {s4['ratio_db']['range'][0]}–{s4['ratio_db']['range'][1]} dB（{s4['ratio_db']['default']}）",
        f"{s4['lowmid_band_hz'][0]}–{s4['lowmid_band_hz'][1]} Hz 比 {s4['high_band_hz'] / 1000:g} kHz 以上高 {rng_txt(s4['lowmid_excess_db'])} dB",
        f"概率 {P['step5_fan']['prob']}" if P["step5_fan"]["prob"] == 0 else "?",
        f"{num(s6['mean_dbfs'])} dBFS 为中心，σ {s6['sigma_db']} dB（{num(s6['clip_range'][0])} 到 {num(s6['clip_range'][1])} 截断）",
        f"概率 {s7['prob']:g}，整段 {num(s7['gain_db'][1])} 到 {num(s7['gain_db'][0])} dB",
        f"概率 {s8['prob']:g}；驱动 {s8['drive_db'][0]}–{s8['drive_db'][1]} dB",
        f"| ±90° | {cell(t90['low'])} | {cell(t90['mid'])} | {cell(t90['high'])} |",
        f"| 180° | {cell(t180['low'])} | {cell(t180['mid'])} | {cell(t180['high'])} |",
        f"重采样到模型采样率（{P['step9_output']['sr'] // 1000} kHz），{P['step9_output']['dtype']}",
    ]


class DocCrossCheck(unittest.TestCase):
    def test_every_param_in_022_doc(self):
        doc = (ROOT / dc.P["source_doc"]).read_text()
        flat = doc.replace("**", "")
        for frag in expected_fragments(dc.P):
            with self.subTest(frag=frag):
                self.assertIn(frag, flat, f"参数文件里的数字在 022 文档里找不到：{frag}")

    def test_negative_control(self):
        """改一个数，交叉校验必须能抓到（防止检查本身是空的）。"""
        import copy

        doc = (ROOT / dc.P["source_doc"]).read_text().replace("**", "")
        bad = copy.deepcopy(dc.P)
        bad["step8_clip"]["prob"] = 0.2
        misses = [f for f in expected_fragments(bad) if f not in doc]
        self.assertEqual(len(misses), 1)


class Behaviour(unittest.TestCase):
    def test_external_mic_is_identity_except_level(self):
        rng = np.random.default_rng(1)
        ch = dc.Chain(rng, "external-mic", "stress")
        x = rng.standard_normal(SR)
        np.testing.assert_array_equal(ch.response(x), x)
        np.testing.assert_array_equal(ch.diffuse(x), x)
        self.assertEqual(float(np.abs(ch.floor(100, -30, rng)).max()), 0.0)
        self.assertIsNone(ch.params["clip_drive_db"])

    def test_front_source_not_attenuated(self):
        rng = np.random.default_rng(2)
        for az in (0, 20, -30):
            a = dc.azimuth_atten(dc.draw_nodes(rng), az, 1.0)
            self.assertEqual(a, {"low": 0.0, "mid": 0.0, "high": 0.0})

    def test_side_attenuation_in_table_range(self):
        rng = np.random.default_rng(3)
        t = dc.P["step2_direction"]["azimuth_table"]
        for _ in range(50):
            for az, key in ((90, "90"), (-90, "90"), (180, "180")):
                a = dc.azimuth_atten(dc.draw_nodes(rng), az, 1.0)
                for b in ("low", "mid", "high"):
                    lo, hi = min(t[key][b]), max(t[key][b])
                    self.assertTrue(lo - 1e-9 <= a[b] <= hi + 1e-9, (az, b, a[b]))

    def test_direct_part_attenuated_high_band(self):
        """侧面源的直达部分高频应被压 ≥ 17.5 × 0.6 dB；混响尾部按扩散场只压 6 dB 量级。"""
        rng = np.random.default_rng(4)
        ch = dc.Chain(rng, "mba-builtin", "common")
        ch.params["strength"], ch.params["reverb_stress"] = 1.0, False
        r = make_rir(rng, (6, 5, 3), 0.4, 1.5, 90)
        r.rest = np.zeros_like(r.rest)  # 只看直达部分
        h, pad, att = ch.source_rir(r, rng)
        self.assertLessEqual(att["high"], -17.5)
        n = 4096
        f = np.fft.rfftfreq(n, 1 / SR)
        hd_in = np.abs(np.fft.rfft(r.direct, n))
        w = int(2.5e-3 * SR)
        k = int(np.argmax(np.abs(r.direct)))
        hd_out = np.abs(np.fft.rfft(h[pad + k - w - 40 : pad + k + w + 40], n))
        band = (f > 4000) & (f < 7000)
        drop = 20 * np.log10(np.mean(hd_out[band]) / np.mean(hd_in[band]))
        self.assertLess(drop, att["high"] + 3)

    def test_response_curve_hits_doc_points(self):
        p = {"flat": False, "tilt_db_per_oct": 0.0, "peak_hz": 2000.0, "peak_db": 0.0, "peak_q": 1.0}
        g = dc.response_curve(p, np.array([100.0, 1000.0, 4000.0, 6000.0]))
        np.testing.assert_allclose(g, [-10, 0, 3, 6], atol=0.05)

    def test_fir_matches_curve(self):
        rng = np.random.default_rng(5)
        ch = dc.Chain(rng, "mba-builtin", "common")
        n = 16384
        f = np.fft.rfftfreq(n, 1 / SR)
        H = 20 * np.log10(np.abs(np.fft.rfft(ch._fir, n)) + 1e-12)
        want = dc.response_curve(ch.params, f)
        sel = (f > 200) & (f < 7000)
        self.assertLess(np.max(np.abs(H[sel] - want[sel])), 1.0)

    def test_floor_ratio(self):
        rng = np.random.default_rng(6)
        ch = dc.Chain(rng, "mba-builtin", "common")
        fl = ch.floor(SR * 4, -30.0, rng)
        lvl = 10 * np.log10(np.mean(fl**2))
        self.assertAlmostEqual(lvl, -30.0 - ch.params["floor_ratio_db"], places=3)

    def test_clip_only_in_stress(self):
        rng = np.random.default_rng(7)
        for _ in range(300):
            self.assertIsNone(dc.Chain(rng, "mba-builtin", "common").params["clip_drive_db"])

    def test_clip_happens_in_stress_at_doc_rate(self):
        """压力档按 022 第 8 步的概率（0.03）削波；第一到三版 90 段压力档碰巧一段都没抽到（概率约 6%），这里证明分支是活的。"""
        rng = np.random.default_rng(11)
        n = 4000
        hits = sum(dc.Chain(rng, "mba-builtin", "stress").params["clip_drive_db"] is not None for _ in range(n))
        p = dc.P["step8_clip"]["prob"]
        self.assertAlmostEqual(hits / n, p, delta=4 * (p * (1 - p) / n) ** 0.5)
        mix = np.sin(np.linspace(0, 200, SR))
        ch = dc.Chain(rng, "mba-builtin", "stress")
        ch.params["clip_drive_db"] = 6.0
        y, clipped = ch.clip(mix)
        self.assertTrue(clipped)
        self.assertGreater(float(np.max(np.abs(y - mix))), 0.1)

    def test_same_clip_same_direction(self):
        """回归（codex 第 3 轮 P2 #3）：同一段里同方位的两个声源，方位衰减必须相同（节点每段抽一次）。"""
        rng = np.random.default_rng(12)
        ch = dc.Chain(rng, "mba-builtin", "common")
        r = make_rir(rng, (6, 5, 3), 0.4, 1.5, 90)
        _, _, a1 = ch.source_rir(r, rng)
        _, _, a2 = ch.source_rir(r, rng)
        self.assertEqual(a1, a2)

    def test_same_seed_same_params(self):
        a = dc.Chain(np.random.default_rng(9), "mba-builtin", "stress").params
        b = dc.Chain(np.random.default_rng(9), "mba-builtin", "stress").params
        self.assertEqual(a, b)


if __name__ == "__main__":
    unittest.main()
