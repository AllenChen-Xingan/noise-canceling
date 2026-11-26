"""exp025 数据消融（#212 · 014 第 4 步）：7 个运行的开训前检查 · 生成命令 · 判结果。

这是什么：只跑模型 A、每个运行 1000 步、比开发份的 hybrid（越低越好），看数据配方的哪一个旋钮
真正有用。7 个运行（大总管定 2026-11-26）：
  - `abl-base-s0` / `abl-base-s1`：基线配方，种子 0 和 1（两者的差就是噪声底 floor）
  - `abl-eq-off` / `abl-clip-off` / `abl-n_noises-off` / `abl-reverb-off`：基线配方，只把该旋钮的 `on` 改成 false
  - `abl-high_snr-on`：基线配方，只把 `high_snr.on` 改成 true

用法（在 trainer/ 目录下）：
    python ablation.py check                # 有拦路的问题就逐条列出来，退出码 1；全通过打印「可以开训」，退出码 0
    python ablation.py commands             # 按 3 + 3 + 1 分批打印开训命令；有问题时列问题、退出码 1
    python ablation.py judge <运行目录> --out <结果.json>   # 判 7 个运行；不完整时退出码 2，结果文件不覆盖

条件正本：
- 配置字段与默认值：config.py（本文件只读它，不再手抄默认值）；运行集合与判法都写在本文件顶上，
  `problems()` 核对运行矩阵与这里的常量一致。
- 变体配方由 traindata 写（`data/recipes/ablation/`）：每份只改一个旋钮的 `on`，顶层 `_ablation` 记
  `base_sha256` / `knob` / `on`；`problems()` 拿它们和基线配方逐项对，改多了、改错了都报出来。
"""
from __future__ import annotations

import argparse
import json
import math
import sys
from hashlib import sha256
from pathlib import Path

import yaml

import config as cfgmod
import formal  # 复用 TODO 占位 / base 解析 / 命令里的 --config 路径（私有函数照用，不改它的行为）

HERE = Path(__file__).resolve().parent
DEFAULT_RUNS = HERE / "configs" / "ablation-runs.yaml"
STEP = 1000  # 比第 1000 步的开发份 hybrid
BATCH_RUNS = 3  # 一张 H100 同时跑 3 个（plan-h100-smoke.md 第五节）

RULE = (
    "大总管定 2026-11-26：模型 A、每个运行 1000 步，比开发份 hybrid（越低越好）。"
    "记两个基线（种子 0、1）在第 1000 步的读数为 b0、b1，base = (b0 + b1) / 2，floor = |b0 − b1|"
    "（两个基线的差就是噪声底）。对每个旋钮运行，d = 它的 hybrid − base："
    "默认开着的旋钮被关掉时，d > floor 判「useful: keep on」（关掉变差了，说明它有用），"
    "d < −floor 判「hurts: candidate to turn off」（关掉反而更好），否则判「消融分不出: keep as is (on)」；"
    "high_snr 被打开时，d < −floor 判「useful: turn on」，d > floor 判「hurts: keep off」，"
    "否则判「消融分不出: keep as is (off)」。比较都是严格不等式。"
    "读数缺失 · 不是有限数 · 没有第 1000 步 · 数据字段对不上 → 该运行判 incomplete；"
    "基线 incomplete 时每个旋钮都 incomplete。"
)

# 判词正本（judge 的返回、测试、报告都用这几个字符串，别处不另写一套）
V_KEEP_ON = "useful: keep on"
V_CAND_OFF = "hurts: candidate to turn off"
V_TIE_ON = "消融分不出: keep as is (on)"
V_TURN_ON = "useful: turn on"
V_KEEP_OFF = "hurts: keep off"
V_TIE_OFF = "消融分不出: keep as is (off)"
V_INCOMPLETE = "incomplete"

BASE_RUNS = ("abl-base-s0", "abl-base-s1")
# 单旋钮运行的（旋钮, 变体里应该是什么值）：默认开着的关掉，默认关着的 high_snr 打开
KNOB_RUNS = (("eq", False), ("clip", False), ("n_noises", False), ("reverb", False), ("high_snr", True))

RUN_KEYS = ("name", "seed", "knob", "on")
# 运行集合正本：(名字, 种子, 旋钮, 目标 on)。顺序就是开训顺序（3 + 3 + 1 分批按它切）
WANTED_RUNS = (
    ("abl-base-s0", 0, None, None),
    ("abl-base-s1", 1, None, None),
    ("abl-eq-off", 0, "eq", False),
    ("abl-clip-off", 0, "clip", False),
    ("abl-n_noises-off", 0, "n_noises", False),
    ("abl-reverb-off", 0, "reverb", False),
    ("abl-high_snr-on", 0, "high_snr", True),
)

# judge 比的对不上就要报的 data 字段（config.data 里除 recipe 之外全部要比：开发份的配方与规模一变，读数就不可比）。
# data_build_id 按设计本来就各不同（每个配方一份新 build），不在此列。
DATA_IGNORE = ("recipe",)


# ---------- 路径与占位 ----------

def _T_dir(runs_path) -> Path:
    """trainer/ 目录：运行矩阵在 configs/ 下，取它的上两级（命令也是在 trainer/ 下跑）。"""
    return Path(runs_path).resolve().parent.parent


def _resolve(T: Path, p, want_dir: bool = False) -> Path | None:
    """把配置里的路径解成真实路径：绝对路径照用，相对路径一律相对 trainer/（train.py 在那里跑）。
    找不到返回 None（由调用方给一句中文）。"""
    if not isinstance(p, str) or not p.strip():
        return None
    q = Path(p)
    c = q if q.is_absolute() else T / q
    return c if (c.is_dir() if want_dir else c.is_file()) else None


def variant_recipe_name(knob, on) -> str:
    """变体配方文件名：关掉默认开着的是 `no-eq.json`，打开默认关着的是 `high_snr-on.json`。"""
    if not isinstance(knob, str) or not knob:
        raise ValueError("变体配方要指定旋钮名")
    return f"{knob}-on.json" if on else f"no-{knob}.json"


# ---------- 开训前检查 ----------

def _run_set_problems(items) -> list[str]:
    """运行矩阵 runs 的问题：四项都写 · 恰好是顶上那 7 个（名字、种子、旋钮、on 逐项）。"""
    if not isinstance(items, list) or not items:
        return ["运行矩阵 runs 空着：数据消融恰好 7 个运行（2 个基线 + 4 个关旋钮 + 1 个开 high_snr）"]
    ps: list[str] = []
    got = []
    for i, run in enumerate(items, 1):
        if not isinstance(run, dict):
            ps.append(f"runs 第 {i} 项不是运行：每一项要写 name、seed、knob、on")
            continue
        miss = [k for k in RUN_KEYS if k not in run]
        if miss:
            # YAML 1.1 把裸写的 on 读成 true：那个键就变成 True，不再是 "on"
            hint = "（YAML 把没加引号的 on 读成 true：键要写成 \"on\"）" if "on" in miss else ""
            ps.append(f"runs 第 {i} 项缺 {miss[0]}：四项都要写（基线把 knob 与 on 都写 null）{hint}")
        got.append(tuple(run.get(k) for k in RUN_KEYS))
    want = list(WANTED_RUNS)
    if got != want:
        lack = [w for w in want if w not in got]
        extra = [g for g in got if g not in want]
        parts = []
        if lack:
            parts.append(f"缺 {lack}")
        if extra:
            parts.append(f"多了或不对 {extra}")
        ps.append("运行矩阵不是约定的 7 个运行（2 个基线种子 0/1 · eq/clip/n_noises/reverb 各关一个 · "
                  f"high_snr 打开）：{'；'.join(parts)}")
    return ps


def _variant_problems(runs: dict, cfg, T: Path) -> list[str]:
    """变体配方与基线配方的交叉校验（traindata 写的文件要对得上这里要的开关）。"""
    ps: list[str] = []
    base_rel = cfg.data.recipe
    bfile = _resolve(T, base_rel)
    if bfile is None:
        return [f"基线配方 {base_rel} 找不到（相对 trainer/ 找）：data.recipe 要指向配方文件"]
    try:
        base_bytes = bfile.read_bytes()
        base = json.loads(base_bytes)
    except (OSError, ValueError) as e:
        return [f"读不了基线配方 {base_rel}：{e}"]
    if not isinstance(base, dict) or not isinstance(base.get("knobs"), dict):
        return [f"基线配方 {base_rel} 结构不对：要是一个带 knobs 的 JSON 对象"]
    bsha = sha256(base_bytes).hexdigest()
    base_flat = formal._flat(base)

    vdir = runs.get("variants_dir")
    if not isinstance(vdir, str) or not vdir.strip():
        return ["运行矩阵没写 variants_dir：变体配方放在哪个目录"]
    vpath = _resolve(T, vdir, want_dir=True)
    if vpath is None:
        ps.append(f"变体配方目录 {vdir} 不存在（相对 trainer/ 找）：等 traindata 把这个目录写出来")

    for run in runs.get("runs") or []:
        if not isinstance(run, dict) or run.get("knob") is None:
            continue
        knob, on, name = run.get("knob"), run.get("on"), run.get("name")
        if not isinstance(knob, str) or not knob or not isinstance(on, bool):
            ps.append(f"运行 {name} 的 knob / on 写错：knob 写旋钮名、on 写 true / false（基线两者都写 null）")
            continue
        fname = variant_recipe_name(knob, on)
        f = (vpath or (T / vdir)) / fname
        if not f.is_file():
            ps.append(f"运行 {name} 要的变体配方 {vdir}/{fname} 不存在：等 traindata 写出来并登记新 build")
            continue
        try:
            var = json.loads(f.read_text())
        except (OSError, ValueError) as e:
            ps.append(f"读不了变体配方 {fname}：{e}")
            continue
        if not isinstance(var, dict):
            ps.append(f"变体配方 {fname} 不是 JSON 对象")
            continue
        marker = var.get("_ablation")
        if not isinstance(marker, dict):
            ps.append(f"变体配方 {fname} 缺顶层 _ablation（要写 base_sha256 · knob · on）："
                      "没法核对它是从哪一份基线配方改出来的")
        else:
            if marker.get("base_sha256") != bsha:
                ps.append(f"变体配方 {fname} 的 _ablation.base_sha256 和基线配方 {base_rel} 的 sha256 对不上："
                          "它不是从这份基线配方改出来的，读数没法比")
            if marker.get("knob") != knob:
                ps.append(f"变体配方 {fname} 的 _ablation.knob 是 {marker.get('knob')!r}，"
                          f"运行 {name} 要的旋钮是 {knob!r}：两处要一致")
            if marker.get("on") != on:
                ps.append(f"变体配方 {fname} 的 _ablation.on 是 {marker.get('on')!r}，"
                          f"运行 {name} 要的值是 {on!r}：两处要一致")
        want_path = f"knobs.{knob}.on"
        vflat = formal._flat({k: v for k, v in var.items() if k != "_ablation"})
        diff = sorted(k for k in set(vflat) | set(base_flat) if vflat.get(k) != base_flat.get(k))
        if diff != [want_path]:
            ps.append(f"变体配方 {fname} 应该只在 {want_path} 一处与基线配方不同，"
                      f"实际不同在 {diff or '没有一处'}：变体只能改这一个开关")
        elif vflat[want_path] != on:
            ps.append(f"变体配方 {fname} 的 {want_path} 是 {vflat[want_path]!r}，运行 {name} 要 {on!r}")
    return ps


def problems(runs_path) -> list[str]:
    """开训会拦下来的一切问题，每条一句中文。空列表 = 可以开训。"""
    p = Path(runs_path)
    try:
        runs = formal.load_runs(p)
    except (OSError, ValueError) as e:
        return [f"读不了运行矩阵 {p}：{e}"]
    ps: list[str] = []
    base = None
    try:
        base = formal._base_path(p, runs)
    except ValueError:
        ps.append("运行矩阵没写 base（或它不是路径）：base 要指向 configs/ablation.yaml")
    if base is not None:
        try:
            cfg = cfgmod.load(base)
        except FileNotFoundError:
            ps.append(f"运行矩阵的 base 指向的文件不存在：{runs.get('base')}")
        except ValueError as e:
            ps.append(f"base 指向的配置本身不合法：{e}")
        else:
            for k, v in formal._todo(cfg):
                ps.append(f"配置 {k} 还是占位 `{v}`：开训前必须填成实际路径")
            for k in formal._blank(cfg):
                ps.append(f"配置 {k} 空着：开训前必须填成实际路径")
            ps.extend(_variant_problems(runs, cfg, _T_dir(p)))
    ps.extend(_run_set_problems(runs.get("runs")))
    return ps


# ---------- 开训命令 ----------

def commands(runs_path, T_dir: str = "$T", python: str = "$PY", gtcrn: str = "$G") -> list[list[str]]:
    """按 3 + 3 + 1 分批：每批最多 3 条命令。有问题时抛 ValueError（内容是 problems 的全部条目）。"""
    p = Path(runs_path)
    ps = problems(p)
    if ps:
        raise ValueError("还不能开训：" + "；".join(ps))
    runs = formal.load_runs(p)
    base = formal._base_path(p, runs)
    cfg = cfgmod.load(base)
    cfg_arg = formal._cfg_arg(p, base)
    vdir = str(runs["variants_dir"]).rstrip("/")
    dev = cfg.data.recipe  # 开发份用基线配方：7 个运行的开发集必须是同一份（data.dev_recipe）
    out: list[str] = []
    for run in runs["runs"]:
        name, seed, knob, on = (run[k] for k in RUN_KEYS)
        recipe = dev if knob is None else f"{vdir}/{variant_recipe_name(knob, on)}"
        out.append(f"cd {T_dir} && VDN_GTCRN_DIR={gtcrn} {python} train.py --config {cfg_arg} "
                   f"--set run.name={name} --set run.seed={seed} --set data.recipe={recipe} "
                   f"--set data.dev_recipe={dev}")
    return [out[i:i + BATCH_RUNS] for i in range(0, len(out), BATCH_RUNS)]


# ---------- 判结果 ----------

def _num(x) -> bool:
    return isinstance(x, (int, float)) and not isinstance(x, bool) and math.isfinite(x)


def _read_run(run_dir: Path, step: int) -> dict:
    """一个运行的记录：第 step 步的开发份 hybrid · 数据字段 · 两个 build id。读不出就填 reason。"""
    out = {"dev_hybrid": None, "data": None, "data_build_id": None, "dev_build_id": None, "reason": None}
    try:
        rec = json.loads((run_dir / "run.json").read_text())
        rows = [json.loads(x) for x in (run_dir / "metrics.jsonl").read_text().splitlines() if x.strip()]
        data = rec["config"]["data"]
        if not isinstance(data, dict):
            raise TypeError("config.data 不是映射")
    except (OSError, ValueError, KeyError, TypeError) as e:
        out["reason"] = f"读不了运行记录：{type(e).__name__}: {e}"
        return out
    out["data"] = {k: v for k, v in data.items() if k not in DATA_IGNORE}
    out["data_build_id"] = rec.get("data_build_id")
    out["dev_build_id"] = rec.get("dev_build_id")
    hits = [r["dev"].get("hybrid") for r in rows if r.get("step") == step and isinstance(r.get("dev"), dict)]
    if not hits:
        out["reason"] = f"没有第 {step} 步的开发份 hybrid 读数（这个运行没跑到 / 跑崩了 / 记录不全）"
    elif not _num(hits[-1]):
        out["reason"] = f"第 {step} 步的开发份 hybrid 不是有限数：{hits[-1]!r}"
    else:
        out["dev_hybrid"] = float(hits[-1])
    return out


def _verdict(on: bool, d: float, floor: float) -> str:
    """严格不等式；on 是「这个旋钮在变体里被打开了」——默认关着的打开了，判读方向就反过来。"""
    if on:
        return V_TURN_ON if d < -floor else (V_KEEP_OFF if d > floor else V_TIE_OFF)
    return V_KEEP_ON if d > floor else (V_CAND_OFF if d < -floor else V_TIE_ON)


def judge(out_dir, step: int = STEP) -> dict:
    """判 7 个运行：返回 {rule, base, floor, runs, knobs}。任何运行不完整 → 它在 runs 里是 incomplete；
    两条基线里任一条不完整（读数、build id、数据字段任一项过不了）→ base 与 floor 都不算，
    每个旋钮都 incomplete。"""
    root = Path(out_dir)
    recs = {name: _read_run(root / name, step) for name, _s, _k, _o in WANTED_RUNS}
    ref_name = BASE_RUNS[0]
    ref = recs[ref_name]
    for name, r in recs.items():
        if r["reason"] or name == ref_name:
            continue
        if ref["reason"]:
            break  # 基线自己就读不出：别的运行的数据字段没有参照，各自的读数问题已经记下了
        if r["dev_build_id"] != ref["dev_build_id"]:
            r["reason"] = (f"开发份 build id 是 {r['dev_build_id']!r}，和 {ref_name} 的 "
                           f"{ref['dev_build_id']!r} 不同：7 个运行必须同用一份开发集，读数才可比")
            continue
        if r["dev_build_id"] is None:
            r["reason"] = "run.json 里没有 dev_build_id：认不出开发集版本，读数没法比"
            continue
        bad = sorted(k for k in set(r["data"]) | set(ref["data"]) if r["data"].get(k) != ref["data"].get(k))
        if bad:
            r["reason"] = (f"数据字段和 {ref_name} 对不上：{bad}（"
                           + "，".join(f"{k} {r['data'].get(k)!r} ≠ {ref['data'].get(k)!r}" for k in bad) + "）")
    if ref["reason"] is None and ref["dev_build_id"] is None:
        ref["reason"] = "run.json 里没有 dev_build_id：认不出开发集版本，读数没法比"

    # base / floor 只用两条基线算；任一条基线不完整就不许算——否则旋钮会被拿去跟一条不可信的
    # 基线比（#212 评审发现：种子 1 的 build id / 数据字段对不上时它 dev_hybrid 还在，旧代码照算）
    bad_base = [n for n in BASE_RUNS if recs[n]["reason"]]
    b0, b1 = recs[BASE_RUNS[0]]["dev_hybrid"], recs[BASE_RUNS[1]]["dev_hybrid"]
    base = floor = None
    if not bad_base and _num(b0) and _num(b1):
        base, floor = (b0 + b1) / 2, abs(b0 - b1)

    knobs: dict = {}
    for name, _seed, knob, on in WANTED_RUNS:
        if knob is None:
            continue
        r = recs[name]
        why = r["reason"]
        if why is None and base is None:
            why = "基线不完整：base 与 floor 都没有，判不了——" + "；".join(
                f"{n}：{recs[n]['reason']}" for n in bad_base)
        if why is not None:
            knobs[knob] = {"run": name, "on": on, "d": None, "verdict": V_INCOMPLETE, "reason": why}
            continue
        d = r["dev_hybrid"] - base
        knobs[knob] = {"run": name, "on": on, "d": d, "verdict": _verdict(on, d, floor), "reason": None}

    runs = {name: {"dev_hybrid": r["dev_hybrid"], "data_build_id": r["data_build_id"],
                   "dev_build_id": r["dev_build_id"], "verdict": V_INCOMPLETE if r["reason"] else "ok",
                   "reason": r["reason"]} for name, r in recs.items()}
    return {"rule": RULE, "base": base, "floor": floor, "runs": runs, "knobs": knobs}


def _complete(result: dict) -> bool:
    return all(r["verdict"] != V_INCOMPLETE for r in result["runs"].values())


# ---------- 命令行 ----------

def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description="exp025 数据消融：开训前检查 / 生成开训命令 / 判结果")
    ap.add_argument("action", choices=("check", "commands", "judge"))
    ap.add_argument("path", nargs="?", help="check / commands 给运行矩阵（默认 configs/ablation-runs.yaml）；"
                                           "judge 给 7 个运行所在的总目录")
    ap.add_argument("--out", help="judge 的结果写到这个新文件（已存在就报错，绝不覆盖旧产物）")
    ap.add_argument("--T-dir", default="$T", help="生成命令里 trainer/ 的位置")
    ap.add_argument("--python", default="$PY", help="生成命令里的 python")
    ap.add_argument("--gtcrn", default="$G", help="生成命令里的 VDN_GTCRN_DIR（GTCRN 起点权重目录）")
    a = ap.parse_args(argv)

    if a.action == "judge":
        if not a.path:
            print("judge 要给 7 个运行所在的总目录", file=sys.stderr)
            return 1
        if not a.out:
            print("judge 要 --out <结果.json>：结果落成文件，不覆盖", file=sys.stderr)
            return 1
        result = judge(a.path)
        print(f"基线均值 base = {result['base']} · 噪声底 floor = {result['floor']}")
        for knob, v in result["knobs"].items():
            print(f"- {knob}: d = {v['d']} → {v['verdict']}" + (f"（{v['reason']}）" if v["reason"] else ""))
        out = Path(a.out)
        try:
            out.parent.mkdir(parents=True, exist_ok=True)
            with open(out, "x", encoding="utf-8") as f:
                json.dump(result, f, ensure_ascii=False, indent=1)
        except FileExistsError:
            print(f"{out} 已存在：实验产物不许原地覆盖，换一个文件名", file=sys.stderr)
            return 1
        return 0 if _complete(result) else 2

    runs = a.path or str(DEFAULT_RUNS)
    ps = problems(runs)
    if ps:
        for s in ps:
            print(f"- {s}")
        return 1
    if a.action == "check":
        print("可以开训")
        return 0
    for i, block in enumerate(commands(runs, a.T_dir, a.python, a.gtcrn), 1):
        print(f"# 第 {i} 批：{len(block)} 个同时跑")
        for c in block:
            print(c)
        print()
    return 0


if __name__ == "__main__":
    sys.exit(main())
