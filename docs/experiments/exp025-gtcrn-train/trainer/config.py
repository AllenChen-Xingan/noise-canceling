"""训练配置：一份 YAML 管全部超参。**这里的 dataclass 是格式正本**；`configs/*.yaml` 是它的实例，
测试 `test_config.py` 核对 `configs/default.yaml` 恰好覆盖全部字段（多一个、少一个都红）。

多出的键一律报错（防拼错键名静默失效，同 `src/eval/src/files.rs` 的 deny_unknown_fields）。
默认值依据：027 第 3.1 节（超参）· 034 papers 第二节（L2-SP · 老师 · EMA 默认关）· D96（3090 $0.50/小时）· 014 第五节（$20 上限）。
"""
from __future__ import annotations

import dataclasses
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Union

import yaml

Num = Union[float, str]  # 损失权重可写 "auto"：开跑前在开发份上标定（027 3.2 权重第 1 步）


@dataclass
class RunCfg:
    out_dir: str = "~/vdn-data/runs"  # 检查点与日志放仓外（D88 · CLAUDE.md 铁律 8）
    name: str = "exp025"
    seed: int = 0


@dataclass
class ModelCfg:
    variant: str = "B"  # A 原结构 · B 加时间注意力头 + 条件输入口子
    k_frames: int = 32  # 注意力回看帧数（028：32 帧 ≈ 0.5 s）
    heads: int = 2
    cond_dim: int = 8  # 条件输入维度（028 场景向量 8 维）；本次喂全零（D98）
    attn: bool = True
    film: bool = True
    init: str = "official"  # official = 官方 DNS3 起点；random = 随机初始化（只给冒烟对照用）
    bn: str = "frozen"  # frozen = 批归一化用起点统计量、不更新（训练与推理一致）；train = 照官方训练模式（扫参选项）


@dataclass
class DataCfg:
    kind: str = "loader"  # loader = traindata 的 data/loader.py；synthetic = 仓内合成小样（测试 · 冒烟）
    shard_root: str = ""
    recipe: str = ""
    seg_s: float = 4.0  # 027：4 s 片段
    batch: int = 32
    num_workers: int = 8
    dev_n: int = 2000
    dev_seed: int = 12345
    dev_batches: int = 20  # 每次开发评估用几批
    calib_batches: int = 200  # 权重 "auto" 标定用几批开发数据（027）
    timeout_s: float = 600.0  # DataLoader 取一批的最长等待；worker 卡住时停下而不是一直计费


@dataclass
class OptimCfg:
    lr: float = 1e-4
    warmup_steps: int = 500
    grad_clip: float = 3.0
    betas: list = field(default_factory=lambda: [0.9, 0.999])


@dataclass
class LossCfgFile:
    w_lvl: Num = "auto"
    w_on: Num = "auto"
    lvl_margin_db: float = 2.0
    sp: bool = False
    w_sp: float = 1e-4
    kd: bool = False
    w_kd: float = 0.1
    level_norm: bool = False
    level_floor: float = 1e-3
    on_skip_leading: bool = False  # 字头项跳过 a = 0 的区间（扫参选项，默认与 V-ONSET 同口径）


@dataclass
class EmaCfg:
    enabled: bool = False  # 034：SAFE 表 6 去掉 EMA 反而更高，只作扫参选项
    decay: float = 0.999


@dataclass
class ScheduleCfg:
    max_steps: int = 5000
    ckpt_every: int = 500
    eval_every: int = 500
    log_every: int = 20
    keep_last: int = 3


@dataclass
class LimitsCfg:
    max_hours: float = 6.0
    max_usd: float = 20.0
    usd_per_hour: float = 0.50


@dataclass
class Config:
    run: RunCfg = field(default_factory=RunCfg)
    model: ModelCfg = field(default_factory=ModelCfg)
    data: DataCfg = field(default_factory=DataCfg)
    optim: OptimCfg = field(default_factory=OptimCfg)
    loss: LossCfgFile = field(default_factory=LossCfgFile)
    ema: EmaCfg = field(default_factory=EmaCfg)
    schedule: ScheduleCfg = field(default_factory=ScheduleCfg)
    limits: LimitsCfg = field(default_factory=LimitsCfg)
    device: str = "auto"  # auto = cuda > mps（带 exp018 绕法）> cpu


def _build(cls, d: dict, where: str):
    if not isinstance(d, dict):
        raise ValueError(f"{where} 应是映射")
    names = {f.name: f for f in dataclasses.fields(cls)}
    extra = set(d) - set(names)
    if extra:
        raise ValueError(f"{where} 有未知键：{sorted(extra)}")
    kw = {}
    for k, v in d.items():
        sub = names[k].default_factory if names[k].default_factory is not dataclasses.MISSING else None
        if sub is not None and dataclasses.is_dataclass(sub):
            kw[k] = _build(sub, v, f"{where}.{k}")
        else:
            kw[k] = v
    return cls(**kw)


def from_dict(d: dict) -> Config:
    cfg = _build(Config, d or {}, "config")
    validate(cfg)
    return cfg


def load(path: str | Path, overrides: dict | None = None) -> Config:
    d = yaml.safe_load(Path(path).read_text()) or {}
    for dotted, v in (overrides or {}).items():
        cur = d
        *head, last = dotted.split(".")
        for h in head:
            cur = cur.setdefault(h, {})
        cur[last] = v
    return from_dict(d)


def to_dict(cfg: Config) -> dict[str, Any]:
    return dataclasses.asdict(cfg)


def validate(cfg: Config) -> None:
    if cfg.model.variant not in ("A", "B"):
        raise ValueError("model.variant 只能是 A 或 B")
    if cfg.model.bn not in ("frozen", "train"):
        raise ValueError("model.bn 只能是 frozen 或 train")
    if cfg.model.init not in ("official", "random"):
        raise ValueError("model.init 只能是 official 或 random")
    if cfg.data.kind not in ("loader", "synthetic"):
        raise ValueError("data.kind 只能是 loader 或 synthetic")
    for k in ("w_lvl", "w_on"):
        v = getattr(cfg.loss, k)
        if not (v == "auto" or isinstance(v, (int, float)) and v >= 0):
            raise ValueError(f"loss.{k} 应是非负数或 \"auto\"")
    if cfg.limits.max_hours <= 0 or cfg.limits.max_usd <= 0:
        raise ValueError("limits 必须为正（不许不设上限）")
    if cfg.schedule.ckpt_every <= 0:
        raise ValueError("schedule.ckpt_every 必须为正")
