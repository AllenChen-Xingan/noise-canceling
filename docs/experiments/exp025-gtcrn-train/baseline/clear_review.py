"""清识别费账本的 review（待核）标记：费用未知的那几笔按最坏情况估算入账，再清掉 review。

大总管 2026-11-25 定：超时 0 字节那笔按「12 秒音频 × 标价」最坏估算入账，标注「费用未知，按最坏估」，然后清 review。
在账本正本所在的机器上跑（云上：/workspace/asr-ledger/），跑之前所有 vdn-eval 进程要停。

用法：
    python3 clear_review.py <账本> --expect-sha256 <当前账本的 sha256> --worst-count N [--audio-s 12] --note "<说明>"

做的事（任一步不对就整个拒绝，什么都不改）：
1. 账本当前 sha256 必须等于 --expect-sha256（防止核过之后又有人写过）
2. 账本必须带 review；拿 `<账本>.lock` 的文件锁（与 vdn-eval 同一把）
3. 先把原文件复制成 `<账本>.before-clear-<UTC 时间>.json`（只新建，已存在就拒绝）
4. spent_usd += N × audio_s × 标价；reserved_usd 归 0（停掉的进程留下的预留，已按最坏估计入账）；calls += N；
   review 删掉；adjustments 追加一条记录（原 review 原文、N、金额、放掉的预留、备份文件名与 sha256、说明）
5. 先写临时文件再改名（原子写）

标价不在这里另写：从 src/eval/src/qa/asr_or.rs 的 PRICE_PER_SECOND 读（唯一真相源）。
"""

import argparse
import datetime as dt
import fcntl
import hashlib
import json
import os
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[4]
PRICE_SRC = REPO / "src/eval/src/qa/asr_or.rs"


def price_per_second(src=PRICE_SRC):
    m = re.search(r"pub const PRICE_PER_SECOND: f64 = ([0-9_.]+);", Path(src).read_text())
    if not m:
        raise SystemExit(f"读不到 PRICE_PER_SECOND：{src}")
    return float(m.group(1).replace("_", ""))


def sha256(b):
    return hashlib.sha256(b).hexdigest()


def clear(path, expect_sha, worst_count, audio_s, note, now=None, price=None):
    """返回 (新账本 dict, 备份路径)。不对就抛 SystemExit，不改任何文件。"""
    path = Path(path)
    price = price_per_second() if price is None else price
    now = now or dt.datetime.now(dt.timezone.utc)
    import math
    if worst_count < 0 or not (math.isfinite(audio_s) and audio_s > 0) or not (math.isfinite(price) and price > 0):
        raise SystemExit("--worst-count 不能为负；--audio-s 与标价要是大于 0 的有限数")
    lock = open(str(path) + ".lock", "a")
    try:
        fcntl.flock(lock, fcntl.LOCK_EX)
        raw = path.read_bytes()
        if sha256(raw) != expect_sha:
            raise SystemExit(f"账本 sha256 是 {sha256(raw)}，不是给的 {expect_sha}：有人改过，先重新核")
        led = json.loads(raw)
        if not led.get("review"):
            raise SystemExit("账本没有 review，不用清")
        adj = led.get("adjustments", [])
        if not isinstance(adj, list):
            raise SystemExit("账本的 adjustments 不是列表：先人工查")
        stamp = now.strftime("%Y%m%dT%H%M%SZ")
        backup = Path(f"{path}.before-clear-{stamp}.json")
        worst = worst_count * audio_s * price
        new = dict(led)
        new["spent_usd"] = led.get("spent_usd", 0.0) + worst
        new["calls"] = led.get("calls", 0) + worst_count
        new["reserved_usd"] = 0.0
        del new["review"]
        new["adjustments"] = adj + [{
            "kind": "review-cleared",
            "at": now.strftime("%Y-%m-%dT%H:%M:%SZ"),
            "review": led["review"],
            "worst_count": worst_count,
            "audio_s": audio_s,
            "price_per_second": price,
            "worst_usd": worst,
            "reserved_released_usd": led.get("reserved_usd", 0.0),
            "backup": backup.name,
            "backup_sha256": sha256(raw),
            "note": "费用未知，按最坏估。" + note,
        }]
        # 先全部算完、序列化（有非有限数就在这里拒），再动文件（codex 本分支第 1 轮 P1 #3 · P2 #4）
        try:
            text = json.dumps(new, ensure_ascii=False, indent=2, allow_nan=False)
        except ValueError as e:
            raise SystemExit(f"新账本里有非有限数（{e}）：不写")
        tmp = Path(str(path) + ".tmp")
        made = []
        try:
            with open(backup, "xb") as f:  # 只新建
                made.append(backup)
                f.write(raw)
            with open(tmp, "x", encoding="utf-8") as f:  # 只新建：已有同名临时文件就拒，不截断别人的（codex 本分支第 2 轮 P2 #2）
                made.append(tmp)
                f.write(text)
            os.replace(tmp, path)
        except BaseException:
            for f in made:
                if f.exists():
                    f.unlink()
            raise
        return new, backup
    finally:
        fcntl.flock(lock, fcntl.LOCK_UN)
        lock.close()


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("ledger")
    ap.add_argument("--expect-sha256", required=True)
    ap.add_argument("--worst-count", type=int, required=True)
    ap.add_argument("--audio-s", type=float, default=12.0)
    ap.add_argument("--note", required=True)
    a = ap.parse_args(argv)
    new, backup = clear(a.ledger, a.expect_sha256, a.worst_count, a.audio_s, a.note)
    out = Path(a.ledger).read_bytes()
    print(f"已清 review；备份 {backup.name}；spent_usd {new['spent_usd']:.6f}，calls {new['calls']}；新 sha256 {sha256(out)}")


if __name__ == "__main__":
    sys.exit(main())
