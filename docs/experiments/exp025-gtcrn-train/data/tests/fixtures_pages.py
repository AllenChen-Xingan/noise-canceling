"""冻结的证据页 / 场景页（`evidence-<build>.md` · `scenes-<build>.md`）的共用规则。

起因（traindata 2026-11-26）：第五批把 `recipe.json` 换到 `but-reverbdb-silence-b5`，而第四批那两页是
build `bc0e9e7b566421f0` 的产物。页按旧配方算得出来、按新配方算不出来，人手上的产物又不能原地覆盖
（CLAUDE.md 第五节第 1 条）。所以页与当前配方只有两种关系：

- `builds/` 里有 build 记录的 `recipe_sha256` == 当前 `recipe.json` 的 sha256：这一批的页，`evidence-<id>.md`
  与 `scenes-<id>.md` 都必须在仓里、且逐字节等于重算结果（第二处表示必须机器核：页是生成的，不许手改）。
  **这种 build 存在时不许用 `未建表` 免票**：页该生成了（第五批 codex 第 1 轮 #2：以前只看 `未建表` 非不非空，
  挂一个与这批无关的来源就能把「页该生成了」压下去）
- 一个这样的 build 都没有时，才轮到 `未建表` 免票，且里面**至少要有一条是当前配方真用到的来源**（人声 · RIR ·
  噪声池，名单正本是 `shard.default_sources()`）；否则 `未建表` 里只有别科的来源，说明这一批的 build 该登记了
- 不相等、也不是本批的页：上一批的页，冻结、只核不改——build 目录还在、记录里的配方确实不是当前这份、
  文件 sha256 等于 `PINNED` 里钉住的值

`PINNED` 什么时候改：再改一次配方、旧页变「冻结」的那一次，把它的 sha256 抄进来。只改页不改配方不算
（那种改动会先被「当前批」那条重算抓到）。
"""
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent

# 第四批的页（build bc0e9e7b566421f0）：第五批换了配方，冻结、只核不改
PINNED = {
    "evidence-bc0e9e7b566421f0.md": "b31b791c309e44e692647c30c810a07e3cc52ad6ef4594e633709288a08160e3",
    "scenes-bc0e9e7b566421f0.md": "5bb983a3aa3af58df59fbba8148de88b13122dbe50a817939ea4e1afaf740ce2",
}


def recipe_sha(data_dir=HERE):
    """当前 `recipe.json` 的 sha256（与 build 记录的 `recipe_sha256` 同一算法）。"""
    return hashlib.sha256((Path(data_dir) / "recipe.json").read_bytes()).hexdigest()


def pending(data_dir=HERE):
    """`split_rules.json` 顶层 `未建表`：本机还没切、音频还在云上的来源（见 tests/test_products.py）。"""
    data = json.loads((Path(data_dir) / "split_rules.json").read_text(encoding="utf-8"))
    return dict(data.get("未建表") or {})


def recipe_sources(data_dir=HERE):
    """当前配方真用到的来源（人声 · RIR · 噪声池），名单正本是 `shard.default_sources()`：这里不手抄。

    手抄一份就会漏——第二批加了 CochlScene 时漏过一次（`shard` 里那条注释写着这回事）。延迟导入：本模块也被
    场景页测试导入，别在 import 时连带 numpy。"""
    if str(HERE) not in sys.path:
        sys.path.insert(0, str(HERE))
    import shard

    return shard.default_sources(Path(data_dir) / "recipe.json")


def current_builds(data_dir=HERE):
    """`builds/` 里 `recipe_sha256` == 当前 `recipe.json` 的那些 build（{build id: 记录}）。"""
    data_dir = Path(data_dir)
    want = recipe_sha(data_dir)
    out = {}
    for p in sorted((data_dir / "builds").glob("*/build.json")):
        rec = json.loads(p.read_text(encoding="utf-8"))
        if rec.get("recipe_sha256") == want:
            out[rec.get("build_id") or p.parent.name] = rec
    return out


def missing_pages(prefix, data_dir=HERE):
    """对得上当前配方的 build 里，还没有 `<prefix>-<build id>.md` 的那些 id（按 id 排序，空 = 一页不缺）。"""
    data_dir = Path(data_dir)
    return sorted(bid for bid in current_builds(data_dir)
                  if not (data_dir / f"{prefix}-{bid}.md").is_file())


def pending_exemption(data_dir=HERE):
    """「本批的页还没生成」的免票：返回 (是否免票, 原因)。

    只有当前配方一个 build 都没登记时才能免票，且 `未建表` 里至少要有一条是当前配方真用到的来源（见模块说明）。"""
    builds = current_builds(data_dir)
    if builds:
        return False, (f"当前配方已有 build {sorted(builds)}：它的 evidence-<id>.md 与 scenes-<id>.md 该在仓里，"
                       "`未建表` 免不了这张票")
    used = recipe_sources(data_dir)
    rel = sorted(n for n in pending(data_dir) if n in set(used))
    if rel:
        return True, f"当前配方还没有 build；`未建表` 里有它用到的来源 {rel}"
    return False, (f"当前配方没有任何 build 对上，`未建表` 里也没有它用到的来源"
                   f"（现有 {sorted(pending(data_dir))}，配方用到 {sorted(used)}）：这一批的 build 该登记、"
                   "页该生成了")


def regen(prefix, build_id, data_dir=HERE):
    """按仓里的工具重算这一页的正文（不读仓里那一份），返回字节。

    证据页走 `build_evidence.py --out`（它自己也核那几列）、场景页走 `scene_table.py --build`：页的正本是那两个
    生成器，这里只调它们，不另写一份渲染。生成器退非零就红（页本身不过，等于没生成）。"""
    data_dir = Path(data_dir)
    if prefix == "evidence":
        with tempfile.TemporaryDirectory() as d:
            out = Path(d) / "evidence.md"
            r = subprocess.run([sys.executable, str(data_dir / "build_evidence.py"),
                                "--data-dir", str(data_dir), "--out", str(out)],
                               capture_output=True, text=True)
            if r.returncode != 0:
                raise AssertionError(f"重算证据页失败（{data_dir}）：{r.stderr or r.stdout}")
            return out.read_bytes()
    if prefix == "scenes":
        r = subprocess.run([sys.executable, str(data_dir / "scene_table.py"),
                            "--build", str(data_dir / "builds" / build_id)],
                           capture_output=True, text=True)
        if r.returncode != 0:
            raise AssertionError(f"重算场景页失败（build {build_id}）：{r.stderr or r.stdout}")
        return r.stdout.encode("utf-8")
    raise AssertionError(f"{prefix!r} 不是页前缀：只有 evidence / scenes")


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
