"""训练时的自动保护（plan-h100-smoke.md 第十二节；大总管定 2026-11-25，技术题，人未参与）。阈值正本都在这里。

训练进程自己判，不靠本机醒着；触发时 train.py 记停止原因与一条 guard 记录、能存就存检查点、退出码非 0，
由 cloudrun 的 job-wait 报大总管。每个运行只看自己的读数，一个运行停下不碰别的运行。
- ② 损失猛涨：前 SPIKE_BASE_STEPS 步训练损失（total）中位数作基准 b，最近 SPIKE_WINDOW 步中位数 m；
  m − b > (SPIKE_K − 1)·|b| 就停（loss_spike）。用中位数：单个坏批不误停；用 |b|：total 可能为负，跨零也成立
- ③ 开发份 hybrid 最好的一次另存 best.pt；连续 PATIENCE 次比最好的高 BEST_TOL 以上记警告，不停
- ④ 期望每步秒数在配置 limits.expect_s_step（默认不设 = 关）；第 SPEED_FROM 到 SPEED_TO 步单步墙钟中位数
  > SLOW_FACTOR × 期望，就在第 SPEED_TO 步停（too_slow）。单步墙钟含取数，不含评估与存盘
- ⑤ 中途卡住：主线程 STALL_S 秒没进展，看门狗不管该不该停都强退（watchdog._Beat）
- ①（不是有限数）的判断在 train.py：损失、梯度范数、开发份读数；本模块只给开发份读数标 nonfinite
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


def _fin(x) -> bool:
    return isinstance(x, (int, float)) and not isinstance(x, bool) and math.isfinite(x)


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
        if not _fin(total):
            return None
        if self.base is None:
            self.base_vals.append(float(total))
            if len(self.base_vals) >= SPIKE_BASE_STEPS:
                self.base = statistics.median(self.base_vals)
            return None
        self.window.append(float(total))
        del self.window[:-SPIKE_WINDOW]
        if len(self.window) < SPIKE_WINDOW:
            return None
        m = statistics.median(self.window)
        return "loss_spike" if m - self.base > (SPIKE_K - 1) * abs(self.base) else None

    def _slow(self, step: int, dt_s: float) -> str | None:
        if self.expect is None or self.speed_judged:
            return None
        if SPEED_FROM <= step <= SPEED_TO and _fin(dt_s):
            self.speed.append(float(dt_s))
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
        if not _fin(hybrid):
            return {"nonfinite": True, "best": False, "warn": None}
        h = float(hybrid)
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
