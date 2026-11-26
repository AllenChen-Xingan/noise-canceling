"""候选噪声的机器筛查（screen.py）：判据在 screen_rules.json，一个阈值都不许写进代码。

- 混响时间：合成冲激响应的真值已知，估到 10 % 以内；衰减没走到 -35 dB 的必须报错
- 人声与标签：模型的判据用假函数喂，不装 Silero 与 PANNs
- 声学量：A 计权电平、长期频谱斜率、逐秒电平标准差，都用已知真值的合成分量核
- 判据只活在 json 里：改一份临时副本里的阈值，判定结果要跟着翻；代码里出现阈值字面量即失败
"""
import copy
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
import soundfile as sf

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import screen  # noqa: E402

RULES_PATH = HERE / "screen_rules.json"
RULES = json.loads(RULES_PATH.read_text(encoding="utf-8"))
SCENE = "meeting-room"
SR = 16000


def noise(seconds, sr=SR, amp=0.1, seed=0):
    rng = np.random.default_rng(seed)
    return (amp * rng.standard_normal(int(seconds * sr))).astype(np.float64)


def decay_rir(T, sr=SR, seed=0):
    """真混响时间 T：白噪声乘 exp(-6.91 t / T)，能量衰减速率正好是每秒 60/T dB。"""
    n = int(sr * (1.5 * T + 0.05))
    t = np.arange(n) / sr
    rng = np.random.default_rng(seed)
    return rng.standard_normal(n) * np.exp(-6.91 * t / T)


def truncated_rir(T, drop_db=20.0, sr=SR, seed=1):
    """衰减只走到 drop_db 就停的冲激响应（尾巴被截断 / 补零）。"""
    n = max(4, int(sr * T * drop_db / 60.0))
    t = np.arange(n) / sr
    rng = np.random.default_rng(seed)
    return rng.standard_normal(n) * np.exp(-6.91 * t / T)


def fake_vad(prob):
    def vad(x16k):
        n = max(1, len(x16k) // 480)  # 30 毫秒一帧
        return np.full(n, prob, dtype=np.float64)

    return vad


def mixed_vad(frac, high=0.9):
    """前 frac 比例的帧说成有人声、其余没有（占比可调，好测阈值移动）。"""

    def vad(x16k):
        n = max(2, len(x16k) // 480)
        p = np.zeros(n, dtype=np.float64)
        p[:int(round(frac * n))] = high
        return p

    return vad


def fake_tagger(probs):
    return lambda x32k: dict(probs)


def pink(n, sr=SR, seed=0):
    """1/f 功率的粉红噪声（每倍频程 -3 dB）。"""
    rng = np.random.default_rng(seed)
    X = np.fft.rfft(rng.standard_normal(n))
    f = np.fft.rfftfreq(n, 1.0 / sr)
    f[0] = f[1]
    y = np.fft.irfft(X / np.sqrt(f), n)
    return 0.5 * y / np.max(np.abs(y))


def scene_probs(**extra):
    """会议室能过的标签概率：前 top_k 里有「Air conditioning」。"""
    d = {"Air conditioning": 0.8, "Hum": 0.1, "Inside, large room or hall": 0.05,
         "Speech": 0.02, "Music": 0.01}
    d.update(extra)
    return d


class Rt60(unittest.TestCase):
    def test_synthetic_decay_within_ten_percent(self):
        for T in (0.2, 0.6, 1.2):
            got = screen.rt60_schroeder(decay_rir(T), SR)
            self.assertAlmostEqual(got, T, delta=0.1 * T, msg=f"真值 {T} 秒，估出 {got:.3f}")

    def test_decay_that_never_reaches_minus_35_db_raises(self):
        with self.assertRaises(ValueError):
            screen.rt60_schroeder(truncated_rir(0.5), SR)
        # 尾巴补零的也要报错：不能拿后面那串静音去当衰减
        with self.assertRaises(ValueError):
            screen.rt60_schroeder(np.concatenate([truncated_rir(0.5), np.zeros(4000)]), SR)

    def test_empty_rir_raises(self):
        with self.assertRaises(ValueError):
            screen.rt60_schroeder(np.zeros(100), SR)


class Speech(unittest.TestCase):
    def test_speech_frac_counts_frames_above_threshold(self):
        probs = np.array([0.0, 0.2, 0.9, 0.7, 0.1])
        self.assertAlmostEqual(screen.speech_frac(probs, 0.5), 0.4)
        self.assertAlmostEqual(screen.speech_frac(probs, 0.95), 0.0)

    def test_frames_speech_prob_uses_the_injected_vad(self):
        x = noise(0.5)
        probs = screen.frames_speech_prob(x, SR, fake_vad(0.8))
        self.assertEqual(probs.shape, (int(0.5 * SR) // 480,))
        self.assertTrue(np.all(probs == 0.8))

    def test_screen_clip_drops_above_max_frac_and_keeps_below(self):
        x = noise(2.5)
        sp = RULES["speech"]
        dirty = screen.screen_clip(x, SR, SCENE, RULES, vad=fake_vad(0.9), tagger=fake_tagger(scene_probs()))
        self.assertEqual(dirty["verdict"], "drop")
        self.assertGreater(dirty["measures"]["speech_frac"], sp["max_frac"])
        self.assertTrue(any("说话帧占比" in r for r in dirty["reasons"]))
        clean = screen.screen_clip(x, SR, SCENE, RULES, vad=fake_vad(0.0), tagger=fake_tagger(scene_probs()))
        self.assertEqual(clean["verdict"], "keep", clean["reasons"])


class Tagger(unittest.TestCase):
    def test_top_tags_sorted_and_cut(self):
        tags = screen.top_tags(noise(1.0), SR, fake_tagger(scene_probs()), 3)
        self.assertEqual(len(tags), 3)
        self.assertEqual(tags[0][0], "Air conditioning")
        self.assertEqual([p for _, p in tags], sorted([p for _, p in tags], reverse=True))

    def test_scene_tag_present_keeps_absent_drops(self):
        x = noise(2.5)
        ok = screen.screen_clip(x, SR, SCENE, RULES, vad=fake_vad(0.0), tagger=fake_tagger(scene_probs()))
        self.assertEqual(ok["verdict"], "keep", ok["reasons"])
        bad = screen.screen_clip(x, SR, SCENE, RULES, vad=fake_vad(0.0),
                                 tagger=fake_tagger({"Crowd": 0.6, "Aircraft": 0.2, "Music": 0.1,
                                                      "Speech": 0.02, "Hum": 0.01}))
        self.assertEqual(bad["verdict"], "drop")
        self.assertTrue(any(SCENE in r for r in bad["reasons"]))

    def test_speech_tag_over_limit_drops_and_names_speech(self):
        x = noise(2.5)
        res = screen.screen_clip(x, SR, SCENE, RULES, vad=fake_vad(0.0),
                                 tagger=fake_tagger(scene_probs(**{"Speech": 0.5})))
        self.assertEqual(res["verdict"], "drop")
        self.assertTrue(any(RULES["speech"]["tag"] in r for r in res["reasons"]), res["reasons"])

    def test_missing_models_are_reported_as_not_checked(self):
        res = screen.screen_clip(noise(2.5), SR, SCENE, RULES)
        self.assertEqual(res["verdict"], "keep", res["reasons"])
        not_checked = [r for r in res["reasons"] if r.startswith("未查")]
        self.assertGreaterEqual(len(not_checked), 2)  # 人声与标签，还应有混响时间
        self.assertTrue(any("混响" in r for r in not_checked), not_checked)


class Rir(unittest.TestCase):
    def test_rt60_outside_range_drops(self):
        rir = decay_rir(1.6)  # 会议室判据是 0.3–1.0 秒
        res = screen.screen_clip(noise(2.5), SR, SCENE, RULES, vad=fake_vad(0.0),
                                 tagger=fake_tagger(scene_probs()), rir=rir)
        self.assertEqual(res["verdict"], "drop")
        self.assertTrue(any("混响时间" in r for r in res["reasons"]))
        inside = screen.screen_clip(noise(2.5), SR, SCENE, RULES, vad=fake_vad(0.0),
                                    tagger=fake_tagger(scene_probs()), rir=decay_rir(0.6))
        self.assertEqual(inside["verdict"], "keep", inside["reasons"])


class Level(unittest.TestCase):
    def test_steady_noise_is_stationary(self):
        self.assertLess(screen.level_std_db(noise(4.0), SR), 1.0)

    def test_twenty_db_step_is_not(self):
        x = np.concatenate([noise(2.0, amp=0.1), noise(2.0, amp=0.01, seed=5)])
        self.assertGreater(screen.level_std_db(x, SR), 6.0)

    def test_too_short_raises(self):
        with self.assertRaises(ValueError):
            screen.level_std_db(noise(0.5), SR)


class AWeight(unittest.TestCase):
    def sine(self, f, seconds=1.0, amp=1.0, sr=SR):
        t = np.arange(int(seconds * sr)) / sr
        return amp * np.sin(2 * np.pi * f * t)

    def test_one_khz_full_scale_is_minus_three_dbfs(self):
        self.assertAlmostEqual(screen.a_weighted_dbfs(self.sine(1000), SR), -3.0, delta=0.2)

    def test_hundred_hz_is_nineteen_db_lower(self):
        hi = screen.a_weighted_dbfs(self.sine(1000), SR)
        lo = screen.a_weighted_dbfs(self.sine(100), SR)
        self.assertAlmostEqual(hi - lo, 19.1, delta=0.5)


class Ltas(unittest.TestCase):
    def test_white_noise_slope_is_flat(self):
        self.assertAlmostEqual(screen.ltas_slope_db_per_oct(noise(4.0), SR), 0.0, delta=1.0)

    def test_pink_noise_slope_is_minus_three_per_octave(self):
        got = screen.ltas_slope_db_per_oct(pink(4 * SR), SR)
        self.assertAlmostEqual(got, -3.0, delta=1.0)


class Rules(unittest.TestCase):
    def test_every_scene_exists_in_exp013(self):
        scenes = json.loads((HERE.parents[1] / "exp013-scene-suite" / "scenes.json")
                            .read_text(encoding="utf-8"))["scenes"]
        for name in RULES["scenes"]:
            self.assertIn(name, scenes, f"screen_rules.json 的 {name} 不是 exp013 的场景")

    def test_threshold_in_temp_copy_moves_verdict(self):
        x = noise(2.5)
        vad = mixed_vad(0.2)
        tagger = fake_tagger(scene_probs())
        base = screen.screen_clip(x, SR, SCENE, RULES, vad=vad, tagger=tagger)
        self.assertEqual(base["verdict"], "drop")
        loose = copy.deepcopy(RULES)
        loose["speech"]["max_frac"] = 0.3
        self.assertEqual(screen.screen_clip(x, SR, SCENE, loose, vad=vad, tagger=tagger)["verdict"], "keep")
        # 平稳度那条阈值同样只在 json 里
        stepped = np.concatenate([noise(2.0, amp=0.1), noise(2.0, amp=0.01, seed=5)])
        self.assertEqual(screen.screen_clip(stepped, SR, SCENE, RULES, vad=fake_vad(0.0),
                                           tagger=tagger)["verdict"], "drop")
        flat = copy.deepcopy(RULES)
        flat["stationarity"]["max_level_std_db"] = 30.0
        self.assertEqual(screen.screen_clip(stepped, SR, SCENE, flat, vad=fake_vad(0.0),
                                           tagger=tagger)["verdict"], "keep")

    def test_no_threshold_literal_in_code(self):
        lines = (HERE / "screen.py").read_text(encoding="utf-8").splitlines()
        code = "\n".join(ln for ln in lines if not ln.strip().startswith("#"))
        for lit in ("0.01", "0.5", "6.0"):
            self.assertNotIn(lit, code, f"阈值 {lit} 写死在 screen.py 里了，应只放在 screen_rules.json")


class Cli(unittest.TestCase):
    def test_no_models_writes_one_line_per_file(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            names = ["a.wav", "b.wav", "c.wav"]
            for i, n in enumerate(names):
                sf.write(str(root / n), noise(1.0, seed=i), SR, subtype="PCM_16")
            (root / "files.txt").write_text("# 注释行不算\n" + "\n".join(names) + "\n", encoding="utf-8")
            out = root / "out.jsonl"
            p = subprocess.run([sys.executable, str(HERE / "screen.py"), "--root", str(root),
                                "--list", str(root / "files.txt"), "--scene", SCENE,
                                "--out", str(out), "--no-models"],
                               capture_output=True, text=True)
            self.assertEqual(p.returncode, 0, p.stderr)
            lines = out.read_text(encoding="utf-8").strip().splitlines()
            self.assertEqual(len(lines), 3)
            for line in lines:
                rec = json.loads(line)
                self.assertEqual(set(rec) & {"path", "sha256", "sr", "seconds", "verdict", "reasons",
                                             "measures"},
                                 {"path", "sha256", "sr", "seconds", "verdict", "reasons", "measures"})
                self.assertEqual(rec["sr"], SR)
                self.assertTrue(any("未查" in r for r in rec["reasons"]), rec["reasons"])
                self.assertEqual(rec["verdict"], "keep")
                self.assertEqual(len(rec["sha256"]), 64)
            self.assertIn("keep 3", p.stdout)


if __name__ == "__main__":
    unittest.main()
