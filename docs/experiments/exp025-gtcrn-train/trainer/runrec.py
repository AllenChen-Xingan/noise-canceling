"""运行记录与检查点：每次运行记 配置 · git 哈希 · 种子 · 数据 build id；每 N 步存（模型 + 优化器 + 步数 + 累计时长）。

目录（仓外，默认 `~/vdn-data/runs/<name>/`）：
- `run.json`：开跑记录；每次续训在 `resumes` 里追加一条；停下时写 `stop`
- `metrics.jsonl`：每 log_every 步一行训练读数，每 eval_every 步一行开发读数
- `ckpt/step-NNNNNNN.pt` + `ckpt/last.txt`（指向最新那份）；只留最近 keep_last 份，另留第 0 步起点
- `heartbeat.json`：每 log_every 步写一次（步数 · 累计计费秒）；被抢占时它比检查点新，续训按它补记花费
写文件一律「写临时文件 → fsync → 改名」；读最新检查点失败时退回上一份（断电后最新那份可能是坏的）。
"""
from __future__ import annotations

import json
import os
import subprocess
import time
from pathlib import Path

import torch

HERE = Path(__file__).resolve().parent

# 续训时允许改的配置（其余一改就拒绝续训：批量、种子、数据、损失等变了，接着训就不是同一个实验）
RESUMABLE_CHANGES = {("schedule", "max_steps"), ("limits", "max_hours"), ("limits", "max_usd"),
                     ("limits", "usd_per_hour"), ("data", "num_workers"), ("schedule", "log_every"),
                     ("device",)}


def git_info() -> dict:
    """提交号与「整个 exp025 目录（含 traindata 的 data/）有没有未提交改动」。"""
    def g(*a):
        r = subprocess.run(["git", "-C", str(HERE), *a], capture_output=True, text=True)
        return r.stdout.strip() if r.returncode == 0 else None
    status = g("status", "--porcelain", "--", str(HERE.parent))
    return {"commit": g("rev-parse", "HEAD"), "dirty": bool(status), "branch": g("rev-parse", "--abbrev-ref", "HEAD")}


def env_info() -> dict:
    """复现要的运行环境：git · torch · CUDA / cuDNN · 显卡型号 · 主机名。"""
    import platform
    e = {"git": git_info(), "torch": torch.__version__, "python": platform.python_version(),
         "host": platform.node(), "cuda": torch.version.cuda, "cudnn": None, "gpu": None}
    if torch.cuda.is_available():
        e["cudnn"] = torch.backends.cudnn.version()
        e["gpu"] = torch.cuda.get_device_name(0)
    return e


def _atomic_write(path: Path, write) -> None:
    tmp = path.with_name(path.name + ".tmp")
    with open(tmp, "wb") as f:
        write(f)
        f.flush()
        os.fsync(f.fileno())
    os.replace(tmp, path)


def config_diff(a: dict, b: dict, prefix=()) -> list[tuple]:
    out = []
    for k in sorted(set(a) | set(b)):
        va, vb = a.get(k), b.get(k)
        if isinstance(va, dict) and isinstance(vb, dict):
            out += config_diff(va, vb, prefix + (k,))
        elif va != vb:
            out.append(prefix + (k,))
    return out


class RunDir:
    def __init__(self, root: Path):
        self.root = root
        self.ckpt_dir = root / "ckpt"
        self.ckpt_dir.mkdir(parents=True, exist_ok=True)

    @property
    def run_json(self) -> Path:
        return self.root / "run.json"

    def read(self) -> dict:
        return json.loads(self.run_json.read_text()) if self.run_json.exists() else {}

    def write(self, rec: dict) -> None:
        text = json.dumps(rec, ensure_ascii=False, indent=2).encode()
        _atomic_write(self.run_json, lambda f: f.write(text))

    def beat(self, step: int, billed_s: float) -> None:
        text = json.dumps({"step": step, "billed_s": billed_s}).encode()
        _atomic_write(self.root / "heartbeat.json", lambda f: f.write(text))

    def heartbeat(self) -> dict:
        p = self.root / "heartbeat.json"
        try:
            return json.loads(p.read_text())
        except (OSError, ValueError):
            return {}

    def log(self, row: dict) -> None:
        with open(self.root / "metrics.jsonl", "a") as f:
            f.write(json.dumps(row, ensure_ascii=False) + "\n")

    def load_latest(self, device) -> tuple[dict | None, Path | None]:
        """最新的可读检查点（按步数从新到旧试）；一份都没有时返回 (None, None)。读坏的那份记进 `bad_ckpt.txt`。"""
        for f in sorted(self.ckpt_dir.glob("step-*.pt"), reverse=True):
            try:
                return torch.load(f, map_location=device, weights_only=False), f
            except Exception as e:  # noqa: BLE001 —— 截断、空文件、反序列化失败都退回上一份
                with open(self.ckpt_dir / "bad_ckpt.txt", "a") as log:
                    log.write(f"{f.name}\t{type(e).__name__}: {e}\n")
        return None, None

    def save(self, state: dict, step: int, keep_last: int) -> Path:
        name = f"step-{step:07d}.pt"
        _atomic_write(self.ckpt_dir / name, lambda f: torch.save(state, f))
        _atomic_write(self.ckpt_dir / "last.txt", lambda f: f.write(name.encode()))
        steps = sorted(self.ckpt_dir.glob("step-*.pt"))
        keep = {self.ckpt_dir / "step-0000000.pt", self.ckpt_dir / name}
        for old in steps[:-keep_last] if keep_last > 0 else []:
            if old not in keep:
                old.unlink()
        return self.ckpt_dir / name


def start_record(cfg_dict: dict, build_id: str, num_workers: int, extra: dict) -> dict:
    return {
        "config": cfg_dict,
        "git": git_info(),
        "seed": cfg_dict["run"]["seed"],
        "data_build_id": build_id,
        "num_workers": num_workers,
        "torch": torch.__version__,
        "started_at": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "resumes": [],
        **extra,
    }


def check_resume(rec: dict, cfg_dict: dict, build_id: str) -> list[str]:
    """续训前核对；返回不许续训的原因（空 = 可以）。"""
    why = []
    stop = rec.get("stop") or {}
    if stop.get("reason") == "nonfinite":  # 训崩过：从同一检查点同一批再跑会确定性地再崩，先诊断（自审 P2）
        why.append(f"上次在第 {stop.get('step')} 步训崩（损失不是有限数），先诊断；要重跑请换 run.name")
    if not build_id or build_id == "unknown":
        why.append("数据 build id 为空，无法核对数据版本")
    for path in config_diff(rec.get("config", {}), cfg_dict):
        if path not in RESUMABLE_CHANGES:
            why.append("配置改了：" + ".".join(path))
    if rec.get("data_build_id") != build_id:
        why.append(f"数据 build 变了：{rec.get('data_build_id')} → {build_id}")
    return why


class EMA:
    """参数与 BatchNorm 统计的指数滑动平均（开关，默认关）。"""

    def __init__(self, model: torch.nn.Module, decay: float):
        self.decay = decay
        self.shadow = {k: v.detach().clone() for k, v in model.state_dict().items()}

    @torch.no_grad()
    def update(self, model: torch.nn.Module) -> None:
        for k, v in model.state_dict().items():
            if v.dtype.is_floating_point:
                self.shadow[k].mul_(self.decay).add_(v.detach(), alpha=1 - self.decay)
            else:
                self.shadow[k].copy_(v)

    def state_dict(self) -> dict:
        return self.shadow

    def load_state_dict(self, sd: dict) -> None:
        self.shadow = {k: v.clone() for k, v in sd.items()}
