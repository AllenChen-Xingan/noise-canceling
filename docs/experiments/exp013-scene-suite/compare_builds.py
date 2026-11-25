"""exp013 · 核对两个 build 是不是同一批声音（本机与云上交叉核对用）。

同一份代码 + 同一批素材 + 同一套依赖，出的每段每轨应逐样本相同。build id 只是名字，会进音频路径，
所以不比 clips.jsonl / hashes.txt 的文件哈希，而是按片段 id 逐段比：
- `hash.samples`（样本内容哈希）与 `hash.wav_sha256`（wav 文件哈希）每条轨相等，含各分轨
- 除 `audio`（路径）外的其余字段全部相等
- 两边 manifest 的 `material_index`（素材索引哈希与文件数）、`code_sha256`、`params_sha256` 相等

用法：python compare_builds.py <build 目录 A> <build 目录 B>；一致退出 0，不一致退出 1 并列出前 20 条差异。
"""
import json
import sys
from pathlib import Path


def _recs(d):
    return {r["id"]: r for r in map(json.loads, (Path(d) / "clips.jsonl").read_text().splitlines()) if r}


def diff_builds(a, b):
    """返回差异列表（空 = 一致）。"""
    errs = []
    ma, mb = (json.loads((Path(d) / "manifest.json").read_text()) for d in (a, b))
    for k in ("material_index", "code_sha256", "params_sha256", "base_seed", "clips"):
        if ma.get(k) != mb.get(k):
            errs.append(f"manifest.{k} 不同：{ma.get(k)!r} ≠ {mb.get(k)!r}")
    ra, rb = _recs(a), _recs(b)
    for cid in sorted(set(ra) ^ set(rb)):
        errs.append(f"{cid} 只在 {'A' if cid in ra else 'B'}")
    for cid in sorted(set(ra) & set(rb)):
        x, y = ra[cid], rb[cid]
        for h in ("samples", "wav_sha256"):
            if x["hash"][h] != y["hash"][h]:
                errs.append(f"{cid} hash.{h} 不同")
        rest = lambda r: {k: v for k, v in r.items() if k not in ("audio", "hash")}  # noqa: E731
        if rest(x) != rest(y):
            keys = sorted(k for k in set(rest(x)) | set(rest(y)) if rest(x).get(k) != rest(y).get(k))
            errs.append(f"{cid} 字段不同：{keys}")
    return errs


def main():
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    errs = diff_builds(sys.argv[1], sys.argv[2])
    if errs:
        print(f"不一致 {len(errs)} 处：")
        print("\n".join(errs[:20]))
        sys.exit(1)
    print(f"一致：{len(_recs(sys.argv[1]))} 段逐轨哈希相同，素材 / 代码 / 参数哈希相同")


if __name__ == "__main__":
    main()
