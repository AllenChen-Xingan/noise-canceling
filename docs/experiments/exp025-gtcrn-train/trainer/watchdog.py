"""训练进程的保险：信号 · 心跳 · 看门狗（从 train.py 按职责拆出，CLAUDE.md 五.7）。

- `_Signals`：收到 SIGTERM / SIGINT 置位，主循环在本步结束后存盘停下
- `_Beat`：后台线程定时写心跳（计费账），兼作看门狗：已该停而主线程久无进展时强退
- `_Deadline`：不碰磁盘的兜底看门狗：本段进程跑满「时长上限 + hang_s」就强退
- `_bounded` / `_hard_exit`：有时限地做一件事、强退前尽量记一笔
"""
from __future__ import annotations

import json
import os
import signal
import threading
import time


class _Signals:
    """收到 SIGTERM / SIGINT（RunPod 抢占、人按 Ctrl-C）时置位，循环在本步结束后存检查点停下。"""

    def __init__(self):
        self.got = None
        self.old = {}

    def __enter__(self):
        if threading.current_thread() is threading.main_thread():
            for sig in (signal.SIGTERM, signal.SIGINT):
                self.old[sig] = signal.signal(sig, self._on)
        return self

    def _on(self, signum, frame):
        self.got = signal.Signals(signum).name

    def __exit__(self, *exc):
        for sig, h in self.old.items():
            signal.signal(sig, h)


HEARTBEAT_S = 30.0


def _bounded(fn, timeout: float) -> bool:
    """在另一个线程里跑 fn，最多等 timeout 秒；异常或超时都只返回 False，不往外抛、不挡调用方。
    看门狗靠它把「写盘」和「判断要不要强退」隔开：网络盘卡住或报错时，强退照样执行（codex 第 7 轮 #1）。"""
    box = {"ok": False}

    def run():
        try:
            fn()
            box["ok"] = True
        except Exception:  # noqa: BLE001 —— 记录失败不能阻止退出
            pass

    t = threading.Thread(target=run, daemon=True)
    t.start()
    t.join(timeout)
    return box["ok"]


def _hard_exit(rd, why: str, _exit=os._exit) -> None:
    """看门狗的最后手段：尽量记一笔（最多等 2 s，写不成也不管），然后强退。
    最近的检查点是完整的（原子写），下次按心跳续训、补账。"""
    text = json.dumps({"why": why, "at": time.strftime("%Y-%m-%dT%H:%M:%S%z")}, ensure_ascii=False)
    _bounded(lambda: (rd.root / "hung.json").write_text(text), 2.0)
    _exit(75)


def _hung_message(reason: str, stall: float, hang_s: float) -> str:
    """强退前记下的一句：挑出来的原因 + 主线程多久没进展（codex guards 第 4 轮 · 改结构）。"""
    if str(reason).split(":", 1)[0] == "stall":
        return f"卡住：主线程 {stall:.0f} s 无进展"
    return f"{reason}；主线程 {hang_s:.0f} s 无进展"


class _Beat:
    """后台线程每 HEARTBEAT_S 秒写一次心跳（步数 · 计费秒 · 累计美元），取数卡住、算标定时也照写。
    被抢占时最多漏记一个间隔（≤ 30 s）；机器的真实账单归 cloudrun（codex 第 1 轮 #4）。心跳只由这个线程写。

    兼作看门狗（codex 第 6 轮 #1）：已该停（到上限 / 收到信号）而主线程超过 hang_s 没有进展（比如卡在一次
    同步取数里，worker = 0 时 DataLoader 没有超时），就强退，免得一直计费。
    每轮**先判断再写盘**，写盘有时限、出错不抛（codex 第 7 轮 #1）：盘坏了或卡住，看门狗仍能按时出手。"""

    def __init__(self, rd, snap, every: float, want_stop=lambda: None, last_progress=None,
                 hang_s: float = 120.0, on_hang=None, stall_s: float | None = None):
        self.rd, self.snap, self.every = rd, snap, every
        self.want_stop, self.last_progress, self.hang_s = want_stop, last_progress, hang_s
        self.stall_s = stall_s  # None = 用 guards.STALL_S（运行时读，测试可改）
        self.on_hang = on_hang or (lambda why: _hard_exit(rd, why))
        self.stop = threading.Event()
        self.closed = threading.Event()
        self.write_failures = 0
        self.inflight = None
        self.th = threading.Thread(target=self._run, daemon=True)

    def _beat(self) -> None:
        """单写者：上一次写入还没回来（盘卡住）就跳过本轮，不叠第二个写入；关闭后迟到的写入不提交（第 8 轮 #2）。"""
        if self.inflight is not None and self.inflight.is_alive():
            self.write_failures += 1
            return
        snap = self.snap()
        box = {}

        def run():
            try:
                self.rd.beat(*snap, commit_ok=lambda: not self.closed.is_set())
                box["ok"] = True
            except Exception:  # noqa: BLE001 —— 写不成只记次数
                pass

        self.inflight = threading.Thread(target=run, daemon=True)
        self.inflight.start()
        self.inflight.join(max(self.every, 1.0))
        if not box.get("ok"):
            self.write_failures += 1

    def _run(self):
        import guards  # 运行时读：测试可改 guards.STALL_S 与优先级表
        while not self.stop.wait(self.every):
            # 判断这一步本身出错（want_stop 抛异常、表外原因让 pick_reason 报错）也要出手：
            # 否则线程静默结束，on_hang 永不叫、心跳也停，卡住保护没了、只剩总时限兜底
            # （codex guards 第 5 轮 #3）。只包判断，不包 on_hang 自己的异常。
            try:
                why = self.want_stop()
                idle = None if self.last_progress is None else time.monotonic() - self.last_progress()
                stall = guards.STALL_S if self.stall_s is None else self.stall_s
                stopped = why if why and idle is not None and idle > self.hang_s else None
                hung = idle is not None and idle > stall
                # 两个条件同时成立时按 guards.STOP_PRIORITY 挑一个，消息里写它（codex guards 第 4 轮 · 改结构）
                top = guards.pick_reason(stopped, "stall" if hung else None)
            except Exception as e:  # noqa: BLE001 —— 判断失灵就按「出手」处理，不能静默丢掉看门狗
                self.on_hang(f"看门狗判断出错：{type(e).__name__}: {e}")
                return
            if top is not None:  # guards ⑤：不管该不该停，久无进展就强退（GPU 卡死、评估卡住）
                self.on_hang(_hung_message(top, stall, self.hang_s))
                return
            self._beat()

    def __enter__(self):
        self.th.start()
        return self

    def __exit__(self, *exc):
        self.stop.set()
        self.th.join()
        self._beat()
        self.closed.set()  # 之后还没写完的心跳一律不提交


class _Deadline:
    """兜底看门狗：本段进程跑满 seconds 秒就强退，不依赖任何磁盘读写（codex 第 8 轮 #1）。"""

    def __init__(self, seconds: float, on_hang=None):
        self.seconds, self.rd = seconds, None
        self.on_hang = on_hang
        self.done = threading.Event()
        self.th = threading.Thread(target=self._run, daemon=True)

    def _run(self):
        if not self.done.wait(self.seconds):
            why = f"本段已跑满 {self.seconds:.0f} s（时长上限 + hang_s）"
            if self.on_hang is not None:
                self.on_hang(why)
            elif self.rd is not None:
                _hard_exit(self.rd, why)
            else:
                os._exit(75)

    def __enter__(self):
        self.th.start()
        return self

    def __exit__(self, *exc):
        self.done.set()
