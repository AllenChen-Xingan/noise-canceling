"""找到官方 GTCRN 源码与检查点（只读导入，不抄进仓）。

官方代码在主目录 `resources/gtcrn/`（不入库）。主目录定位复用 exp019 `gtcrn_src.main_root`（正本），
也可用环境变量 `VDN_GTCRN_DIR` 直接指到 `resources/gtcrn`（云上用）。

起点检查点：`stream/onnx_models/model_trained_on_dns3.tar`——产品里 `gtcrn_simple.onnx` 由它导出（exp019）。
注意 `checkpoints/` 下同名文件哈希不同（2026-11-16 实测），不用它。
"""
from __future__ import annotations

import hashlib
import importlib.util
import os
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
EXP019 = HERE.parents[1] / "exp019-gtcrn-arch"

# 起点检查点的 sha256（2026-11-16 实测）；换了文件就红，防止拿错起点
START_CKPT_REL = "stream/onnx_models/model_trained_on_dns3.tar"
START_CKPT_SHA256 = "250fa2820ea9947704a62dda8e642ba773a067719c5e4294ad95aeab23f06442"


def _load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


def gtcrn_dir() -> Path:
    env = os.environ.get("VDN_GTCRN_DIR")
    if env:
        d = Path(env)
    else:
        src = _load("exp019_gtcrn_src", EXP019 / "gtcrn_src.py")
        d = src.main_root() / "resources" / "gtcrn"
    if not (d / "gtcrn.py").exists():
        raise FileNotFoundError(f"没找到官方 GTCRN：{d}（设 VDN_GTCRN_DIR）")
    return d


def official_module():
    """官方离线版 `gtcrn.py` 模块（含 GTCRN、DPGRNN、ERB 等类）。"""
    if "vdn_official_gtcrn" in sys.modules:
        return sys.modules["vdn_official_gtcrn"]
    return _load("vdn_official_gtcrn", gtcrn_dir() / "gtcrn.py")


def official_loss_module():
    if "vdn_official_loss" in sys.modules:
        return sys.modules["vdn_official_loss"]
    return _load("vdn_official_loss", gtcrn_dir() / "loss.py")


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def start_ckpt_path(check: bool = True) -> Path:
    p = gtcrn_dir() / START_CKPT_REL
    if check and sha256(p) != START_CKPT_SHA256:
        raise ValueError(f"起点检查点哈希不对：{p}")
    return p


def load_start_state():
    """官方起点权重的 state_dict（CPU）。"""
    import torch
    return torch.load(start_ckpt_path(), map_location="cpu")["model"]
