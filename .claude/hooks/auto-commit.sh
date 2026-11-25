#!/usr/bin/env bash
# PostToolUse 自动提交：工作区相对上次提交改动超过 5 行（增 + 删，含新文件），就全部提交。
# 人定 2026-10-29：「所有改动超过 5 行的都要自动提交，写死在 hook」；
# 提交信息带 `Committed-by: <模型家族> <模型名>`（从本会话记录读出，读不到写 unknown）。
# 只提交，不推送。被 .gitignore 忽略的不会进来。
# 自测：bash .claude/hooks/auto-commit.sh --self-test
set -uo pipefail

THRESHOLD=5
MAX_FILE_BYTES=$((10 * 1024 * 1024))   # 铁律 8：单文件超 10 MB 不进库

# ── 模型 id → 「家族 名字」：claude-opus-5-5 → Claude Opus 5.5 ──
pretty_model() {
  python3 -I -c '
import re, sys
m = sys.argv[1].strip()
m = re.sub(r"\[.*?\]$", "", m)            # 去掉 [1m] 之类后缀
m = re.sub(r"-\d{8}$", "", m)              # 去掉日期后缀
p = m.split("-")
if len(p) >= 3 and p[0] == "claude":
    name = p[1].capitalize()
    ver = ".".join(x for x in p[2:] if x.isdigit())
    print(f"Claude {name} {ver}".strip())
else:
    print(m or "unknown")
' "$1"
}

# ── 只在本项目仓库（含它的 worktree）里提交（人定 2026-11-25「改 hook」）──
# 起因：手下在 resources/gtcrn（第三方克隆）里跑 grep，hook 在那个克隆里提交了 pyc。
# 判法：cwd 所在仓库的公共 git 目录，必须等于本脚本所在仓库的公共 git 目录。
common_dir() {   # $1 = 目录；打印绝对公共 git 目录，不在仓库里则打印空
  local d; d="$(git -C "$1" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" || return 0
  [ -n "$d" ] && (cd "$d" 2>/dev/null && pwd -P)
}
same_project() { # $1 = cwd，$2 = 本项目任一目录；同一项目返回 0
  local a b; a="$(common_dir "$1")"; b="$(common_dir "$2")"
  [ -n "$a" ] && [ -n "$b" ] && [ "$a" = "$b" ]
}
HOOK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"

if [ "${1:-}" = "--self-test" ]; then
  fail=0
  t() { got="$(pretty_model "$1")"; [ "$got" = "$2" ] && echo "  ✓ $1 → $got" || { echo "  ✗ $1 → $got（应为 $2）"; fail=1; }; }
  t claude-opus-5-5 "Claude Opus 5.5"
  t claude-sonnet-4-5-20250929 "Claude Sonnet 4.5"
  t "claude-fable-5-1[1m]" "Claude Fable 5.1"
  t "" "unknown"
  # 仓库归属：本项目、本项目的 worktree 算；嵌在里面的别的克隆、仓库外目录不算
  tmp="$(mktemp -d)"
  git init -q "$tmp/proj" && git -C "$tmp/proj" -c user.email=t@t -c user.name=t commit -q --allow-empty -m init
  git -C "$tmp/proj" worktree add -q -b wt "$tmp/proj-wt" 2>/dev/null
  git init -q "$tmp/proj/resources/other"
  mkdir -p "$tmp/plain"
  s() { if same_project "$1" "$tmp/proj"; then r=yes; else r=no; fi
        [ "$r" = "$2" ] && echo "  ✓ 归属 $3 → $r" || { echo "  ✗ 归属 $3 → $r（应为 $2）"; fail=1; }; }
  s "$tmp/proj" yes "本项目"
  s "$tmp/proj-wt" yes "本项目的 worktree"
  s "$tmp/proj/resources/other" no "嵌在里面的第三方克隆"
  s "$tmp/plain" no "仓库外目录"
  rm -rf "$tmp"
  exit $fail
fi

input="$(cat)"
cwd="$(printf '%s' "$input" | jq -r '.cwd // empty' 2>/dev/null)"
transcript="$(printf '%s' "$input" | jq -r '.transcript_path // empty' 2>/dev/null)"
[ -n "$cwd" ] || cwd="${CLAUDE_PROJECT_DIR:-.}"

git -C "$cwd" rev-parse --is-inside-work-tree >/dev/null 2>&1 || exit 0
same_project "$cwd" "$HOOK_DIR" || exit 0   # 别的仓库（如 resources/ 下的第三方克隆）一律不碰
top="$(git -C "$cwd" rev-parse --show-toplevel)"
gitdir="$(git -C "$top" rev-parse --absolute-git-dir)"
g() { git -C "$top" "$@"; }

# 合并、变基、拣选进行中不动
for f in MERGE_HEAD REBASE_HEAD CHERRY_PICK_HEAD rebase-merge rebase-apply; do
  [ -e "$gitdir/$f" ] && exit 0
done

# 多个会话可能同时改同一仓库：用目录锁串行化，60 秒以上的旧锁视为残留
lock="$gitdir/vdn-autocommit.lock"
for _ in $(seq 1 50); do
  mkdir "$lock" 2>/dev/null && break
  if [ -d "$lock" ] && [ $(( $(date +%s) - $(stat -f %m "$lock" 2>/dev/null || echo 0) )) -gt 60 ]; then
    rmdir "$lock" 2>/dev/null
  fi
  sleep 0.1
done
[ -d "$lock" ] || exit 0
trap 'rmdir "$lock" 2>/dev/null' EXIT

# 用临时索引统计，不动真索引
tmpidx="$(mktemp)"
[ -f "$gitdir/index" ] && cp "$gitdir/index" "$tmpidx" || rm -f "$tmpidx"
GIT_INDEX_FILE="$tmpidx" g add -A 2>/dev/null
if g rev-parse --verify -q HEAD >/dev/null; then base=HEAD; else base="$(g hash-object -t tree /dev/null)"; fi
stats="$(GIT_INDEX_FILE="$tmpidx" g diff --cached --numstat "$base")"
rm -f "$tmpidx"
[ -n "$stats" ] || exit 0

# 二进制文件 numstat 是 "-"，按超阈值算
read -r added deleted nfiles <<<"$(printf '%s\n' "$stats" | awk '{a+=($1=="-")?6:$1; d+=($2=="-")?0:$2; n++} END{print a+0, d+0, n+0}')"
total=$((added + deleted))
[ "$total" -gt "$THRESHOLD" ] || exit 0

g add -A

# 守门 1：超大文件撤出暂存
big=""
while IFS= read -r f; do
  [ -f "$top/$f" ] || continue
  sz=$(stat -f %z "$top/$f" 2>/dev/null || echo 0)
  if [ "$sz" -gt "$MAX_FILE_BYTES" ]; then g reset -q -- "$f"; big="$big $f"; fi
done < <(g diff --cached --name-only)

# 守门 2：扫密钥，命中就整次不提交
if command -v gitleaks >/dev/null 2>&1; then
  if ! (cd "$top" && gitleaks git --staged --no-banner --redact >/dev/null 2>&1); then
    g reset -q
    msg="auto-commit 拦下：gitleaks 在待提交内容里发现疑似密钥，本次没有提交。请检查后再改。"
    jq -n --arg m "$msg" '{hookSpecificOutput:{hookEventName:"PostToolUse",additionalContext:$m}}'
    exit 0
  fi
fi

g diff --cached --quiet && exit 0

# 模型：取会话记录里最后一条带 model 的助手消息
model_id=""
if [ -n "$transcript" ] && [ -f "$transcript" ]; then
  model_id="$(tail -n 400 "$transcript" | grep -o '"model":"claude-[^"]*"' | tail -1 | sed 's/"model":"//; s/"$//')"
fi
model_pretty="$(pretty_model "${model_id:-}")"

files="$(g diff --cached --name-only)"
nstaged="$(printf '%s\n' "$files" | grep -c .)"
first="$(printf '%s\n' "$files" | head -1)"
if [ "$nstaged" -gt 1 ]; then subj="chore(auto): ${first} 等 ${nstaged} 个文件（+${added} -${deleted}）"
else subj="chore(auto): ${first}（+${added} -${deleted}）"; fi

msgfile="$(mktemp)"
{
  echo "$subj"
  echo
  printf '%s\n' "$files" | head -20 | sed 's/^/- /'
  [ "$nstaged" -gt 20 ] && echo "- …"
  echo
  echo "Change-By:      ai"
  echo "Agent:          Claude Code"
  echo "Model:          ${model_id:-unknown}"
  echo "Committed-by:   ${model_pretty}"
} > "$msgfile"

if g commit -q --no-verify -F "$msgfile" >/dev/null 2>&1; then
  hash="$(g rev-parse --short HEAD)"
  msg="auto-commit ${hash}：${nstaged} 个文件（+${added} -${deleted}），Committed-by ${model_pretty}"
  [ -n "$big" ] && msg="$msg。超过 10 MB 没进库：$big"
  jq -n --arg m "$msg" '{hookSpecificOutput:{hookEventName:"PostToolUse",additionalContext:$m}}'
fi
rm -f "$msgfile"
exit 0
