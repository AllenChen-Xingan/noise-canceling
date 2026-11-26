"""exp025 · 整段版 GTCRN 离线脚本（run_gtcrn_offline.py）与两条路对拍（parity.py）的测试。

不跑真模型、不用 onnxruntime、不下载任何东西（恒等模型 + 假会话就够了）。
用法（项目根下，python 要有 numpy）；本文件在 eval/ 目录里，所以按文件名跑：
  cd docs/experiments/exp025-gtcrn-train/eval && <python> -m unittest -v test_offline
"""
import contextlib
import io
import json
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import parity  # noqa: E402
import run_gtcrn_offline as off  # noqa: E402

RNG = np.random.default_rng(7)


def noise(n):
    return (RNG.standard_normal(n) * 0.1).astype(np.float32)


class Frontend(unittest.TestCase):
    """口径是第二处表示：与两份正本交叉校验（数字不手抄）。"""

    def test_constants_come_from_trainer_and_match_rust(self):
        self.assertIn(f"N_FFT, HOP = {off.N_FFT}, {off.HOP}", off.LOSSES.read_text())
        src = (off.ROOT / "src/types/src/frontend.rs").read_text()
        block = re.search(r"pub const FRONTEND: StftSpec = StftSpec \{(.*?)\};", src, re.S).group(1)
        n_fft = int(re.search(r"^\s*n_fft: (\d+),", block, re.M).group(1))
        hop = int(re.search(r"^\s*hop: (\d+),", block, re.M).group(1))
        self.assertEqual((off.N_FFT, off.HOP), (n_fft, hop))

    def test_window_is_sqrt_hann_half_overlap(self):
        w = off.WIN.astype(np.float64)
        self.assertEqual(len(w), off.N_FFT)
        self.assertAlmostEqual(float(w[0]), 0.0, places=7)
        self.assertAlmostEqual(float(w[off.N_FFT // 2]), 1.0, places=7)
        # 50% 重叠时窗平方之和恒为 1（round-trip 能还原的前提，Rust `frontend.rs` 的注释同一句）
        v = w ** 2
        np.testing.assert_allclose(v[: off.HOP] + v[off.HOP:], 1.0, atol=1e-6)

    def test_declared_latency_is_what_rust_declares(self):
        src = (off.ROOT / "src/models/src/lib.rs").read_text()
        m = re.search(r"ModelKind::Gtcrn => Declared \{[^}]*latency_samples: (\d+)", src)
        self.assertIsNotNone(m, "lib.rs 里没找到 GTCRN 声明的延迟；正本改了，本脚本要跟着看")
        self.assertEqual(off.DECLARED_LATENCY, int(m.group(1)))
        # 前置补零（窗长 − 帧移）必须就是模型声明的算法延迟；两个正本打出不一致的数就停下核对
        self.assertEqual(off.PAD, off.DECLARED_LATENCY)


class RepoPaths(unittest.TestCase):
    """写进仓的路径不许带家目录：仓是公开的（`model_info` / `setup` 都过 `rel`）。"""

    def test_inside_worktree_is_relative_to_root(self):
        p = off.ROOT / "docs/experiments/exp025-gtcrn-train/eval/parity.py"
        self.assertEqual(off.rel(p), "docs/experiments/exp025-gtcrn-train/eval/parity.py")

    def test_inside_main_root_is_prefixed_with_placeholder(self):
        # 实物（ONNX · 音频）在主目录的 _tmp 下，不在本工作副本里：写 `<主目录>/…`，不写绝对路径
        main = off._ext.main_root()
        self.assertEqual(off.rel(main / "_tmp" / "exp025-x" / "gtcrn-official.onnx"),
                         "<主目录>/_tmp/exp025-x/gtcrn-official.onnx")

    def test_outside_both_is_refused(self):
        with tempfile.TemporaryDirectory() as t:
            p = Path(t).resolve() / "gtcrn-official.onnx"
            with self.assertRaises(SystemExit) as e:
                off.rel(p)
            self.assertIn(str(p), str(e.exception))

    def test_run_metadata_has_no_absolute_path(self):
        main = off._ext.main_root()
        onnx = main / "_tmp" / "exp025-x" / "gtcrn-official.onnx"
        for block in (off.model_info(onnx, "0" * 64), off.setup(onnx, "0" * 64)):
            self.assertNotIn(str(main), json.dumps(block, ensure_ascii=False))
            self.assertNotIn(str(Path.home()), json.dumps(block, ensure_ascii=False))


class ClipList(unittest.TestCase):
    """片段清单跟评测集正本（scenes.json）走，而不是共用框架里写死的旧 build。"""

    def test_clip_list_follows_scenes_json(self):
        self.assertEqual(off.CLIPS, off.suite_clips())
        self.assertTrue((off.ROOT / off.CLIPS).is_file(), off.CLIPS)
        # 换掉框架常量后，`_ext.main` 自己那道「scenes.json 与常量一致」的自检才过得去
        self.assertEqual(off._ext.scenes_clips_path(), off.CLIPS)


# ---- 窗口：第二处表示，与两份正本交叉校验（CLAUDE.md 五.5） ----
# 与正本比窗的容差。1e-7 用在「同一精度路径」上：Rust `coef` 与 torch 的 float64 窗都先算 f64 再转 f32。
WINDOW_TOL = 1e-7
# torch 实际的 f32 窗是另一回事：hann 在 f32 里算 `0.5 − 0.5·cos`，小值处两个 0.5 相减丢精度，
# i=1 处相对误差被放大到 1.2e-4（实测最大绝对差 7.13e-07，2026-11-26）。这是 torch 的算术，不是我们换了公式，
# 所以对照它另给一条 1e-6 的线，并在 torch 的 float64 窗上再核一次 1e-7。
WINDOW_TOL_F32 = 1e-6
GOLDEN = HERE / "fixtures" / "window-sqrt-hann-512.json"


def rust_window_formula():
    """从正本 `src/dsp/src/window.rs` 取 `SqrtHannPeriodic` 的系数表达式（金标文件就是照它生成的）。"""
    src = (off.ROOT / "src/dsp/src/window.rs").read_text()
    m = re.search(r"Window::SqrtHannPeriodic => (\(.*?\.sqrt\(\))", src, re.S)
    if not m:
        raise AssertionError("window.rs 里找不到 SqrtHannPeriodic 的表达式；正本改了，金标要重新生成")
    return m.group(1).strip()


def golden_window():
    """Rust 算出来的 512 个 f32 系数（金标，见 `bless_window`）。"""
    g = json.loads(GOLDEN.read_text())
    bits = np.array([int(x, 16) for x in g["bits_f32_hex"]], dtype=np.uint32)
    return g, bits.view(np.float32)


def window_max_diff(win, want):
    """逐系数最大绝对差；长度不同直接报错（长度也是口径的一部分）。"""
    a, b = np.asarray(win, np.float64), np.asarray(want, np.float64)
    if a.shape != b.shape:
        raise ValueError(f"窗长不一致：{a.shape} vs {b.shape}")
    return float(np.max(np.abs(a - b)))


def window_ok(win, want, tol=WINDOW_TOL):
    """窗是不是与参照同一（正例与负例都走这一条）。"""
    try:
        return window_max_diff(win, want) <= tol
    except ValueError:
        return False


def trainer_losses():
    """`trainer/losses.py`（torch 的窗口正本）。导入会连带 torch 与官方 GTCRN 源码。"""
    t = HERE.parent / "trainer"
    if str(t) not in sys.path:
        sys.path.insert(0, str(t))
    import losses  # noqa: PLC0415 —— 没有 torch 时整条测试跳过，不在导入期炸
    return losses


def torch_window_f64():
    """`losses.window()` 的同一个调用（`hann_window(n).pow(0.5)`），只是 dtype 换成 float64：
    用来把「公式不同」和「torch 的 f32 算术精度」分开（下面的 WINDOW_TOL / WINDOW_TOL_F32）。"""
    import torch
    return torch.hann_window(off.N_FFT, dtype=torch.float64).pow(0.5).numpy()


def bless_window():
    """重新生成金标：把 `window.rs` 的表达式抠出来，用 `rustc` 编一个 10 行程序算 512 个 f32 系数。
    不需要 cargo、不碰工作区的 `target/`（产物都在临时目录）。只在正本改了之后手工跑一次。"""
    expr, n = rust_window_formula(), off.N_FFT
    rs = (f"const PI: f64 = std::f64::consts::PI;\n"
          f"fn coef(i: usize, n: usize) -> f64 {{ {expr} }}\n"
          f"fn main() {{ for i in 0..{n}usize {{ println!(\"{{:08x}}\", (coef(i, {n}) as f32).to_bits()); }} }}\n")
    with tempfile.TemporaryDirectory() as t:
        src, exe = Path(t) / "w.rs", Path(t) / "w"
        src.write_text(rs)
        subprocess.run(["rustc", "-O", str(src), "-o", str(exe)], check=True)
        bits = subprocess.run([str(exe)], check=True, capture_output=True, text=True).stdout.split()
    if len(bits) != n:
        raise SystemExit(f"金标算出 {len(bits)} 个系数，应为 {n}")
    GOLDEN.parent.mkdir(parents=True, exist_ok=True)
    GOLDEN.write_text(json.dumps({"n_fft": n, "rust_expr": expr,
                                  "note": "Rust `src/dsp/src/window.rs::coef(Window::SqrtHannPeriodic)`：先 f64 算、再转 f32。"
                                          "下面是 f32 的位模式（十六进制），比的是位不是字面值",
                                  "command": "<python> test_offline.py --bless-window（要 rustc，不需要 cargo）",
                                  "bits_f32_hex": bits}, ensure_ascii=False, indent=1) + "\n")
    print(f"已重写金标：{GOLDEN}（{n} 个系数）")


class WindowSingleSource(unittest.TestCase):
    """离线窗口不许只跟自己比：与 Rust 正本金标、trainer 的 torch 实现各比一遍，另带一条负例。"""

    def test_formula_text_is_what_the_golden_was_made_from(self):
        g, _ = golden_window()
        self.assertEqual(g["n_fft"], off.N_FFT)
        self.assertEqual(rust_window_formula(), g["rust_expr"],
                         "window.rs 的公式变了：重新生成金标（--bless-window）后再看差在哪")

    def test_window_matches_rust_golden(self):
        _, want = golden_window()
        self.assertTrue(window_ok(off.WIN, want), f"与 Rust 金标的最大差 {window_max_diff(off.WIN, want):.3e}")
        # 同一精度路径（f64 算完再转 f32）应当逐位相同
        np.testing.assert_array_equal(off.WIN.view(np.uint32), want.view(np.uint32))

    def test_window_matches_trainer_torch(self):
        try:
            f32 = trainer_losses().window().detach().cpu().numpy()
            f64 = torch_window_f64()
        except ImportError as e:  # 没装 torch（本机 trainer venv 有）
            self.skipTest(f"没有 torch / 导不进 trainer/losses.py：{e}")
        # 1) 公式同一：同一精度路径（float64 算完再转 f32）≤ 1e-7
        self.assertLessEqual(window_max_diff(off.WIN, f64), WINDOW_TOL, "与 torch（float64）的算法不同")
        # 2) trainer 实际用的是 f32 窗：只能到 1e-6（理由见 WINDOW_TOL_F32）
        self.assertLessEqual(window_max_diff(off.WIN, f32), WINDOW_TOL_F32,
                             "与 torch（float32，trainer 实际用的）差得比 f32 精度还多")
        # 3) f32 与 f64 的窗差别就落在 1e-7..1e-6 之间：说明第 2 条不是随手放大的
        self.assertGreater(window_max_diff(f32, f64), WINDOW_TOL)

    def test_perturbed_window_is_caught(self):
        # 负例：改一个系数 1e-6（> WINDOW_TOL）→ 与金标 / torch float64 的比对照样要红
        _, want = golden_window()
        bad = off.WIN.astype(np.float64).copy()
        bad[7] += 1e-6
        bad = bad.astype(np.float32)
        self.assertFalse(window_ok(bad, want))
        self.assertTrue(window_ok(off.WIN, want))
        try:
            f64 = torch_window_f64()
        except ImportError:
            return
        self.assertFalse(window_ok(bad, f64))
        self.assertTrue(window_ok(off.WIN, f64))


class RoundTrip(unittest.TestCase):
    def test_identity_model_returns_the_input_same_length(self):
        for n in (1600, 4096, 16001):
            x = noise(n)
            y = off.process_with(lambda spec: spec, x)
            self.assertEqual(y.shape, (n,))
            self.assertEqual(y.dtype, np.float32)
            np.testing.assert_allclose(y, x, atol=1e-5, rtol=0)

    def test_identity_model_is_exact_where_builtin_leaves_zero_padding(self):
        # 与内置流式的口径一致：补零的那些样本不属于输出（取回的就是原输入，不多不少）
        x = np.zeros(1024, np.float32)
        x[300:700] = noise(400)
        y = off.process_with(lambda spec: spec, x)
        np.testing.assert_allclose(y, x, atol=1e-6, rtol=0)


def causal_stream_identity(x, win, n_fft, hop):
    """照 Rust `StreamingStft`（`src/dsp/src/stft.rs`）：缓冲初始全零 · 每次喂一个 hop · 分析 [上一块, 本块] ·
    合成同窗重叠相加 · 吐前 hop。频谱原样送回（恒等模型），输出比输入晚 n_fft − hop 个样本。"""
    buf, ola, out = np.zeros(n_fft), np.zeros(n_fft), []
    for i in range(0, len(x), hop):
        blk = np.zeros(hop)
        blk[: len(x[i:i + hop])] = x[i:i + hop]
        buf = np.concatenate([buf[hop:], blk])
        ola = ola + np.fft.irfft(np.fft.rfft(buf * win), n=n_fft) * win
        out.append(ola[:hop].copy())
        ola = np.concatenate([ola[hop:], np.zeros(hop)])
    return np.concatenate(out)


class SameAsBuiltinFraming(unittest.TestCase):
    def test_offline_equals_causal_streaming_after_latency(self):
        # 内置那一侧是「输入缓冲初始全零 + 重叠相加 + 输出晚 窗长−帧移」；离线这条路必须逐样本同一，
        # 否则甲路的对齐 / 口径与内置路不一致（`_ext.probe` 的互相关也会报滞后 ≠ 0）。
        n = off.HOP * 20
        x = noise(n)
        raw = causal_stream_identity(np.concatenate([x, np.zeros(off.N_FFT, np.float32)]),
                                     off.WIN, off.N_FFT, off.HOP)
        y = off.process_with(lambda spec: spec, x)
        np.testing.assert_allclose(y, raw[off.PAD:off.PAD + n], atol=1e-6, rtol=0)


class FakeInput:
    def __init__(self, name, shape):
        self.name, self.shape = name, shape


class FakeSession:
    """记下 run() 收到的喂入张量，再把 spec 原样吐回（当恒等模型）。"""

    def __init__(self, cond_dim=None):
        self.inputs = [FakeInput("spec", [1, off.BINS, None, 2])]
        if cond_dim is not None:
            self.inputs.append(FakeInput("cond", [1, cond_dim]))
        self.feeds = []

    def get_inputs(self):
        return self.inputs

    def run(self, names, feed):
        self.feeds.append((names, dict(feed)))
        return [feed["spec"]]


class CondInput(unittest.TestCase):
    def test_zeros_cond_is_fed_with_the_model_dim(self):
        s = FakeSession(cond_dim=8)
        off.process_with(off.Offline(s), noise(1600))
        self.assertEqual(len(s.feeds), 1)
        names, feed = s.feeds[0]
        self.assertIsNone(names)
        self.assertEqual(feed["spec"].shape[:2], (1, off.BINS))
        self.assertEqual(feed["spec"].dtype, np.dtype(np.float32))
        self.assertEqual(feed["cond"].shape, (1, 8))
        self.assertEqual(feed["cond"].dtype, np.dtype(np.float32))
        np.testing.assert_array_equal(feed["cond"], np.zeros((1, 8), np.float32))

    def test_no_cond_input_no_cond_fed(self):
        s = FakeSession()
        off.process_with(off.Offline(s), noise(1600))
        self.assertNotIn("cond", s.feeds[0][1])

    def test_wrong_output_shape_is_rejected(self):
        class Bad(FakeSession):
            def run(self, names, feed):
                return [feed["spec"][..., :1]]

        with self.assertRaises(ValueError):
            off.process_with(off.Offline(Bad()), noise(1600))


def row(clip, values, system="gtcrn"):
    return {"clip": clip, "scene": "cafe", "tier": "t", "device": "d", "system": system,
            "scores": [{"v": v, "value": val, "verdict": "pass", "threshold": 0.0, "veto": False, "extra": {}}
                       for v, val in values.items()]}


class Parity(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.dir = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def write(self, name, rows):
        p = self.dir / name
        p.write_text("\n".join(json.dumps(r, ensure_ascii=False) for r in rows) + "\n")
        return str(p)

    def run_parity(self, *argv):
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            code = parity.main(list(argv))
        return code, buf.getvalue()

    def test_equal_files_pass(self):
        a = self.write("a.jsonl", [row("c0", {"V-NAN": 0.0, "V-ESTOI": 0.06})])
        b = self.write("b.jsonl", [row("c0", {"V-NAN": 0.0, "V-ESTOI": 0.06}, system="gtcrn-offline")])
        code, out = self.run_parity("--a", a, "--b", b)
        self.assertEqual(code, 0, out)
        self.assertIn("PASS", out)
        self.assertIn("V-ESTOI", out)

    def test_shift_beyond_tolerance_fails(self):
        a = self.write("a.jsonl", [row("c0", {"V-ESTOI": 0.06})])
        b = self.write("b.jsonl", [row("c0", {"V-ESTOI": 0.06 + 10 * parity.TOL}, system="off")])
        code, out = self.run_parity("--a", a, "--b", b)
        self.assertNotEqual(code, 0, out)
        self.assertIn("FAIL", out)
        self.assertIn("c0", out)

    def test_shift_within_tolerance_passes(self):
        a = self.write("a.jsonl", [row("c0", {"V-ESTOI": 0.06})])
        b = self.write("b.jsonl", [row("c0", {"V-ESTOI": 0.06 + parity.TOL / 2}, system="off")])
        code, out = self.run_parity("--a", a, "--b", b)
        self.assertEqual(code, 0, out)

    def test_skipped_items_are_ignored_and_listed(self):
        a = self.write("a.jsonl", [row("c0", {"V-NAN": 0.0, "V-LAT": 41.4, "V-FRAME": 0.3, "V-EAT-ASR": None})])
        b = self.write("b.jsonl", [row("c0", {"V-NAN": 0.0, "V-LAT": 999.0, "V-FRAME": None, "V-EAT-ASR": None},
                                       system="off")])
        code, out = self.run_parity("--a", a, "--b", b)
        self.assertEqual(code, 0, out)
        for item in parity.SKIP:
            self.assertIn(item, out)
        self.assertIn("跳过", out)
        self.assertNotIn("V-LAT      999", out)

    def test_one_sided_missing_value_fails(self):
        # 一边是 null、另一边有数 → 不可比，必须 FAIL（codex 评审第 1 轮 P1 #5：不能当「跳过」）
        a = self.write("a.jsonl", [row("c0", {"V-NAN": 0.0, "V-ESTOI": 0.06})])
        b = self.write("b.jsonl", [row("c0", {"V-NAN": 0.0, "V-ESTOI": None}, system="off")])
        code, out = self.run_parity("--a", a, "--b", b)
        self.assertNotEqual(code, 0, out)
        self.assertIn("FAIL", out)
        self.assertIn("V-ESTOI", out)

    def test_item_missing_in_one_file_fails(self):
        # 项在一份文件里根本没有（另一份有数）→ 同样 FAIL
        a = self.write("a.jsonl", [row("c0", {"V-NAN": 0.0, "V-ESTOI": 0.06})])
        b = self.write("b.jsonl", [row("c0", {"V-NAN": 0.0}, system="off")])
        code, out = self.run_parity("--a", a, "--b", b)
        self.assertNotEqual(code, 0, out)
        self.assertIn("V-ESTOI", out)

    def test_both_null_is_unverified_not_pass_evidence(self):
        # 两边都是 null：列成「未验证」，不算 PASS 证据，且总结里要说清几项未验证
        a = self.write("a.jsonl", [row("c0", {"V-NAN": 0.0, "V-EAT-ASR": None})])
        b = self.write("b.jsonl", [row("c0", {"V-NAN": 0.0, "V-EAT-ASR": None}, system="off")])
        code, out = self.run_parity("--a", a, "--b", b)
        self.assertEqual(code, 0, out)
        self.assertIn("未验证", out)
        self.assertIn("1 项未验证", out)
        self.assertRegex(out, r"V-EAT-ASR\s+0\s+1\s+[\d.e+-]+\s+未验证（两边都没有值）")

    def test_clip_in_only_one_file_fails(self):
        a = self.write("a.jsonl", [row("c0", {"V-NAN": 0.0}), row("c1", {"V-NAN": 0.0})])
        b = self.write("b.jsonl", [row("c0", {"V-NAN": 0.0}, system="off")])
        code, out = self.run_parity("--a", a, "--b", b)
        self.assertNotEqual(code, 0, out)
        self.assertIn("c1", out)

    def test_two_systems_in_one_file(self):
        rows = [row("c0", {"V-NAN": 0.0}, system="gtcrn"),
                row("c0", {"V-NAN": 0.0}, system="gtcrn-offline")]
        p = self.write("both.jsonl", rows)
        code, out = self.run_parity("--a", p, "--b", p, "--system-a", "gtcrn", "--system-b", "gtcrn-offline")
        self.assertEqual(code, 0, out)
        with self.assertRaises(SystemExit) as e:
            self.run_parity("--a", p, "--b", p)
        self.assertIn("gtcrn-offline", str(e.exception))

    def test_nothing_compared_is_not_a_pass(self):
        a = self.write("a.jsonl", [row("c0", {"V-ESTOI": None})])
        b = self.write("b.jsonl", [row("c0", {"V-ESTOI": None}, system="off")])
        code, out = self.run_parity("--a", a, "--b", b)
        self.assertNotEqual(code, 0, out)


if __name__ == "__main__":
    if "--bless-window" in sys.argv:  # 手工重生成窗口金标（要 rustc）
        sys.argv.remove("--bless-window")
        bless_window()
    unittest.main()
