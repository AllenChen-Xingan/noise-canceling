"""开训前核对数据 build 的登记记录（formal.py · ablation.py 共用；正本是 data/loader.py）。

分片和配方都在，还不够：loader 只认**登记过的** build。它启动时要求仓里 `builds/<build_id>/build.json`
存在、交集检查通过；而 build_id 是由分片索引、切分表、分片、配方、数据代码的 sha256 算出来的——配方或
数据代码在登记之后改过一个字节，算出来的就是另一个 id，loader 就会拒绝。本文件把这几条写成机器能判的
清单，每条一句中文。公式与名字一律从 loader 取（`sha_bytes` · `build_id_of` · `CODE` · `EXP013_CODE` ·
`BUILDS` · `HERE` · `EXP013`），这里不另抄一份。
"""
from __future__ import annotations

import json
import re
from pathlib import Path

ID_RE = re.compile(r"^[0-9a-f]{16}$")  # build_id = loader.build_id_of 取 sha256 的前 16 位（十六进制）


def loader_module():
    """traindata 的 data/loader.py 模块（延迟导入：它连带 numpy · soundfile，本文件被 import 时不必付这笔）。

    入口与 T/data.py 相同（`data.import_loader`），拿到的是同一份 sys.modules 缓存。
    """
    from data import import_loader
    return import_loader()


def default_builds_dir() -> Path:
    """登记记录的默认目录：loader.BUILDS（= data/builds/，随仓走）。"""
    return Path(loader_module().BUILDS)


def data_build_problem(build_id) -> str | None:
    """运行矩阵里的 `data_build` 本身的问题（缺 · 空 · 不是 16 位十六进制）；没问题返回 None。"""
    if build_id is None or (isinstance(build_id, str) and not build_id.strip()):
        return ("运行矩阵没写 data_build：要写训练数据用的 build id（16 位十六进制），"
                "登记记录的正本是 data/builds/<id>/build.json")
    if not isinstance(build_id, str) or not ID_RE.match(build_id):
        return (f"运行矩阵的 data_build 是 {build_id!r}，不是 16 位十六进制："
                "build id 是 loader 把身份字段算成 sha256 后取前 16 位（例如 bc0e9e7b566421f0）")
    return None


def code_sha256(mod=None) -> dict:
    """当前数据代码的 sha256：公式与 loader.build_info 里的那一份相同（含 `exp013/` 前缀的模块）。"""
    m = mod or loader_module()
    return {**{f: m.sha_bytes((m.HERE / f).read_bytes()) for f in m.CODE},
            **{f"exp013/{f}": m.sha_bytes((m.EXP013 / f).read_bytes()) for f in m.EXP013_CODE}}


def _read_record(builds_dir: Path, build_id: str) -> tuple[dict | None, str | None]:
    """读 `builds_dir/<build_id>/build.json`。返回 (记录, None)，读不出来返回 (None, 一句原因)。

    原因不带主语（是谁的登记由 `_record_fault` 拼上）：同一个读法要给基线 build、变体的期望 id 两处用。
    """
    rec_path = builds_dir / build_id / "build.json"
    try:
        rec = json.loads(rec_path.read_text())
    except FileNotFoundError:
        return None, (f"没登记：{rec_path} 不存在——先按 data/loader.py 的 register_build "
                      "把这份分片与配方登记进仓（记录是开训的前提）")
    except (OSError, ValueError) as e:
        return None, (f"的登记记录 {rec_path} 读不了（{type(e).__name__}: {e}）——"
                      "用 register_build 重新登记一份")
    if not isinstance(rec, dict):
        return None, (f"的登记记录 {rec_path} 不是 JSON 对象：用 register_build 重新登记一份")
    return rec, None


def _record_fault(builds_dir: Path, build_id: str, who: str) -> tuple[dict | None, str | None]:
    """`who` 在 builds_dir/<build_id> 的那份登记可不可信：在 · valid 是 true · 记录自己对得上。

    返回 (记录, None) 或 (None, 一句中文原因)。`who` 是一句话的主语，由调用方给（「数据 build X」
    「变体配方 Y（应登记成 build Z）」）：同一个判法要给基线和变体两处用，不许各写一份。
    """
    rec, why = _read_record(builds_dir, build_id)
    if why is not None:
        return None, f"{who} {why}"
    rec_path = builds_dir / build_id / "build.json"
    if rec.get("valid") is not True:
        return None, (f"{who} 的登记记录 {rec_path} 写着 valid = {rec.get('valid')!r}："
                      "交集检查没过（作废的 build），这批分片不能用来训练")
    said, calc = rec.get("build_id"), loader_module().build_id_of(rec)
    if said != build_id or calc != build_id:
        return None, (f"{who} 的登记记录 {rec_path} 自己对不上：记录里的 build_id 是 {said!r}，"
                      f"按记录内容算出来是 {calc!r}，目录名是 {build_id}——三个必须相同"
                      "（记录被手改过，或者放错了目录）")
    return rec, None


def build_problems(recipe_path, build_id, builds_dir=None) -> list[str]:
    """这份配方在这个 build 上开训前的登记问题，每条一句中文。

    空列表 = 记录在、交集检查通过、配方与数据代码的 sha256 都和登记时一模一样。
    结构上的问题（没登记 · valid 不是 true · 记录自己对不上）先报一条就停：记录本身不可信时，
    拿它的 sha256 去比没有意义。`builds_dir` 默认 loader.BUILDS。
    """
    bad = data_build_problem(build_id)
    if bad is not None:
        return [bad]
    bdir = Path(builds_dir) if builds_dir is not None else default_builds_dir()
    rec, why = _record_fault(bdir, build_id, f"数据 build {build_id}")
    if why is not None:
        return [why]
    rec_path = bdir / build_id / "build.json"
    ps: list[str] = []
    rp = Path(recipe_path)
    try:
        rsha = loader_module().sha_bytes(rp.read_bytes())
    except OSError as e:
        ps.append(f"读不了配方 {rp}（{type(e).__name__}: {e}）：要读到配方文件的字节才算 sha256，"
                  f"才能和登记记录 {rec_path} 对（配置里的相对路径按 trainer/ 解）")
        return ps
    if rsha != rec.get("recipe_sha256"):
        ps.append(f"配方对不上数据 build {build_id}：{rp} 的 sha256 是 {rsha[:12]}…，登记记录 {rec_path} "
                  f"里记的是 {str(rec.get('recipe_sha256'))[:12]}…（配错了配方，或者配方在登记之后改过）")
    cur, want = code_sha256(), rec.get("code_sha256") or {}
    if cur != want:
        diff = sorted(k for k in set(cur) | set(want) if cur.get(k) != want.get(k))
        ps.append(f"数据 build {build_id} 的数据代码在登记之后改过：{', '.join(diff)} 的 sha256 和登记记录 "
                  f"{rec_path} 里记的不同——loader 启动时会拒绝这个 build：要么把本地的数据代码还原，"
                  "要么按现在的代码重新登记")
    return ps


def variant_build_problems(variant_path, base_build_id, builds_dir=None) -> list[str]:
    """变体配方要**自己**有一份合格的登记：**loader 现在会算出来的那个 id** 上得有一份 valid、自己一致的记录。

    预测 id 的办法只有一种：拿基线的身份字段（分片索引、切分表、分片、数据代码），把配方换成这份变体的
    sha256，再用 loader.build_id_of 算一遍——这就是 loader 开训时算出来的那个。只比「配方与 index 相同」
    不够：数据代码改过、只重登记了基线 build 时，旧变体记录照样配方与 index 都对，但算出来是另一个 id，
    loader 会拒绝，所以这里必须精确预测 id，不能扫目录找「看起来对」的记录。

    基线的数据代码是否还是现在这份，由 build_problems（基线 build 自己的记录）已判过；这里只管变体那一份。
    基线记录读不出来时不报（build_problems 已经报过那一条，这里没有身份字段可取）。
    """
    bdir = Path(builds_dir) if builds_dir is not None else default_builds_dir()
    base, _why = _read_record(bdir, str(base_build_id))
    if base is None:
        return []
    vp = Path(variant_path)
    try:
        vsha = loader_module().sha_bytes(vp.read_bytes())
    except OSError as e:
        return [f"读不了变体配方 {vp}（{type(e).__name__}: {e}）：要读到文件的字节才算 sha256"]
    m = loader_module()
    parts = {k: base[k] for k in m.ID_KEYS if k in base}
    parts["recipe_sha256"] = vsha
    want = m.build_id_of(parts)
    who = f"变体配方 {vp.name}（按基线 build {base_build_id} 的分片与现在的数据代码算出来应登记成 build {want}）"
    _rec, why = _record_fault(bdir, want, who)
    return [] if why is None else [why]
