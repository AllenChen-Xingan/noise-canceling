#!/bin/sh
# 在云上机器里跑：按依赖内容复用一份 venv。
# 每台机器的仓库副本自带一份约 8 GB 的 venv，网络盘上重复占地方（大总管定 2026-11-25）。
# 改法：同一套 requirements 只留一份 tar 放网络盘；机器上解包到容器盘自己的目录，机器关机即丢。
#
# 用法：sh remote/venv_cache.sh <requirements.txt> [<requirements.txt> ...]
# 环境变量（括号里是默认值）：
#   VDN_PY_VERSION        （3.12.14）venv 用的 Python 版本
#   VENV_STORE            （/workspace/venvs）网络盘上的 tar 仓库
#   VENV_ROOT             （/root/venvs）容器盘上的解包目录
#   UV_PYTHON_INSTALL_DIR （/workspace/uv-python）uv 自己下载的 Python 放哪
#   UV_CACHE_DIR          （/workspace/uv-cache）uv 的包缓存放哪
# uv 从 PATH 里找。
#
# 输出约定：每步一行「=== <步骤> seconds=<整数>」；最后一行一定是「PY=<venv 里 python 的绝对路径>」。
# 退出码非 0 表示失败；失败时网络盘上只会留 <key>.tar.part-*，不会留半截的 <key>.tar。
set -eu

if [ $# -lt 1 ]; then
  echo "用法：sh remote/venv_cache.sh <requirements.txt> [<requirements.txt> ...]" >&2
  exit 2
fi

PYV="${VDN_PY_VERSION:-3.12.14}"
STORE="${VENV_STORE:-/workspace/venvs}"
ROOT="${VENV_ROOT:-/root/venvs}"
export UV_PYTHON_INSTALL_DIR="${UV_PYTHON_INSTALL_DIR:-/workspace/uv-python}"
export UV_CACHE_DIR="${UV_CACHE_DIR:-/workspace/uv-cache}"

sha_full() {  # sha_full <文件>：整段 sha256 十六进制
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | cut -d' ' -f1
  else
    shasum -a 256 "$1" | cut -d' ' -f1
  fi
}

mkdir -p "$STORE" "$ROOT"
STORE=$(cd "$STORE" && pwd)
ROOT=$(cd "$ROOT" && pwd)

# key = 前 16 位：py 版本 + uv 版本 + 每份 requirements 全文（按给定顺序）
KEYSRC=$(mktemp -t vdnkey)
trap 'rm -f "$KEYSRC"' EXIT
{
  echo "py=$PYV"
  uv --version
  for f in "$@"; do
    cat "$f"
  done
} > "$KEYSRC"
KEY=$(sha_full "$KEYSRC" | cut -c1-16)
TAR="$STORE/$KEY.tar"

now() { date +%s; }

if [ -f "$TAR" ]; then
  # 已有成品：解包，绝不装包
  rm -rf "$ROOT/$KEY"
  t0=$(now)
  tar -xf "$TAR" -C "$ROOT"
  if [ ! -x "$ROOT/$KEY/bin/python" ]; then
    echo "解包后缺 $ROOT/$KEY/bin/python，$TAR 可能坏了" >&2
    exit 4
  fi
  echo "=== restore seconds=$(( $(now) - t0 ))"
else
  t0=$(now)
  uv venv --relocatable --python "$PYV" "$ROOT/$KEY"
  echo "=== venv seconds=$(( $(now) - t0 ))"
  if [ ! -x "$ROOT/$KEY/bin/python" ]; then
    echo "uv venv 之后缺 $ROOT/$KEY/bin/python" >&2
    exit 4
  fi
  i=0
  for f in "$@"; do
    i=$((i + 1))
    t0=$(now)
    # 不带 -q：要 uv 自己打的「Prepared / Installed … in」两段时间
    uv pip install --python "$ROOT/$KEY/bin/python" -r "$f"
    echo "=== pip-$i seconds=$(( $(now) - t0 ))"
  done
  t0=$(now)
  PART="$TAR.part-$$-$(awk 'BEGIN{srand();print int(rand()*1000000)}')"
  tar -cf "$PART" -C "$ROOT" "$KEY"
  SUM="$STORE/$KEY.tar.sha256.part-$$"
  echo "$(sha_full "$PART")  $KEY.tar" > "$SUM"
  if [ -e "$TAR" ]; then
    # 另一台机器抢先发布了同一份：留它的，删自己的
    echo "--- 发布：已有 $KEY.tar，保留网络盘上那份"
    rm -f "$PART" "$SUM"
  else
    mv "$PART" "$TAR"
    mv "$SUM" "$STORE/$KEY.tar.sha256"
    echo "--- 发布：$TAR"
  fi
  echo "=== pack seconds=$(( $(now) - t0 ))"
fi

echo "PY=$ROOT/$KEY/bin/python"
