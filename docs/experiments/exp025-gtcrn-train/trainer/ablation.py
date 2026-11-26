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
- 数据 build 与各份配方的登记记录：buildpin.py（与 formal.py 同一份判法：配方与数据代码的 sha256
  必须和登记时一模一样，5 份变体也各有自己的那一份记录）。
"""
from __future__ import annotations

import argparse
import json
import math
import sys
from hashlib import sha256
from pathlib import Path

import yaml

import buildpin  # 数据 build 的登记核对（判法与 formal 同一份：配方 · 数据代码 · 变体各自的记录）
import config as cfgmod
import formal  # 复用 TODO 占位 / base 解析 / 命令里的 --config 路径（私有函数照用，不改它的行为）

HERE = Path(__file__).resolve().parent
DEFAULT_RUNS = HERE / "configs" / "ablation-runs.yaml"
STEP = 1000  # 比第 1000 步的开发份 hybrid
WANT_VARIANT = "A"  # 消融只跑模型 A：A、B 的差是另一件事，混进来就分不出旋钮的作用
BATCH_RUNS = 3  # 一张 H100 同时跑 3 个（plan-h100-smoke.md 第五节）
# 判结果时允许各不同的三处（**正本**）：其余 config 一律逐项比，缺键也算不同。RULE 的文字也从这里取。
CFG_IGNORE = ("run.name", "run.seed", "data.recipe")

RULE = (
    f"大总管定 2026-11-26：模型 {WANT_VARIANT}、每个运行 {STEP} 步，比开发份 hybrid（越低越好）。"
    f"记两个基线（种子 0、1）在第 {STEP} 步的读数为 b0、b1，base = (b0 + b1) / 2，floor = |b0 − b1|"
    "（两个基线的差就是噪声底）。对每个旋钮运行，d = 它的 hybrid − base："
    "默认开着的旋钮被关掉时，d > floor 判「useful: keep on」（关掉变差了，说明它有用），"
    "d < −floor 判「hurts: candidate to turn off」（关掉反而更好），否则判「消融分不出: keep as is (on)」；"
    "high_snr 被打开时，d < −floor 判「useful: turn on」，d > floor 判「hurts: keep off」，"
    "否则判「消融分不出: keep as is (off)」。比较都是严格不等式。"
    "每个运行的整份 config 也要对得上：拍平成「段.键」逐项和 abl-base-s0 比，只有 "
    + "、".join(CFG_IGNORE) +
    " 三项允许各不同，其余任何一项不同、或缺键，都算不同（缺键 · 键值是 null · 空对象，各算各的）；"
    f"而且每个运行都必须照设计跑：model.variant = {WANT_VARIANT} · schedule.max_steps = {STEP} · "
    "data.dev_recipe 指明的开发份配方等于基线的 data.recipe（7 个运行同用一份开发集）· "
    "run.seed 是运行矩阵给这个名字的种子（差一个种子就量不出 floor）· data.recipe 是这个名字说的那一份"
    "配方（基线用基线配方 = abl-base-s0 的 data.dev_recipe；旋钮运行的文件名是 no-<旋钮>.json 或 "
    "<旋钮>-on.json）。"
    "读数缺失 · 不是有限数 · 没有第 1000 步 · 指标行不是 JSON 对象 · 数据字段或整份配置对不上 · 不合设计"
    " → 该运行判 incomplete；基线 incomplete 时每个旋钮都 incomplete。"
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


# ---------- 路径与占位 ----------

def _T_dir(runs_path) -> Path:
    """trainer/ 目录：运行矩阵在 configs/ 下，取它的上两级（命令也是在 trainer/ 下跑）。"""
    return Path(runs_path).resolve().parent.parent


def _abs(T: Path, p) -> Path:
    """配置里的路径解成真实路径：绝对路径照用，相对路径一律相对 trainer/（train.py 在那里跑）。"""
    q = Path(p)
    return q if q.is_absolute() else T / q


def _resolve(T: Path, p, want_dir: bool = False) -> Path | None:
    """同 `_abs`，再看它在不在：找不到返回 None（由调用方给一句中文）。"""
    if not isinstance(p, str) or not p.strip():
        return None
    c = _abs(T, p)
    return c if (c.is_dir() if want_dir else c.is_file()) else None


def variant_recipe_name(knob, on) -> str:
    """变体配方文件名：关掉默认开着的是 `no-eq.json`，打开默认关着的是 `high_snr-on.json`。"""
    if not isinstance(knob, str) or not knob:
        raise ValueError("变体配方要指定旋钮名")
    return f"{knob}-on.json" if on else f"no-{knob}.json"


# ---------- 结构比较（唯一正本）----------

_MISSING = object()  # 缺键的记号：「键不在」和「键在、值是 null」是两件事


def _flat_strict(d: dict, prefix: str = "") -> dict:
    """把配置 / 配方拍平成「段.键」，用于**结构比较**：
    - 空的对象、列表也留成一个叶子（`noise_pools: {}` 和「没有这个键」不是一回事）；
    - 缺键在比的时候取到 `_MISSING`，和「键在、值是 null」分开。
    正本在这里：judge 比运行配置 · problems 比变体配方都用它。（`formal._flat` 是另一件事：
    它把空对象丢掉，别处依赖它的行为，不改它，也不能拿它比结构。）"""
    out = {}
    for k, v in d.items():
        key = prefix + k
        if isinstance(v, dict) and v:
            out.update(_flat_strict(v, key + "."))
        else:
            out[key] = v
    return out


def _flat_diff(a: dict, b: dict) -> dict:
    """两份**已拍平**的结构不同的键：{键: (a 的值, b 的值)}，缺键记作 `_MISSING`。
    键集合不一致也算不同（拍平后看一眼那边没有就是缺键）。"""
    return {k: (a.get(k, _MISSING), b.get(k, _MISSING))
            for k in sorted(set(a) | set(b)) if a.get(k, _MISSING) != b.get(k, _MISSING)}


def _show(v) -> str:
    """给原因行用：缺键就说「缺这个键」，不能写成 None（那会像是现值就是 null）。"""
    return "缺这个键" if v is _MISSING else repr(v)


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
    base_flat = _flat_strict(base)

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
        vflat = _flat_strict({k: v for k, v in var.items() if k != "_ablation"})
        diff = _flat_diff(vflat, base_flat)
        if sorted(diff) != [want_path]:
            ps.append(f"变体配方 {fname} 应该只在 {want_path} 一处与基线配方不同，"
                      f"实际不同在 {sorted(diff) or '没有一处'}：变体只能改这一个开关")
        elif diff[want_path][0] != on:
            ps.append(f"变体配方 {fname} 的 {want_path} 是 {diff[want_path][0]!r}，运行 {name} 要 {on!r}")
    return ps


def _build_pin_problems(runs: dict, cfg, T: Path, builds_dir) -> list[str]:
    """数据 build 的登记核对：基线配方与 5 份变体配方都要各自有一份合格的登记（判法在 buildpin.py）。
    缺文件 / 写错的 knob 已经由 `_variant_problems` 报过，这里只管「文件在、登记记录对不上」。"""
    bid = runs.get("data_build")
    ps = buildpin.build_problems(_abs(T, cfg.data.recipe), bid, builds_dir)
    if buildpin.data_build_problem(bid) is not None or not isinstance(runs.get("variants_dir"), str):
        return ps  # build id 本身不可用 / 没写变体目录：上面已经报过，变体无从对上
    vp = _abs(T, runs["variants_dir"])
    for run in runs.get("runs") or []:
        if not isinstance(run, dict) or run.get("knob") is None:
            continue
        knob, on = run.get("knob"), run.get("on")
        if not isinstance(knob, str) or not knob or not isinstance(on, bool):
            continue
        f = vp / variant_recipe_name(knob, on)
        if f.is_file():
            ps.extend(buildpin.variant_build_problems(f, bid, builds_dir))
    return ps


def problems(runs_path, builds_dir=None) -> list[str]:
    """开训会拦下来的一切问题，每条一句中文。空列表 = 可以开训。

    builds_dir：登记记录的目录（默认 data/loader.py 的 BUILDS）；测试把假登记记录放别处时用它。
    """
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
            todo = dict(formal._todo(cfg))
            for k, v in todo.items():
                ps.append(f"配置 {k} 还是占位 `{v}`：开训前必须填成实际路径")
            for k in formal._blank(cfg):
                ps.append(f"配置 {k} 空着：开训前必须填成实际路径")
            ps.extend(_variant_problems(runs, cfg, _T_dir(p)))
            if "data.recipe" not in todo and cfg.data.recipe:
                # 配方是 TODO / 空着时上面已经报过：那时核对登记记录没有意义
                ps.extend(_build_pin_problems(runs, cfg, _T_dir(p), builds_dir))
    ps.extend(_run_set_problems(runs.get("runs")))
    return ps


# ---------- 开训命令 ----------

def commands(runs_path, T_dir: str = "$T", python: str = "$PY", gtcrn: str = "$G",
             builds_dir=None) -> list[list[str]]:
    """按 3 + 3 + 1 分批：每批最多 3 条命令。有问题时抛 ValueError（内容是 problems 的全部条目）。"""
    p = Path(runs_path)
    ps = problems(p, builds_dir)
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


def _flat_cfg(config: dict) -> dict:
    """整份配置按 `_flat_strict` 拍平成「段.键」，只去掉 CFG_IGNORE 那三项（它们本来就各不同）。
    缺的键这里没有键：比的时候由 `_flat_diff` 记成 `_MISSING`，不会和「值是 null」混成一样。"""
    return {k: v for k, v in _flat_strict(config).items() if k not in CFG_IGNORE}


def _rows(path: Path) -> list[dict]:
    """metrics.jsonl 的每一行都要是 JSON 对象：合法 JSON 但不是对象（null / [] / 数字）也算读不了。"""
    out: list[dict] = []
    for i, line in enumerate(path.read_text().splitlines(), 1):
        if not line.strip():
            continue
        row = json.loads(line)
        if not isinstance(row, dict):
            raise TypeError(f"metrics.jsonl 第 {i} 行不是 JSON 对象：{line.strip()[:40]!r}")
        out.append(row)
    return out


def _read_run(run_dir: Path, step: int) -> dict:
    """一个运行的记录：第 step 步的开发份 hybrid · 配置（拍平，去了 CFG_IGNORE 那三项）· 自己配的
    data.recipe · run.seed（单独留一份：它就在 CFG_IGNORE 里，拍平时被滤掉了）· 两个 build id。
    读不出就填 reason。"""
    out = {"dev_hybrid": None, "cfg": None, "recipe": None, "seed": None, "data_build_id": None,
           "dev_build_id": None, "reason": None}
    try:
        rec = json.loads((run_dir / "run.json").read_text())
        config = rec["config"]
        if not isinstance(config, dict):
            raise TypeError("config 不是映射")
        data = config["data"]
        if not isinstance(data, dict):
            raise TypeError("config.data 不是映射")
        run_seg = config.get("run")
        rows = _rows(run_dir / "metrics.jsonl")
    except (OSError, ValueError, KeyError, TypeError) as e:
        out["reason"] = f"读不了运行记录：{type(e).__name__}: {e}"
        return out
    flat = _flat_cfg(config)
    out["cfg"] = flat
    out["recipe"] = data.get("recipe")
    out["seed"] = run_seg.get("seed") if isinstance(run_seg, dict) else None
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


def _design_reason(name: str, r: dict, ref: dict) -> str | None:
    """设计的固定项（#212 评审）：model.variant = WANT_VARIANT · schedule.max_steps = STEP ·
    data.dev_recipe 指明开发份配方且等于基线（ref）的 data.recipe · run.seed 是运行矩阵里给这个名字的
    种子 · data.recipe 是这个名字说的那一份配方。返回第一条不合的原因，合上了返回 None。"""
    flat = r["cfg"]
    v, n = flat.get("model.variant"), flat.get("schedule.max_steps")
    if v != WANT_VARIANT:
        return f"model.variant 是 {v!r}：消融只跑模型 {WANT_VARIANT}，A/B 混进来就分不出旋钮的作用"
    if n != STEP:
        return f"schedule.max_steps 是 {n!r}：消融每个运行 {STEP} 步，比的就是第 {STEP} 步的读数"
    dev = flat.get("data.dev_recipe")
    if not isinstance(dev, str) or not dev.strip():
        return (f"data.dev_recipe 空着：7 个运行的开发份要用基线配方 {ref['recipe']!r} 固定住（空 = 跟随"
                " data.recipe，各运行就会各写一份开发集），读数才可比")
    if dev != ref["recipe"]:
        return (f"data.dev_recipe 是 {dev!r}，和基线 {BASE_RUNS[0]} 的 data.recipe {ref['recipe']!r} 不同："
                "7 个运行必须同用一份开发份配方")
    return _seed_reason(name, r) or _recipe_reason(name, r, ref)


def _seed_reason(name: str, r: dict) -> str | None:
    """run.seed 必须是运行矩阵给这个名字的种子：名字说种子 1、实际跑种子 0，两个基线的差就量不出
    噪声底（floor 会变成 0，所有旋钮都被判「分不出」）。"""
    want = next(s for n, s, _k, _o in WANTED_RUNS if n == name)
    got = r["seed"]
    if isinstance(got, bool) or not isinstance(got, int) or got != want:
        return (f"run.seed 是 {got!r}：{name} 这个运行要跑种子 {want}（种子写在名字里；两条基线跑不同的"
                "种子，差才是噪声底 floor）")
    return None


def _recipe_reason(name: str, r: dict, ref: dict) -> str | None:
    """data.recipe 必须是这个名字说的那一份配方：基线用基线配方（= abl-base-s0 的 data.dev_recipe），
    旋钮运行用自己那一个变体配方（文件名 no-<旋钮>.json / <旋钮>-on.json）。"""
    got = r["recipe"]
    _seed, knob, on = next((s, k, o) for n, s, k, o in WANTED_RUNS if n == name)
    if knob is None:
        want = (ref["cfg"] or {}).get("data.dev_recipe")
        if want is None:
            return None  # 基线自己读不出，连基线配方叫什么都不知道：它的 reason 已经记下了
        if got != want:
            return (f"data.recipe 是 {got!r}：基线运行的训练配方要等于 {BASE_RUNS[0]} 的 data.dev_recipe "
                    f"{want!r}（7 个运行比的是同一份配方上的差别）")
        return None
    want_name = variant_recipe_name(knob, on)
    got_name = Path(got).name if isinstance(got, str) else None
    if got_name != want_name:
        return (f"data.recipe 是 {got!r}（文件名 {got_name!r}），{name} 要的是 {want_name}："
                f"「关掉 {knob}」/「打开 {knob}」这个运行必须用自己那一个变体配方")
    return None


def _verdict(on: bool, d: float, floor: float) -> str:
    """严格不等式；on 是「这个旋钮在变体里被打开了」——默认关着的打开了，判读方向就反过来。"""
    if on:
        return V_TURN_ON if d < -floor else (V_KEEP_OFF if d > floor else V_TIE_OFF)
    return V_KEEP_ON if d > floor else (V_CAND_OFF if d < -floor else V_TIE_ON)


def judge(out_dir, step: int = STEP) -> dict:
    """判 7 个运行：返回 {rule, base, floor, runs, knobs}。任何运行不完整 → 它在 runs 里是 incomplete；
    两条基线里任一条不完整（读数、build id、数据字段、整份配置、设计固定项任一项过不了）→
    base 与 floor 都不算，每个旋钮都 incomplete。"""
    root = Path(out_dir)
    recs = {name: _read_run(root / name, step) for name, _s, _k, _o in WANTED_RUNS}
    ref_name = BASE_RUNS[0]
    ref = recs[ref_name]
    # 设计固定项各自查，不必等参照：“彼此都一样”不等于“照设计跑”（#212 评审）
    for name, r in recs.items():
        if r["reason"] is None:
            r["reason"] = _design_reason(name, r, ref)
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
        # 整份配置逐项相同（含 device · limits · schedule 全段 · run.out_dir · 数据字段等）：
        # 只有 CFG_IGNORE 那三项允许各不同；缺键记作 _MISSING，和「键在、值是 null」不是一回事
        diff = _flat_diff(r["cfg"], ref["cfg"])
        if diff:
            # 全落在 data 里就说是数据字段（它最容易错：开发份规模一变读数就不可比）
            what = "数据字段" if all(k.startswith("data.") for k in diff) else "配置字段"
            r["reason"] = (f"{what}和 {ref_name} 对不上（只有 {'、'.join(CFG_IGNORE)} 可以各不同）："
                           + "，".join(f"{k} {_show(mine)} ≠ {_show(theirs)}"
                                      for k, (mine, theirs) in diff.items()))
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
    ap.add_argument("--builds-dir", default=None,
                    help="登记记录的目录（默认 data/loader.py 的 BUILDS，即 data/builds/）")
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
    ps = problems(runs, a.builds_dir)
    if ps:
        for s in ps:
            print(f"- {s}")
        return 1
    if a.action == "check":
        print("可以开训")
        return 0
    for i, block in enumerate(commands(runs, a.T_dir, a.python, a.gtcrn, a.builds_dir), 1):
        print(f"# 第 {i} 批：{len(block)} 个同时跑")
        for c in block:
            print(c)
        print()
    return 0


if __name__ == "__main__":
    sys.exit(main())
