#!/bin/sh
# 在云上机器里跑：装 trainer 的 Python 环境，每一步单独计时（大总管定 2026-11-25，「C」）。
# 做法与现行相同（traindata 的 setup_uv.sh 钉版本 uv → uv venv 3.12.14 → 两份 requirements 一起装），只多了计时与统计，
# 用来查清「装环境慢在哪」：uv 不带 -q，它自己会打「Prepared N packages in X」（下载 + 解包）和
# 「Installed N packages in Y」（从缓存链接或复制进 venv），两段分得开。
#
# 用法：sh setup_env_timed.sh <仓库根> <setup_uv.sh 路径>
# 环境变量：UV_PYTHON_INSTALL_DIR、UV_CACHE_DIR（默认网络盘上的 /workspace/uv-python、/workspace/uv-cache）、
#           VDN_BIN（uv 装在哪，默认 /workspace/bin）、VDN_PY_VERSION（默认 3.12.14）
set -eu
R=$1
SETUP_UV=$2
D="$R/docs/experiments/exp025-gtcrn-train/data"
T="$R/docs/experiments/exp025-gtcrn-train/trainer"
export UV_PYTHON_INSTALL_DIR="${UV_PYTHON_INSTALL_DIR:-/workspace/uv-python}"
export UV_CACHE_DIR="${UV_CACHE_DIR:-/workspace/uv-cache}"
VDN_BIN="${VDN_BIN:-/workspace/bin}"
PYV="${VDN_PY_VERSION:-3.12.14}"
export PATH="$VDN_BIN:$PATH"

step() {  # step <名字> <命令…>：打印「=== <名字> seconds=N」
  name=$1; shift
  t0=$(date +%s)
  "$@"
  echo "=== $name seconds=$(( $(date +%s) - t0 ))"
}

count() {  # count <目录>：文件数与占用，不存在就记 0
  if [ -d "$1" ]; then echo "$(find "$1" -type f | wc -l | tr -d ' ') files $(du -sh "$1" | cut -f1)"; else echo "0 files"; fi
}

echo "--- uv cache before: $(count "$UV_CACHE_DIR")"
step uv-install env VDN_BIN="$VDN_BIN" bash "$SETUP_UV"
uv --version
step venv uv venv --python "$PYV" "$R/.venv"
step pip-data uv pip install --python "$R/.venv/bin/python" -r "$D/requirements.txt"
step pip-both uv pip install --python "$R/.venv/bin/python" -r "$D/requirements.txt" -r "$T/requirements.txt"
echo "--- venv: $(count "$R/.venv")"
echo "--- uv cache after: $(count "$UV_CACHE_DIR")"
echo "--- 挂载：venv 在 $(df -P "$R/.venv" | tail -1 | awk '{print $1}')，缓存在 $(df -P "$UV_CACHE_DIR" | tail -1 | awk '{print $1}')"
