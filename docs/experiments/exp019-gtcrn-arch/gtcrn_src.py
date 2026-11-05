"""找到官方 GTCRN 源码（只读导入，不抄）并加载流式模型。

官方代码在主目录的 `resources/gtcrn/`（不入库）。worktree 里没有它，所以按
`git rev-parse --git-common-dir` 找主仓根；也可用环境变量 `VDN_MAIN_ROOT` 指定。
"""
import os
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent


def main_root() -> Path:
    env = os.environ.get("VDN_MAIN_ROOT")
    if env:
        return Path(env)
    out = subprocess.run(
        ["git", "-C", str(HERE), "rev-parse", "--git-common-dir"],
        check=True, capture_output=True, text=True,
    ).stdout.strip()
    common = Path(out)
    if not common.is_absolute():
        common = (HERE / common).resolve()
    return common.parent


def stream_dir() -> Path:
    d = main_root() / "resources" / "gtcrn" / "stream"
    if not (d / "gtcrn_stream.py").exists():
        raise FileNotFoundError(f"没找到官方流式 GTCRN：{d}")
    return d


def import_official():
    """返回 (GTCRN, StreamGTCRN, convert_to_stream)。"""
    d = str(stream_dir())
    if d not in sys.path:
        sys.path.insert(0, d)
    from gtcrn import GTCRN  # noqa: E402  官方离线版（stream/ 目录里那份）
    from gtcrn_stream import StreamGTCRN  # noqa: E402
    from modules.convert import convert_to_stream  # noqa: E402
    return GTCRN, StreamGTCRN, convert_to_stream


def load_stream_model(weights: str = "model_trained_on_dns3.tar"):
    """加载产品用的那份权重（`gtcrn_simple.onnx` 就是它导出的），转成流式版。"""
    import torch
    GTCRN, StreamGTCRN, convert_to_stream = import_official()
    off = GTCRN().eval()
    ckpt = torch.load(stream_dir() / "onnx_models" / weights, map_location="cpu")
    off.load_state_dict(ckpt["model"])
    st = StreamGTCRN().eval()
    convert_to_stream(st, off)
    return off, st
