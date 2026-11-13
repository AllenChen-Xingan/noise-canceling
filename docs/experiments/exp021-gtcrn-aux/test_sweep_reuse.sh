#!/usr/bin/env bash
# sweep.sh 复用核对的测试：bash docs/experiments/exp021-gtcrn-aux/test_sweep_reuse.sh（worktree 根目录，不需要名额）
# 在临时「主目录」里造几份 run.json，看 reuse_ok 该复用的复用、该拒的拒。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
M="$tmp/noise_canceling"
mkdir -p "$tmp/work" "$M/_tmp/030/d/a/mix" "$M/_tmp/030/d/n/mix" "$M/raw" "$M/other"
echo '{"name": "raw"}' > "$M/raw/run.json"
echo '{"name": "other"}' > "$M/other/run.json"
sha=$(shasum -a 256 "$M/raw/run.json" | cut -d' ' -f1)
for x in a n; do : > "$M/_tmp/030/d/$x/mix/c1.wav"; : > "$M/_tmp/030/d/$x/mix/c2.wav"; done
cat > "$M/_tmp/030/d/a/run.json" <<J
{"model": {"exp021_remix": {"from": "raw", "from_run_json_sha256": "$sha", "files": 2, "head": "raw.head", "off": false, "open_db": -9.0, "floor_db": -12.0}}}
J
cat > "$M/_tmp/030/d/n/run.json" <<J
{"model": {"exp021_remix": {"from": "raw", "from_run_json_sha256": "$sha", "files": 2, "head": null, "off": true, "open_db": -6.0, "floor_db": -12.0}}}
J
eval "$(sed -n '/^reuse_ok()/,/^}/p' "$here/sweep.sh")"
cd "$tmp/work"
fail=0
expect() { # 期望 名字 命令…
  local want="$1" name="$2"; shift 2
  if "$@"; then got=yes; else got=no; fi
  [ "$got" = "$want" ] && echo "ok   $name" || { echo "FAIL $name（期望 $want，得到 $got）"; fail=1; }
}
src=raw; headarg=raw.head
expect yes "同来源同参数同预热段 → 复用" reuse_ok _tmp/030/d/a -9 -12 0
expect no  "参数不同 → 拒" reuse_ok _tmp/030/d/a -6 -12 0
expect yes "不混回不看预热段 → 复用" reuse_ok _tmp/030/d/n 0 0 1
headarg=
expect no  "没给预热段 → 拒（防补零版混进来）" reuse_ok _tmp/030/d/a -9 -12 0
src=other; headarg=raw.head
expect no  "来源不同 → 拒" reuse_ok _tmp/030/d/a -9 -12 0
expect no  "不混回来源不同 → 拒" reuse_ok _tmp/030/d/n 0 0 1
src=raw; headarg=raw.head
echo '{"name": "raw", "changed": 1}' > "$M/raw/run.json"
expect no  "同路径来源内容变了 → 拒" reuse_ok _tmp/030/d/a -9 -12 0
echo '{"name": "raw"}' > "$M/raw/run.json"
expect yes "来源改回 → 复用" reuse_ok _tmp/030/d/a -9 -12 0
rm "$M/_tmp/030/d/a/mix/c2.wav"
expect no  "档位缺文件 → 拒" reuse_ok _tmp/030/d/a -9 -12 0
exit $fail
