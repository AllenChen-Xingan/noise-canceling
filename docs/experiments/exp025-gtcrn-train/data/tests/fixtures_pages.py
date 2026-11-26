"""冻结的证据页 / 场景页（`evidence-<build>.md` · `scenes-<build>.md`）的共用规则。

起因（traindata 2026-11-26）：第五批把 `recipe.json` 换到 `but-reverbdb-silence-b5`，而第四批那两页是
build `bc0e9e7b566421f0` 的产物。页按旧配方算得出来、按新配方算不出来，人手上的产物又不能原地覆盖
（CLAUDE.md 第五节第 1 条）。所以页与当前配方只有两种关系：

- build 记录的 `recipe_sha256` == 当前 `recipe.json` 的 sha256：这一批的页，用工具 `--check` 逐字节重算
  （第二处表示必须机器核：页是生成的，不许手改）
- 不相等：上一批的页，冻结、只核不改——build 目录还在、记录里的配方确实不是当前这份、文件 sha256 等于 `PINNED` 里钉住的值
- 一条也做不到（当前配方既没有对应的 build，`split_rules.json` 的 `未建表` 也是空的）：新页该生成了，红

`PINNED` 什么时候改：再改一次配方、旧页变「冻结」的那一次，把它的 sha256 抄进来。只改页不改配方不算
（那种改动会先被「当前批」那条重算抓到）。
"""
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent

# 第四批的页（build bc0e9e7b566421f0）：第五批换了配方，冻结、只核不改
PINNED = {
    "evidence-bc0e9e7b566421f0.md": "b31b791c309e44e692647c30c810a07e3cc52ad6ef4594e633709288a08160e3",
    "scenes-bc0e9e7b566421f0.md": "5bb983a3aa3af58df59fbba8148de88b13122dbe50a817939ea4e1afaf740ce2",
}


def recipe_sha():
    """当前 `recipe.json` 的 sha256（与 build 记录的 `recipe_sha256` 同一算法）。"""
    return hashlib.sha256((HERE / "recipe.json").read_bytes()).hexdigest()


def pending():
    """`split_rules.json` 顶层 `未建表`：这一批的音频还在云上、本机还没切（见 tests/test_products.py）。"""
    data = json.loads((HERE / "split_rules.json").read_text(encoding="utf-8"))
    return dict(data.get("未建表", {}))


def pages(prefix):
    """`evidence-*.md` / `scenes-*.md`；按文件名排序（build id 在文件名里）。"""
    return sorted(HERE.glob(f"{prefix}-*.md"))


def build_id_of(page):
    """页的文件名就是 `<前缀>-<build id>.md`：id 只从文件名取，不另抄一份清单。"""
    parts = page.stem.split("-", 1)
    if len(parts) != 2 or not parts[1]:
        raise AssertionError(f"{page.name} 不是 `<前缀>-<build id>.md` 的形状：认不出它属于哪个 build")
    return parts[1]


def build_of(page):
    """页对应的 build 记录；目录或 `build_id` 对不上就红（页不能凭空挂着）。"""
    bid = build_id_of(page)
    p = HERE / "builds" / bid / "build.json"
    if not p.is_file():
        raise AssertionError(f"{page.name} 说它属于 build {bid}，仓里却没有 {p}：页与 build 记录一起进仓")
    b = json.loads(p.read_text(encoding="utf-8"))
    if b.get("build_id") != bid:
        raise AssertionError(f"{p} 记的 build_id 是 {b.get('build_id')}，与目录名 {bid} 不同")
    return b


def sha256_of(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()
