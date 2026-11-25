"""exp013 · 核对两个 build 是不是同一批声音。两种模式：

**逐位模式**（同一平台）：同一份代码 + 同一批素材 + 同一套依赖，每段每轨应逐样本相同。build id 只是名字，会进音频路径，
所以不比 clips.jsonl / hashes.txt 的文件哈希，而是按片段 id 逐段比：
- `hash.samples`（样本内容哈希）与 `hash.wav_sha256`（wav 文件哈希）每条轨相等，含各分轨
- 除 `audio`（路径）外的其余字段全部相等
- 两边 manifest 的素材索引、代码、参数哈希等（[`MANIFEST_KEYS`]）相等

**容差模式**（跨平台，2026-11-25 大总管定）：x86_64 与 arm64 的浮点数学库舍入不同，逐位一致做不到
（见 `crosscheck-20261125.md`）。改为抽几段两边都有的音频逐轨比：样本数相等、逐样本最大绝对差 ≤ [`TOL_ABS`]；
manifest 的素材 / 代码 / 参数哈希仍须逐位相等。阈值的正本是本文件的 `TOL_ABS`。

用法：
  python compare_builds.py <build 目录 A> <build 目录 B>
  python compare_builds.py --tolerance <build A> <音频根 A> <build B> <音频根 B> <片段 id,片段 id,…>
一致退出 0，不一致退出 1 并列出前 20 条差异。音频根 = 清单里相对路径 `_tmp/exp013/<id>/…` 的起点。
"""
import json
import sys
from pathlib import Path

import numpy as np
from scipy.io import wavfile

# 逐样本最大绝对差的上限（约 −180 dBFS）。跨平台实测差最大约 3.6e-15（cloudrun，2026-11-25）
TOL_ABS = 1e-9
MANIFEST_KEYS = ("material_index", "code_sha256", "params_sha256", "base_seed", "clips")


def _recs(d):
    return {r["id"]: r for r in map(json.loads, (Path(d) / "clips.jsonl").read_text().splitlines()) if r}


def _manifest_errs(a, b):
    ma, mb = (json.loads((Path(d) / "manifest.json").read_text()) for d in (a, b))
    return [f"manifest.{k} 不同：{ma.get(k)!r} ≠ {mb.get(k)!r}" for k in MANIFEST_KEYS if ma.get(k) != mb.get(k)]


def diff_builds(a, b):
    """逐位模式。返回差异列表（空 = 一致）。"""
    errs = _manifest_errs(a, b)
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


def _tracks(rec):
    """(轨名, 相对路径)，分轨记成 stems/<实例>。"""
    out = [(k, v) for k, v in rec["audio"].items() if k != "stems"]
    return out + [(f"stems/{k}", v) for k, v in rec["audio"].get("stems", {}).items()]


def _read(p):
    _, x = wavfile.read(str(p))
    return np.asarray(x, np.float64)


def tolerance_check(a, root_a, b, root_b, clips):
    """容差模式。`clips` = 要比的片段 id。返回差异列表（空 = 一致），另把每段每轨的最大差记在 `tolerance_check.last`。"""
    errs = _manifest_errs(a, b)
    ra, rb = _recs(a), _recs(b)
    stats = {}
    for cid in clips:
        if cid not in ra or cid not in rb:
            errs.append(f"{cid} 不在 {'A' if cid not in ra else 'B'} 里")
            continue
        ta, tb = dict(_tracks(ra[cid])), dict(_tracks(rb[cid]))
        for k in sorted(set(ta) ^ set(tb)):
            errs.append(f"{cid} · {k} 只在 {'A' if k in ta else 'B'}")
        for k in sorted(set(ta) & set(tb)):
            x, y = _read(Path(root_a) / ta[k]), _read(Path(root_b) / tb[k])
            if len(x) != len(y):
                errs.append(f"{cid} · {k} 样本数不同：{len(x)} ≠ {len(y)}")
                continue
            d = float(np.max(np.abs(x - y))) if len(x) else 0.0
            stats[(cid, k)] = d
            if not d <= TOL_ABS:
                errs.append(f"{cid} · {k} 逐样本最大差 {d:.3g} > {TOL_ABS:g}")
    tolerance_check.last = stats
    return errs


def main():
    argv = sys.argv[1:]
    if argv[:1] == ["--tolerance"] and len(argv) == 6:
        clips = [c for c in argv[5].split(",") if c]
        errs = tolerance_check(argv[1], argv[2], argv[3], argv[4], clips)
        worst = max(tolerance_check.last.values(), default=0.0)
        if errs:
            print(f"不一致 {len(errs)} 处：")
            print("\n".join(errs[:20]))
            sys.exit(1)
        n = len(tolerance_check.last)
        print(f"一致（容差 {TOL_ABS:g}）：{len(clips)} 段 {n} 条轨，逐样本最大差 {worst:.3g}；素材 / 代码 / 参数哈希相同")
        return
    if len(argv) != 2:
        sys.exit(__doc__)
    errs = diff_builds(argv[0], argv[1])
    if errs:
        print(f"不一致 {len(errs)} 处：")
        print("\n".join(errs[:20]))
        sys.exit(1)
    print(f"一致：{len(_recs(argv[0]))} 段逐轨哈希相同，素材 / 代码 / 参数哈希相同")


if __name__ == "__main__":
    main()
