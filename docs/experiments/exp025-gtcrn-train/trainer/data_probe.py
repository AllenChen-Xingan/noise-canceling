"""过拟合慢的数据侧诊断（大总管 2026-11-25 ④–⑥），由 `overfit_probe.run_probe` 调用。只诊断，不改判据、不改配方正本。

- 对齐：输入（noisy）与目标（clean）的互相关峰值时延，要等于 0
- 目标本身：近乎静音、削波、电平是否在配方范围，另把输入与目标写成 16 位 wav 给人听
- 逐个关配方旋钮：reverb · eq · clip 关掉，n_noises 改成 1 段，再加四个全关；每组同样 4 段 150 步
  改过的配方只写进报告目录的临时文件，build 登记不认它们，所以用 check_build=False 读；基线仍用登记过的正本
  注意：关旋钮会改变 mix.plan 里随机数的用法，同一段的人声、噪声、RIR 来源不变（在 mix.plan 之前抽），片段起点等可能不同
"""
from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path

import numpy as np
import soundfile as sf
import torch

import losses
import oracle

MAX_LAG = 4000      # 0.25 s
NEAR_SILENT_DBFS = -60.0
CLIP_ABS = 0.999
ABLATIONS = ("baseline", "reverb_off", "eq_off", "clip_off", "n_noises_1", "all4_off")
_OFF = {"reverb_off": ("reverb",), "eq_off": ("eq",), "clip_off": ("clip",), "n_noises_1": ("n_noises",),
        "all4_off": ("reverb", "eq", "clip", "n_noises")}


def alignment(noisy: torch.Tensor, clean: torch.Tensor, max_lag: int = MAX_LAG) -> dict:
    """每段的互相关峰值时延（样本；正 = 输入比目标晚）与该处的归一化相关系数。"""
    segs = []
    for x, y in zip(noisy.double(), clean.double()):
        n = x.numel() + y.numel()
        r = torch.fft.irfft(torch.fft.rfft(x, n) * torch.fft.rfft(y, n).conj(), n)
        lags = torch.cat([r[: max_lag + 1], r[-max_lag:]])
        idx = torch.cat([torch.arange(max_lag + 1), torch.arange(-max_lag, 0)])
        k = int(lags.abs().argmax())
        corr = float(lags[k] / (x.norm() * y.norm()).clamp(min=1e-20))
        segs.append({"lag_samples": int(idx[k]), "lag_ms": float(idx[k]) / 16.0, "corr": corr})
    return {"segments": segs, "all_zero_lag": all(s["lag_samples"] == 0 for s in segs), "max_lag": max_lag}


def _dbfs(x: torch.Tensor) -> float:
    return float(10 * torch.log10(x.double().pow(2).mean().clamp(min=1e-20)))


def target_stats(clean: torch.Tensor, noisy: torch.Tensor, metas, recipe) -> dict:
    lv = (recipe or {}).get("knobs", {}).get("level")
    segs = []
    for i, (c, x) in enumerate(zip(clean, noisy)):
        m = metas[i] if metas else {}
        want = m.get("level_dbfs")
        rng_ok = None if (lv is None or want is None) else bool(lv["lo_dbfs"] <= want <= lv["hi_dbfs"])
        segs.append({"clean_dbfs": _dbfs(c), "noisy_dbfs": _dbfs(x), "near_silent": _dbfs(c) < NEAR_SILENT_DBFS,
                     "clean_peak": float(c.abs().max()), "clipped_samples": int((c.abs() >= CLIP_ABS).sum()),
                     "level_dbfs_meta": want, "level_in_recipe_range": rng_ok,
                     **{k: m.get(k) for k in ("snr_db", "n_noises", "reverb", "clip_thr", "scene", "speech", "rir")}})
    return {"segments": segs, "rule": f"近乎静音 = 整段 < {NEAR_SILENT_DBFS:g} dBFS；削波 = |x| ≥ {CLIP_ABS}"}


def write_wavs(out_dir: Path, noisy: torch.Tensor, clean: torch.Tensor, sr: int = 16000) -> list[dict]:
    out_dir = Path(out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    files = []
    for i in range(noisy.shape[0]):
        for tag, x in (("noisy", noisy[i]), ("clean", clean[i])):
            p = out_dir / f"seg{i}-{tag}.wav"
            sf.write(p, np.clip(x.detach().cpu().numpy(), -1, 1), sr, subtype="PCM_16")
            files.append({"path": str(p), "sha256": hashlib.sha256(p.read_bytes()).hexdigest()})
    return files


def ablation_recipes(recipe: dict) -> dict:
    out = {}
    for name in ABLATIONS:
        r = copy.deepcopy(recipe)
        for k in _OFF.get(name, ()):
            r["knobs"][k]["on"] = False
        out[name] = r
    return out


def run_ablations(cfg, device, recipe: dict, make_batch, steps: int, curve_fn, on_entry=lambda n, r: None) -> dict:
    """每组：make_batch(配方, 组名) → 4 段 → 这组自己的 oracle → curve_fn 跑 steps 步（lr 1e-3）。"""
    crit = losses.TrainLoss(losses.LossCfg()).to(device)
    out = {}
    for name, r in ablation_recipes(recipe).items():
        b = make_batch(r, name)
        o = oracle.oracle_loss(crit, b, device)
        c = curve_fn(cfg, b, device, 1e-3, steps, o["loss"])
        last = c["marks"][-1]
        out[name] = {"steps": steps, "oracle_loss": o["loss"], "oracle_si_snr_db": o["si_snr_db"],
                     "loss0": c["loss0"], "loss": last["loss"], "drop_frac_of_oracle_span": last["drop_frac_of_oracle_span"],
                     "si_snr_db0": c["si_snr_db0"], "si_snr_gain_db": last["si_snr_gain_db"],
                     "off": list(_OFF.get(name, ()))}
        on_entry(name, out[name])
    return out


def loader_batch_maker(mod, cfg, work_dir: Path, n: int):
    """返回 make_batch：把配方写成 work_dir/<组名>.json，用 loader 的 DevSet 渲染前 n 段（与开发份第 0 批前 n 段同下标）。"""
    from data import collate
    d = cfg.data

    def make(recipe: dict, name: str):
        if name == "baseline":
            path, check = d.recipe, True  # 基线用登记过的正本
        else:
            Path(work_dir).mkdir(parents=True, exist_ok=True)
            path, check = Path(work_dir) / f"{name}.json", False
            path.write_text(json.dumps(recipe, ensure_ascii=False, indent=1))
        ds = mod.DevSet(d.shard_root, str(path), n=n, seed=d.dev_seed, seg_s=d.seg_s, check_build=check)
        exs = [ds[j] for j in range(n)]
        b = collate(exs)
        b["metas"] = [e["meta"] for e in exs]
        return b
    return make
