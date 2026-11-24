"""导出 ONNX 并与 PyTorch 对拍（交代第 5 条）。

    python export.py --ckpt <run>/ckpt/step-0005000.pt --out model.onnx [--ema]
    python export.py --official --out gtcrn_a.onnx          # 起点权重（A），用来核导出这一步本身

导出的是**整段版**：输入 `spec` (1, 257, T, 2)，T 可变；B 另有输入 `cond` (1, cond_dim)。
导出前一律在 CPU 上重建**原模块**（不带 MPS 绕法，027 第七节局限 ②）再装权重。
流式版（每帧一调、带缓存，产品 `gtcrn_simple.onnx` 那种）不在本文件：A 可走官方 `convert_to_stream`；
B 的注意力要在 Rust 引擎里另写（014 第六节：归 backlog）。
对拍：两种长度的随机输入，ONNX Runtime 与 PyTorch 输出最大差 ≤ tol（默认 1e-4），超了返回非零。
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import numpy as np
import torch

import models
from official import load_start_state


def model_from_ckpt(path: Path | None, use_ema: bool = False) -> tuple[torch.nn.Module, dict]:
    if path is None:
        m = models.build("A")
        m.load_base(load_start_state())
        return m.eval(), {"variant": "A", "source": "official"}
    st = torch.load(path, map_location="cpu", weights_only=False)
    mc = st["config"]["model"]
    kw = {} if mc["variant"] == "A" else {k: mc[k] for k in ("k_frames", "heads", "cond_dim", "attn", "film")}
    m = models.build(mc["variant"], **kw)
    sd = st["ema"] if use_ema else st["model"]
    if sd is None:
        raise ValueError("这份检查点没有 EMA 权重")
    m.load_state_dict(sd)
    return m.eval(), {"variant": mc["variant"], "source": str(path), "step": st["step"], "ema": use_ema}


def _inputs(m, t: int, seed: int):
    g = torch.Generator().manual_seed(seed)
    spec = torch.randn(1, 257, t, 2, generator=g) * 0.1
    film = getattr(m, "film", None)
    cond = None if film is None else torch.randn(1, film.d, generator=g)
    return spec, cond


def export(m: torch.nn.Module, out: Path) -> list[str]:
    spec, cond = _inputs(m, 63, 0)
    names = ["spec"] + (["cond"] if cond is not None else [])
    args = (spec,) if cond is None else (spec, cond)
    torch.onnx.export(m, args, str(out), input_names=names, output_names=["spec_enh"],
                      dynamic_axes={"spec": {2: "T"}, "spec_enh": {2: "T"}}, opset_version=17, dynamo=False)
    return names


def compare(m: torch.nn.Module, onnx_path: Path, lengths=(63, 250), tol: float = 1e-4) -> dict:
    """ONNX Runtime 对 PyTorch（float32）的最大差。判据：≤ max(tol, 4 × PyTorch 自身 float32 对 float64 的差)。
    后一项是这组权重下浮点舍入本来就有的误差（训后权重大时下游会放大它；自审实测注意力层单独两者同为约 1e-5）。"""
    import copy

    import onnxruntime as ort
    sess = ort.InferenceSession(str(onnx_path), providers=["CPUExecutionProvider"])
    m64 = copy.deepcopy(m).double()
    rows = []
    for i, t in enumerate(lengths):
        spec, cond = _inputs(m, t, 100 + i)
        feed = {"spec": spec.numpy()}
        if cond is not None:
            feed["cond"] = cond.numpy()
        with torch.no_grad():
            ref = m(spec, cond).numpy()
            old = torch.get_default_dtype()
            torch.set_default_dtype(torch.float64)  # 官方 GRNN 在 h=None 时按默认 dtype 新建隐状态
            try:
                ref64 = m64(spec.double(), None if cond is None else cond.double()).numpy()
            finally:
                torch.set_default_dtype(old)
        got = sess.run(None, feed)[0]
        f32_err = float(np.abs(ref - ref64).max())
        d = float(np.abs(ref - got).max())
        lim = max(tol, 4 * f32_err)
        rows.append({"T": t, "max_abs_diff": d, "torch_f32_vs_f64": f32_err, "limit": lim, "pass": d <= lim,
                     "ref_rms": float(np.sqrt((ref ** 2).mean()))})
    return {"pass": all(r["pass"] for r in rows), "tol": tol, "cases": rows}


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description="exp025 导出 ONNX 并对拍")
    src = ap.add_mutually_exclusive_group(required=True)
    src.add_argument("--ckpt", type=Path)
    src.add_argument("--official", action="store_true")
    ap.add_argument("--ema", action="store_true")
    ap.add_argument("--out", type=Path, required=True)
    ap.add_argument("--tol", type=float, default=1e-4)
    a = ap.parse_args(argv)
    m, meta = model_from_ckpt(None if a.official else a.ckpt, a.ema)
    meta["inputs"] = export(m, a.out)
    meta["parity"] = compare(m, a.out, tol=a.tol)
    text = json.dumps(meta, ensure_ascii=False, indent=2)
    a.out.with_suffix(".json").write_text(text)  # torch 导出器会往标准输出打整张图，机器读这份
    print(text)
    return 0 if meta["parity"]["pass"] else 1


if __name__ == "__main__":
    sys.exit(main())
