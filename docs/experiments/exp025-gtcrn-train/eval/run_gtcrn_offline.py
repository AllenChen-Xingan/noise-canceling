"""exp025 · 整段版 GTCRN ONNX 离线跑 exp013 场景测试集，写「外部输出目录」（`eval-plan.md` 第二节第 2 步的路 甲）。

整段版 ONNX 由 `../trainer/export.py` 导出：输入 `spec` (1, 257, T, 2)、输出 `spec_enh`；模型 B 另有 `cond`
(1, cond_dim)。这里一次前向跑完整段，再 iSTFT 回波形；输出**已对齐**（扣掉声明延迟），交
`vdn-eval scorecard --system external:<目录>` 打分（形状正本 `src/eval/src/system/external.rs`）。

口径（数字只有一处正本，不手抄）：
- 帧长 / 帧移：从 `../trainer/losses.py` 的 `N_FFT, HOP = <帧长>, <帧移>` 读；窗形同它的 `window()`
  （`hann_window(N_FFT).pow(0.5)`，周期 sqrt-Hann）。精度顺序照 Rust `src/dsp/src/window.rs` 的 `coef`：f64 算完再转 f32
- 对齐：内置 GTCRN 的算法延迟 = `latency_samples: 256`（`src/models/src/lib.rs:75`，本脚本只引用、不再编一个数）。
  前面补「窗长 − 帧移」个零 = 内置流式 STFT 的「输入缓冲初始全零」，后面补够尾帧；取回 `[延迟, 延迟 + n)`
  就是已对齐的输出（残差只剩 f32 舍入）
- 声明延迟：交给共用框架的标称值 = 内置 GTCRN 声明的那个数；`_ext.probe` 再按截半实测前视，取两者较大者写进 run.json

用法（项目根下；python 要有 numpy · soundfile · onnxruntime）：
  python3 docs/experiments/exp025-gtcrn-train/eval/run_gtcrn_offline.py \
      --onnx <整段版.onnx> --name gtcrn-offline [--shard k/N] [--run-id ID] [--finalize] [--probe]
其余选项（--shard / --run-id / --finalize / --limit / --probe）都是共用框架 `_ext.py` 的，含义见它的文件头。
片段清单跟 `src/eval/suite/scenes.json` 指的那份（评测集正本）走；`--shard` 按片段轮流分。
参数 · 每个输出文件的 sha256 · 分片耗时写 `builds/<name>-<run-id>/`（进仓）；音频只写主目录 `_tmp/exp016/`。
"""
import argparse
import json
import re
import sys
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "exp016-candidates"))  # 共用框架（正本，导入不复制）
import _ext  # noqa: E402

ROOT = _ext.ROOT
LOSSES = HERE.parent / "trainer/losses.py"


def suite_clips():
    """评测集的片段清单：以 `src/eval/suite/scenes.json` 为准（正本，任务指向哪份就用哪份）。
    共用框架 `_ext.py` 的 `CLIPS` 常量还写着旧 build（20261103230756-17f5），评测集已换成
    `20261124165427-6323`（`eval-plan.md` 第一节）；不改 `_ext.py`（别的实验在用），在这里按正本换掉。"""
    suite = json.loads((ROOT / "src/eval/suite/scenes.json").read_text())
    paths = {t["real"]["clips"] for t in suite["tasks"]}
    if len(paths) != 1:
        raise SystemExit(f"scenes.json 里的片段清单不止一份：{sorted(paths)}")
    return paths.pop()


CLIPS = suite_clips()
_ext.CLIPS = CLIPS  # `_ext.main` 拿它与 scenes.json 比对，不一致就拒绝跑


def frontend():
    """帧长 · 帧移：从 trainer/losses.py 源码读（本文件不重打这两个数）。"""
    src = LOSSES.read_text()
    m = re.search(r"^N_FFT, HOP = (\d+), (\d+)$", src, re.M)
    if not m:
        raise SystemExit(f"{LOSSES} 里找不到 `N_FFT, HOP = <帧长>, <帧移>`；口径正本改了，本脚本要跟着改")
    return int(m.group(1)), int(m.group(2))


N_FFT, HOP = frontend()
BINS = N_FFT // 2 + 1
# 前置补零 = 内置流式 STFT 的输出延迟（窗长 − 帧移）；也等于内置 GTCRN 声明的算法延迟
PAD = N_FFT - HOP
# 内置 GTCRN 声明的算法延迟（16 kHz 样本）。正本：`src/models/src/lib.rs` 里 `ModelKind::Gtcrn` 的
# `Declared { latency_samples: 256 }`（同文件 `latency_ledger` 把它记成「模型算法延迟」）。测试 `test_offline.py`
# 会回读 lib.rs 核对，正本改了这里就红。
DECLARED_LATENCY = 256


def window():
    """周期 sqrt-Hann。= `losses.py` 的 `window()`；先 f64 再转 f32，照 `src/dsp/src/window.rs` 的 `coef`。"""
    i = np.arange(N_FFT, dtype=np.float64)
    return np.sqrt(0.5 - 0.5 * np.cos(2.0 * np.pi * i / N_FFT)).astype(np.float32)


WIN = window()
WIN2 = WIN.astype(np.float64) ** 2


def stft(x):
    """(n,) → (BINS, T, 2)。`torch.stft(center=False)` 同口径：不归一化、末尾凑不满一帧的丢掉。"""
    x = np.asarray(x)
    if x.ndim != 1 or len(x) < N_FFT:
        raise ValueError(f"STFT 要一维且不短于 {N_FFT} 个样本，收到 {x.shape}")
    frames = np.lib.stride_tricks.sliding_window_view(x.astype(np.float64), N_FFT)[::HOP]
    s = np.fft.rfft(frames * WIN, axis=1)
    return np.stack([s.real, s.imag], axis=-1).transpose(1, 0, 2).astype(np.float32)


def istft(spec, length):
    """(257, T, 2) → (length,)。重叠相加后按「窗平方和」归一（`torch.istft` 同口径）：
    对齐后那段区域里窗平方和恒为 1，所以与内置流式的「不归一」逐值相同。"""
    spec = np.asarray(spec)
    if spec.ndim != 3 or spec.shape[0] != BINS or spec.shape[-1] != 2:
        raise ValueError(f"频谱要 (257, T, 2)，收到 {spec.shape}")
    z = spec[..., 0].astype(np.float64) + 1j * spec[..., 1].astype(np.float64)
    frames = np.fft.irfft(z.T, n=N_FFT, axis=1) * WIN
    y = np.zeros(length, np.float64)
    wsum = np.zeros(length, np.float64)
    for k in range(frames.shape[0]):
        a = k * HOP
        y[a:a + N_FFT] += frames[k]
        wsum[a:a + N_FFT] += WIN2
    y = np.where(wsum > 0, y / np.where(wsum > 0, wsum, 1.0), 0.0)
    return y.astype(np.float32)


def pad_align(x):
    """前置一个「窗长 − 帧移」的零（内置流式的初始缓冲），后置够尾帧的零。"""
    return np.concatenate([np.zeros(PAD, np.float32), np.asarray(x, np.float32),
                           np.zeros(N_FFT, np.float32)])


def process_with(model, x):
    """整段处理一段：补零分帧 → `model(spec)` → iSTFT → 去掉补齐 → 等长、已对齐。"""
    x = np.asarray(x, np.float32)
    xp = pad_align(x)
    spec = stft(xp)
    enh = np.asarray(model(spec))
    if enh.shape != spec.shape:
        raise ValueError(f"模型输出形状 {enh.shape} ≠ 输入 {spec.shape}（导出时 dynamic_axes 漏了 T？）")
    y = istft(enh, len(xp))
    return y[PAD:PAD + len(x)]


class Offline:
    """整段版 ONNX 的一次前向。`cond`（模型 B 的条件输入）离线没有外部条件 → 喂全零（D98：全零时那一层恒等）。"""

    def __init__(self, session):
        self.session = session
        self.inputs = {i.name: i for i in session.get_inputs()}
        if "spec" not in self.inputs:
            raise SystemExit(f"ONNX 没有 spec 输入（有：{sorted(self.inputs)}）")

    def __call__(self, spec):
        feed = {"spec": np.ascontiguousarray(spec[None], dtype=np.float32)}
        if "cond" in self.inputs:
            dim = self.inputs["cond"].shape[-1]
            if not isinstance(dim, int):
                raise SystemExit(f"cond 末维不是固定值 {self.inputs['cond'].shape}；本脚本只认 (1, cond_dim)")
            feed["cond"] = np.zeros((1, dim), np.float32)
        return np.asarray(self.session.run(None, feed)[0])[0]


def load_onnx(path):
    try:
        import onnxruntime as ort  # 只在真跑时导入（测试不需要）
    except ImportError as e:  # 没装就早报，不拖到处理第一段才炸
        raise SystemExit(f"没装 onnxruntime（{e}）；用 trainer 的环境：仓库根 `.venv`") from e
    return Offline(ort.InferenceSession(str(path), providers=["CPUExecutionProvider"]))


def rel(path):
    """仓内给相对路径，仓外给绝对路径（写进仓里的清单只留可追到的指针）。"""
    p = Path(path).resolve()
    return str(p.relative_to(ROOT)) if p.is_relative_to(ROOT) else str(p)


def setup(onnx, sha):
    """写进 `builds/<name>-<run-id>/params.json` 的运行参数（运行指纹的一部分，续跑必须一致）。"""
    return {"onnx": rel(onnx), "onnx_sha256": sha,
            "stft": {"n_fft": N_FFT, "hop": HOP, "window": "sqrt-hann periodic", "pad_head": PAD},
            "declared_latency_nominal": DECLARED_LATENCY, "cond": "全零（模型 B 的 cond 输入，D98）"}


def model_info(onnx, sha):
    """写进 run.json 的 `model` 字段（原样进成绩单元数据）。"""
    return {"repo": "GTCRN（官方结构；整段版由 trainer/export.py 导出）", "weights": rel(onnx),
            "weights_sha256": sha, "mode": "整段一次前向 · 因果零填充 · 每段无状态",
            "stft": f"{N_FFT}/{HOP} 周期 sqrt-Hann", "runner": rel(__file__)}


def take_args(argv):
    """本脚本自己的两个选项（`--onnx` · `--name`）；其余原样交给共用框架 `_ext.main`。"""
    ap = argparse.ArgumentParser(prog="run_gtcrn_offline.py", add_help=False,
                                 description="exp025：整段版 GTCRN 离线跑 exp013 场景测试集")
    ap.add_argument("--onnx", required=True, type=Path, help="整段版 ONNX（trainer/export.py 导出）")
    ap.add_argument("--name", required=True, help="外部系统名，[a-z0-9-]+，不能撞内置名")
    a, rest = ap.parse_known_args(argv)
    # 名字规则的正本在 `src/eval/src/system/external.rs::check_name`；入口会再查一遍，这里先报更早
    if not re.fullmatch(r"[a-z0-9-]+", a.name):
        raise SystemExit(f"系统名 {a.name!r} 不合规则 [a-z0-9-]+（正本 external.rs check_name）")
    if not a.onnx.is_file():
        raise SystemExit(f"ONNX 不存在：{a.onnx}")
    return a, rest


def main(argv=None):
    argv = list(sys.argv[1:] if argv is None else argv)
    a, rest = take_args(argv)
    sha = _ext.sha256_file(a.onnx)
    _ext.HERE = HERE  # 清单写本目录的 builds/（框架默认写它自己的 builds/，那是另一个实验的目录）
    model = load_onnx(a.onnx)
    sys.argv = [sys.argv[0]] + rest  # 框架自己再解析一次命令行
    _ext.main(a.name, lambda x: process_with(model, x), DECLARED_LATENCY,
              model_info(a.onnx, sha), lambda: setup(a.onnx, sha))


if __name__ == "__main__":
    main()
