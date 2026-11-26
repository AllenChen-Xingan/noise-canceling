"""训练时的自动保护（plan-h100-smoke.md 第十二节；大总管定 2026-11-25，技术题，人未参与）。阈值正本都在这里。

训练进程自己判，不靠本机醒着；触发时 train.py 记停止原因与一条 guard 记录、能存就存检查点、退出码非 0，
由 cloudrun 的 job-wait 报大总管。每个运行只看自己的读数，一个运行停下不碰别的运行。
- ② 损失猛涨：前 SPIKE_BASE_STEPS 步训练损失（total）中位数作基准 b，最近 SPIKE_WINDOW 步中位数 m；
  m − b > (SPIKE_K − 1)·|b| 就停（loss_spike）。用中位数：单个坏批不误停；用 |b|：total 可能为负，跨零也成立
- ③ 开发份 hybrid 最好的一次另存 best.pt；连续 PATIENCE 次比最好的高 BEST_TOL 以上记警告，不停
- ④ 期望每步秒数在配置 limits.expect_s_step（默认不设 = 关）；第 SPEED_FROM 到 SPEED_TO 步单步墙钟中位数
  > SLOW_FACTOR × 期望，就在第 SPEED_TO 步停（too_slow）。单步墙钟含取数，不含评估与存盘
- ⑤ 中途卡住：主线程 STALL_S 秒没进展，看门狗不管该不该停都强退（watchdog._Beat）
- ①（不是有限数）：唯一的判断是本模块的 `is_finite`，读到坏读数就 `require_finite` 抛 `NonFinite`，
  由 train.py 一处接住（codex guards 第 4 轮 · 改结构：同一个坑连着两轮都是「下一批想停」把 NaN 掩掉了，
  所以不再逐处补判断）
"""
from __future__ import annotations

import math
import statistics

SPIKE_K = 3.0
SPIKE_BASE_STEPS = 20
SPIKE_WINDOW = 50
PATIENCE = 3
BEST_TOL = 1e-3
SLOW_FACTOR = 1.2
SPEED_FROM = 50
SPEED_TO = 500
STALL_S = 900.0

# 几个停止原因同时成立时挑最优先的那个（最高的排最前，大总管定）。新原因必须先进这张表，
# 否则 pick_reason 当场报错（不静默地当成最低优先）。正本只在这里。
STOP_PRIORITY = ("nonfinite", "loss_spike", "stall", "too_slow", "data_timeout", "max_usd",
                 "max_hours", "signal", "data_exhausted", "interrupted", "max_steps")
_RANK = {r: i for i, r in enumerate(STOP_PRIORITY)}


def pick_reason(*reasons) -> str | None:
    """几个停止原因同时成立时挑最高优先的那个（codex guards 第 4 轮 · 改结构）。
    None 不算原因；"signal:SIGTERM" 这种按 "signal" 排名、但原样返回；表里没有的当场 ValueError。"""
    got = [r for r in reasons if r is not None]
    if not got:
        return None

    def rank(r: str) -> int:
        base = str(r).split(":", 1)[0]
        if base not in _RANK:
            raise ValueError(f"未知的停止原因：{r}（新原因先加进 guards.STOP_PRIORITY）")
        return _RANK[base]
    return min(got, key=rank)


class NonFinite(Exception):
    """读到了不是有限数的读数。where 说清在哪读到的，detail 是现场（读数路径 → 那个值，已转成 float）。"""

    def __init__(self, where: str, detail: dict):
        self.where, self.detail = where, detail
        super().__init__(f"{where} 读到不是有限数：{detail}")


def _num(x) -> float | None:
    """把一个读数变成 Python 浮点数；不是读数（bool / None / 字符串 / 多元素张量）返回 None。
    0 维 torch 张量与 numpy 标量都算读数：`.item()` 之后就能 JSON 化，现场记录才写得进文件。"""
    if isinstance(x, bool) or x is None or isinstance(x, str):
        return None
    if isinstance(x, (int, float)):
        return float(x)
    try:
        return float(x) if getattr(x, "ndim", None) == 0 else None
    except (TypeError, ValueError):
        return None


def is_finite(x) -> bool:
    """唯一的「是不是有限数」判断（codex guards 第 4 轮 · 改结构）。递归字典 / 列表 / 元组；
    bool / None / 字符串不是读数，当作没问题。别的模块不许自己写 isnan / isinf 之类的判断，
    tests/test_finite.py 会扫（防无心之失）。"""
    if isinstance(x, dict):
        return all(is_finite(v) for v in x.values())
    if isinstance(x, (list, tuple)):
        return all(is_finite(v) for v in x)
    v = _num(x)
    return v is None or math.isfinite(v)


def _bad(x, path: str = "") -> dict:
    """收集不是有限数的读数（路径 → 值），写进 NonFinite.detail；空字典表示全是有限数。"""
    if isinstance(x, dict):
        out = {}
        for k, v in x.items():
            out |= _bad(v, f"{path}.{k}" if path else str(k))
        return out
    if isinstance(x, (list, tuple)):
        out = {}
        for i, v in enumerate(x):
            out |= _bad(v, f"{path}[{i}]")
        return out
    v = _num(x)
    return {} if v is None or is_finite(v) else {path or "value": v}


def require_finite(where: str, values) -> None:
    """读数里只要有一个不是有限数就抛 `NonFinite`（where 说清在哪），不给「下一批想停就返回 None」
    之类的路留余地（codex guards 第 4 轮 · 改结构）。"""
    if not is_finite(values):
        raise NonFinite(where, _bad(values))


class Guard:
    def __init__(self, expect_s_step: float | None = None):
        self.expect = expect_s_step
        self.base_vals: list[float] = []
        self.base: float | None = None
        self.window: list[float] = []
        self.speed: list[float] = []
        self.speed_judged = False
        self.best: dict | None = None
        self.bad = 0

    # ---- 每个训练步 ----
    def on_train_step(self, step: int, total: float, dt_s: float) -> str | None:
        """step 是这一步做完后的步数（从 1 起）。返回要停的原因或 None。"""
        why = self._spike(total)
        return why or self._slow(step, dt_s)

    def _spike(self, total: float) -> str | None:
        v = _num(total)
        if v is None or not is_finite(v):  # 不是有限数交给 ① 处理，这里不判
            return None
        if self.base is None:
            self.base_vals.append(v)
            if len(self.base_vals) >= SPIKE_BASE_STEPS:
                self.base = statistics.median(self.base_vals)
            return None
        self.window.append(v)
        del self.window[:-SPIKE_WINDOW]
        if len(self.window) < SPIKE_WINDOW:
            return None
        m = statistics.median(self.window)
        return "loss_spike" if m - self.base > (SPIKE_K - 1) * abs(self.base) else None

    def _slow(self, step: int, dt_s: float) -> str | None:
        if self.expect is None or self.speed_judged:
            return None
        dt = _num(dt_s)
        if SPEED_FROM <= step <= SPEED_TO and dt is not None and is_finite(dt):
            self.speed.append(dt)
        if step < SPEED_TO:
            return None
        self.speed_judged = True
        if not self.speed:
            return None
        return "too_slow" if statistics.median(self.speed) > SLOW_FACTOR * self.expect else None

    def spike_detail(self) -> dict:
        return {"base": self.base, "median_window": statistics.median(self.window) if self.window else None,
                "k": SPIKE_K, "base_steps": SPIKE_BASE_STEPS, "window": SPIKE_WINDOW}

    def slow_detail(self) -> dict:
        return {"median_s_step": statistics.median(self.speed) if self.speed else None, "expect_s_step": self.expect,
                "factor": SLOW_FACTOR, "from_step": SPEED_FROM, "to_step": SPEED_TO, "samples": len(self.speed)}

    # ---- 每次开发评估 ----
    def on_eval(self, step: int, hybrid) -> dict:
        """返回 {"nonfinite", "best", "warn"}；best 为真时调用方另存 best.pt。"""
        h = _num(hybrid)
        if h is None or not is_finite(h):  # 缺读数与坏读数一样按非有限处理
            return {"nonfinite": True, "best": False, "warn": None}
        if self.best is None or h < self.best["hybrid"]:
            self.best, self.bad = {"step": step, "hybrid": h}, 0
            return {"nonfinite": False, "best": True, "warn": None}
        self.bad = self.bad + 1 if h > self.best["hybrid"] + BEST_TOL else 0
        warn = None
        if self.bad >= PATIENCE:
            warn = (f"开发份 hybrid 连续 {self.bad} 次比最好的一次（第 {self.best['step']} 步 "
                    f"{self.best['hybrid']:.5f}）高 {BEST_TOL:g} 以上")
        return {"nonfinite": False, "best": False, "warn": warn}

    # ---- 续训 ----
    def state_dict(self) -> dict:
        return {"base_vals": list(self.base_vals), "base": self.base, "window": list(self.window),
                "speed": list(self.speed), "speed_judged": self.speed_judged, "best": self.best, "bad": self.bad}

    def load_state_dict(self, d: dict) -> None:
        self.base_vals, self.base = list(d["base_vals"]), d["base"]
        self.window, self.speed = list(d["window"]), list(d["speed"])
        self.speed_judged, self.best, self.bad = d["speed_judged"], d["best"], d["bad"]
