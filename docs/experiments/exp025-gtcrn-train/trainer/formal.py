"""正式训练（exp025，6 个运行）开训前的检查，以及开训命令的生成。

这是什么：`train.py --config configs/formal.yaml --set ...` 一次训一个运行；6 个运行 = 模型 A、B ×
种子 0、1、2，每批 3 个同时跑一张 H100。本文件把「开训前必须成立的条件」写成机器能判的清单，
免得半填的配置上了机器才发现（一次开机十来分钟、几美元）。

用法（在 trainer/ 目录下）：
    python formal.py check              # 有拦路的问题就逐条列出来，退出码 1；全通过打印「可以开训」，退出码 0
    python formal.py commands           # 按运行矩阵打印每批 3 条命令（每批一块）；有问题时列问题、退出码 1
    python formal.py check 某个-runs.yaml   # 也可以指定别的运行矩阵文件

默认运行矩阵：configs/formal-runs.yaml（正在用的那一份）。条件正本：
- 配置字段与默认值：config.py（本文件只读它，不再手抄默认值）
- 每步 0.39 s：plan-h100-smoke.md 第五节（3 个同时跑的实测）
- 加训场景（target_scenes）人已定（D113）：名字按地点键表查，正本是 specs/schema/eval-suite.schema.json
  里 `$defs.MeetingPlace.enum`（兜底键 other 不是场景，不算）
- 加训哪几个场景由配方定：正本是 data/recipe.json 的 `knobs.target_scene.scene`（loader.load_recipe 校验、
  scene_table.target_scenes 读）。configs/formal-runs.yaml 的 `target_scenes` 是第二处写法，训练代码不读它；
  这里拿它和配方比，缺的、多的各报一条（#213）
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import yaml

import buildpin  # 数据 build 的登记核对（配方与数据代码的 sha256，正本在 data/loader.py）
import config as cfgmod

HERE = Path(__file__).resolve().parent
DEFAULT_RUNS = HERE / "configs" / "formal-runs.yaml"
SCHEMA_REL = Path("specs") / "schema" / "eval-suite.schema.json"  # 地点键表正本
FALLBACK_PLACE = "other"  # 兜底键：不是场景，不进成绩单

VARIANTS = ("A", "B")
SEEDS = (0, 1, 2)
WANTED = {(v, s) for v in VARIANTS for s in SEEDS}  # 恰好 6 个运行
BATCH_RUNS = 3  # 一张 H100 同时跑 3 个（plan-h100-smoke.md 第五节，大总管定 2026-11-25）
RUN_KEYS = ("name", "variant", "seed")
MUST_SET = ("data.shard_root", "data.recipe")  # 空着就不可能有数据可训


def load_runs(path) -> dict:
    """读运行矩阵（base · target_scenes · batches）。base 是相对本文件所在目录的路径。"""
    d = yaml.safe_load(Path(path).read_text()) or {}
    if not isinstance(d, dict):
        raise ValueError(f"{path} 应是映射（base · target_scenes · batches）")
    return d

def _base_path(runs_path, runs: dict) -> Path:
    b = runs.get("base")
    if not isinstance(b, str) or not b.strip():
        raise ValueError("missing base")
    p = Path(b)
    return p if p.is_absolute() else (Path(runs_path).resolve().parent / p).resolve()


def _flat(d, prefix="") -> dict:
    out = {}
    for k, v in d.items():
        if isinstance(v, dict):
            out.update(_flat(v, prefix + k + "."))
        else:
            out[prefix + k] = v
    return out


def _todo(cfg) -> list[tuple[str, str]]:
    """有效配置里仍是 TODO 占位的字符串字段：[(字段名, 值)]。"""
    return [(k, v) for k, v in _flat(cfgmod.to_dict(cfg)).items()
            if isinstance(v, str) and v.strip().startswith("TODO")]


def _blank(cfg) -> list[str]:
    """有效配置里开训前必须有值、现在还是空串的字段。正本在这里（其他地方只引用）。"""
    flat = _flat(cfgmod.to_dict(cfg))
    return [k for k in MUST_SET if flat.get(k) == ""]


def _recipe_path(runs_path, recipe) -> Path:
    """配置里的 data.recipe 解成真实路径：绝对路径照用，相对路径相对 trainer/（train.py 在那里跑，
    与 ablation.py 同一约定）。"""
    q = Path(recipe)
    return q if q.is_absolute() else (Path(runs_path).resolve().parent.parent / q)


def _build_problems(runs: dict, cfg, runs_path, builds_dir) -> list[str]:
    """运行矩阵钉的 data_build 与它的登记记录：配方与数据代码都要和登记时一模一样（判法在 buildpin.py）。"""
    return buildpin.build_problems(_recipe_path(runs_path, cfg.data.recipe), runs.get("data_build"), builds_dir)


def _repo_root() -> Path:
    """从本文件往上找那个含 specs/schema/eval-suite.schema.json 的目录（仓库根）。"""
    for root in (HERE, *HERE.parents):
        if (root / SCHEMA_REL).is_file():
            return root
    raise FileNotFoundError(f"从 {HERE} 往上找不到 {SCHEMA_REL}：找不到地点键表的正本")


def place_keys() -> frozenset[str]:
    """地点键表（= 真实场景名）：从 JSON schema 的 MeetingPlace.enum 读，去掉兜底键 other。

    正本：specs/schema/eval-suite.schema.json → $defs.MeetingPlace.enum。这里不抄一份名单，
    以免地点键表改名后两边不一致。
    """
    f = _repo_root() / SCHEMA_REL
    try:
        d = json.loads(f.read_text())
    except (OSError, json.JSONDecodeError) as e:
        raise ValueError(f"读不了地点键表 {f}：{e}") from e
    try:
        enum = d["$defs"]["MeetingPlace"]["enum"]
    except (KeyError, TypeError) as e:
        raise ValueError(f"{f} 里没有 $defs.MeetingPlace.enum：地点键表的正本不见了") from e
    keys = frozenset(k for k in enum if isinstance(k, str)) - {FALLBACK_PLACE}
    if not keys:
        raise ValueError(f"{f} 里 $defs.MeetingPlace.enum 去掉 {FALLBACK_PLACE} 后一个场景都不剩")
    return keys


def _scene_problems(scenes) -> list[str]:
    """target_scenes 的逐项问题：不是字符串 · 不在地点键表里 · 兜底键 other · 重复。"""
    keys = place_keys()
    ps: list[str] = []
    seen: list = []
    for item in scenes:
        if not isinstance(item, str):
            ps.append(f"target_scenes 里有一项 {item!r} 不是字符串：场景名要写地点键，"
                      f"名单见 {SCHEMA_REL.as_posix()}")
            continue
        if item == FALLBACK_PLACE:
            ps.append(f"target_scenes 里的 {item} 是兜底键、不是场景：要训哪个场景就写哪个地点键，"
                      f"名单见 {SCHEMA_REL.as_posix()}")
            continue
        if item not in keys:
            ps.append(f"target_scenes 里的 {item} 不在地点键表里：名字要照 "
                      f"{SCHEMA_REL.as_posix()} 的 $defs.MeetingPlace.enum 写")
            continue
        if item in seen:
            ps.append(f"target_scenes 里的 {item} 重复了：每个场景只写一次，"
                      f"重复会让同一个场景进两次加权")
            continue
        seen.append(item)
    return ps


def _scene_table_module():
    """traindata 的 data/scene_table.py：读 `knobs.target_scene.scene` 的唯一入口（它自己在导入时把 data/ 放进
    sys.path）。延迟导入：它连带的模块只在真的要比对场景时才付这笔。"""
    d = str(buildpin.loader_module().HERE)
    if d not in sys.path:
        sys.path.insert(0, d)
    import scene_table  # 函数内导入：只在真的要比对场景时才付它的导入成本
    return scene_table


def _scene_src_problems(runs, cfg, runs_path) -> list[str]:
    """`target_scenes` 与配方 `knobs.target_scene.scene` 必须一致（#213）。

    两边都不是本文件的正本：配方是加训场景的唯一真相源（traindata 定），运行矩阵里这份是第二处写法、
    训练代码也不读它。所以按集合比：配方有而这里没写、这里写了而配方没有，各报一条。
    """
    want = runs.get("target_scenes")
    if not isinstance(want, list):
        return []  # 不是清单 / 空着已经由 problems 里那几条报过，这里没有可比的集合
    rp = _recipe_path(runs_path, cfg.data.recipe)
    try:
        recipe = buildpin.loader_module().load_recipe(rp)
    except OSError as e:
        return [f"target_scenes 没法比对：读不了配方 {rp}（{type(e).__name__}: {e}）"]
    except (ValueError, KeyError, TypeError) as e:
        return [f"target_scenes 没法比对：{rp} 不是一份合格的混音配方（{type(e).__name__}: {e}）"]
    ts = recipe["knobs"]["target_scene"]
    if ts.get("on") is not True:
        return [f"配方 {rp} 的 knobs.target_scene.on 是 {ts.get('on')!r}、不是 true：开关关着就没有加训场景，"
                f"target_scenes 不能比配方多要场景（两处要一致）"]
    try:
        got = _scene_table_module().target_scenes(recipe)
    except SystemExit as e:
        return [f"配方 {rp} 没写目标场景（{e}）——加训哪几个场景的正本是配方 knobs.target_scene.scene，"
                f"不能空着，target_scenes 要写的正是它"]
    only_runs = sorted({s for s in want if isinstance(s, str)} - set(got))
    only_recipe = sorted(set(got) - {s for s in want if isinstance(s, str)})
    ps: list[str] = []
    if only_runs:
        ps.append(f"运行矩阵的 target_scenes 多要了 {only_runs}：加训哪几个场景的正本是配方 "
                  f"knobs.target_scene.scene（{rp.name}），两处必须一致——要么改配方，要么把 target_scenes "
                  "改成配方里的那几个")
    if only_recipe:
        ps.append(f"配方 knobs.target_scene.scene 里的 {only_recipe} 没写进运行矩阵的 target_scenes"
                  f"（{Path(runs_path).name}）：两处必须一致——要么补齐 target_scenes，要么改配方")
    return ps


def _matrix_problems(batches) -> list[str]:
    """运行矩阵的问题：每批恰好 3 个、名字唯一、合起来恰好 A/B × 种子 0/1/2。"""
    if not isinstance(batches, list) or not batches:
        return ["运行矩阵 batches 空着：要 A、B 各 3 个种子、共 6 个运行，每批 3 个"]
    ps: list[str] = []
    seen: set = set()
    pairs: set = set()
    for bi, batch in enumerate(batches, 1):
        if not isinstance(batch, list) or len(batch) != BATCH_RUNS:
            n = len(batch) if isinstance(batch, list) else 0
            ps.append(f"第 {bi} 批有 {n} 个运行，不是 {BATCH_RUNS} 个：一张卡同时跑 {BATCH_RUNS} 个，"
                      f"请把这一批拆开或补齐")
        for run in batch if isinstance(batch, list) else []:
            if not isinstance(run, dict):
                ps.append(f"第 {bi} 批里有一项不是运行：每一项都要有 name、variant、seed 三个字段")
                continue
            missing = [k for k in RUN_KEYS if k not in run]
            if missing:
                ps.append(f"第 {bi} 批有一个运行缺 {missing[0]}：每一项都要写 name、variant、seed")
            name = run.get("name")
            if name in seen:
                ps.append(f"运行名 {name} 重复：6 个运行的名字必须各不相同，不然两个运行写同一个目录")
            seen.add(name)
            pairs.add((run.get("variant"), run.get("seed")))
    if pairs != WANTED:
        lack = sorted(WANTED - pairs, key=str)
        extra = sorted(pairs - WANTED, key=str)
        parts = []
        if lack:
            parts.append(f"缺 {lack}")
        if extra:
            parts.append(f"多了 {extra}")
        ps.append(f"运行矩阵不是恰好 A、B 各 3 个种子（共 6 个）：{'，'.join(parts)}")
    return ps


def problems(runs_path, builds_dir=None) -> list[str]:
    """开训会拦下来的一切问题，每条一句中文。空列表 = 可以开训。

    builds_dir：登记记录的目录（默认 data/loader.py 的 BUILDS）；测试把假登记记录放别处时用它。
    """
    p = Path(runs_path)
    ps: list[str] = []
    try:
        runs = load_runs(p)
    except (OSError, ValueError) as e:
        return [f"读不了运行矩阵 {p}：{e}"]

    cfg = None
    base = None
    try:
        base = _base_path(p, runs)
    except ValueError:
        ps.append("运行矩阵没写 base（或它不是路径）：base 要指向正式配置，例如 formal.yaml")
    if base is not None:
        try:
            cfg = cfgmod.load(base)
        except FileNotFoundError:
            ps.append(f"运行矩阵的 base 指向的文件不存在：{runs.get('base')}")
        except ValueError as e:
            ps.append(f"base 指向的配置本身不合法：{e}")
        else:
            todo = dict(_todo(cfg))
            for k, v in todo.items():
                ps.append(f"配置 {k} 还是占位 `{v}`：开训前必须填成实际路径")
            for k in _blank(cfg):
                ps.append(f"配置 {k} 空着：开训前必须填成实际路径")
            if "data.recipe" not in todo and cfg.data.recipe:
                # 配方是什么还不知道（TODO / 空着）时上面已经报过：这时核对登记记录没有意义
                ps.extend(_build_problems(runs, cfg, p, builds_dir))
                # 同一份配方也是 target_scenes 的正本（#213）：一起比，免得配方里少了场景还过 check
                ps.extend(_scene_src_problems(runs, cfg, p))

    if not (runs.get("target_scenes") or []):
        ps.append("target_scenes 还空着：要写明加训哪几个场景（人已定，见 D113），"
                  f"名字按 {SCHEMA_REL.as_posix()} 的地点键表写，空着就不许开训")
    elif isinstance(runs.get("target_scenes"), list):
        ps.extend(_scene_problems(runs["target_scenes"]))
    else:
        ps.append("target_scenes 不是清单：要写成场景名清单，例如 [rail-air-cabin, canteen]")

    ps.extend(_matrix_problems(runs.get("batches")))

    if cfg is not None and cfg.limits.expect_s_step is None:
        ps.append("limits.expect_s_step 没设：3 个同时跑时就没法判「每步太慢」，机器会白烧钱")
    return ps


def _cfg_arg(runs_path, base: Path) -> str:
    """命令里的 --config 路径：相对 trainer/ 目录（运行矩阵在 trainer/configs/ 下），否则给绝对路径。"""
    root = Path(runs_path).resolve().parent.parent
    try:
        return base.relative_to(root).as_posix()
    except ValueError:
        return str(base)


def commands(runs_path, T_dir: str = "$T", python: str = "$PY", gtcrn: str = "$G",
             builds_dir=None) -> list[list[str]]:
    """每批 3 条命令。有问题时抛 ValueError（内容是 problems 的全部条目）。"""
    p = Path(runs_path)
    ps = problems(p, builds_dir)
    if ps:
        raise ValueError("还不能开训：" + "；".join(ps))
    runs = load_runs(p)
    cfg_arg = _cfg_arg(p, _base_path(p, runs))
    return [[f"cd {T_dir} && VDN_GTCRN_DIR={gtcrn} {python} train.py --config {cfg_arg} "
             f"--set run.name={run['name']} --set model.variant={run['variant']} --set run.seed={run['seed']}"
             for run in batch] for batch in runs["batches"]]


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description="正式训练：开训前检查 / 生成开训命令")
    ap.add_argument("action", choices=("check", "commands"))
    ap.add_argument("runs", nargs="?", default=str(DEFAULT_RUNS), help="运行矩阵文件（默认 configs/formal-runs.yaml）")
    ap.add_argument("--T-dir", default="$T", help="生成命令里 trainer/ 的位置")
    ap.add_argument("--python", default="$PY", help="生成命令里的 python")
    ap.add_argument("--gtcrn", default="$G", help="生成命令里的 VDN_GTCRN_DIR（GTCRN 起点权重目录）")
    ap.add_argument("--builds-dir", default=None,
                    help="登记记录的目录（默认 data/loader.py 的 BUILDS，即 data/builds/）")
    a = ap.parse_args(argv)

    ps = problems(a.runs, a.builds_dir)
    if a.action == "check":
        if ps:
            for s in ps:
                print(f"- {s}")
            return 1
        print("可以开训")
        return 0
    if ps:
        for s in ps:
            print(f"- {s}")
        return 1
    for i, block in enumerate(commands(a.runs, a.T_dir, a.python, a.gtcrn, a.builds_dir), 1):
        print(f"# 第 {i} 批：{len(block)} 个同时跑")
        for c in block:
            print(c)
        print()
    return 0


if __name__ == "__main__":
    sys.exit(main())
