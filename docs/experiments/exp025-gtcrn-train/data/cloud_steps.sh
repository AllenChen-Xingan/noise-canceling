#!/usr/bin/env bash
# exp025 数据线 · 云上步骤（RunPod H100，网络盘挂在 /workspace；D96 · D102）。只在云上跑，本机不下数据（D88）。
# 用法：bash cloud_steps.sh <步骤>，步骤按顺序：setup · test · head · fetch · fetch-e3 · split · shard · register · rebuild
# fetch 之前要大总管放行（2026-11-24 交代：先不跑，等 cloudrun 试开完）。
set -euo pipefail
ROOT=${VDN_ROOT:-/workspace}
HERE=$(cd "$(dirname "$0")" && pwd)
REPO=$(cd "$HERE/../../../.." && pwd)
PY="$REPO/.venv/bin/python"
SHARDS=${VDN_SHARDS:-$ROOT/shards/first-batch}
case "${1:-}" in
  setup)  # 装同版本 python 与依赖（uv 没有就先装）
    command -v uv >/dev/null || pip install -q uv
    uv venv -q --python 3.12.14 "$REPO/.venv"
    uv pip install -q --python "$PY" -r "$HERE/requirements.txt" ;;
  test)   # 单元测试 + GPU 对拍（有 CUDA 时对拍在 GPU 上跑）
    cd "$HERE" && "$PY" -m unittest discover -s tests -t . ;;
  head)   # 只核服务器报的大小，不下载
    "$PY" "$HERE/fetch.py" --head --only A1,A2,A3,A4,A5,E1,E2,E3 ;;
  fetch)  # 第一批 A1–A5（约 18.2 GB 下载，解压峰值约 37 GB）+ DEMAND（训练用其中 6 个环境前 60%，E2）
    "$PY" "$HERE/fetch.py" --root "$ROOT" --only A1,A2,A3,A4,A5,E2 ;;
  fetch-e3)  # 单独补下 E3（Wikimedia 上的 3 个 exp013 原料，共约 12 MB）：串行、每次请求前停 5 s、429 按 Retry-After 等
    "$PY" "$HERE/fetch.py" --root "$ROOT" --only E3 ;;
  split)  # 先看清单（QUT / TUM 的目录结构未核），再切；切分表写进仓里 splits/
    "$PY" "$HERE/split.py" --root "$ROOT" --inventory
    "$PY" "$HERE/split.py" --root "$ROOT" ;;
  shard)
    "$PY" "$HERE/shard.py" --root "$ROOT" --out "$SHARDS" ;;
  register)  # 交集检查 + 登记训练 build（builds/<id>/build.json 进仓）
    "$PY" "$HERE/loader.py" --register --shards "$SHARDS" ;;
  rebuild)   # exp013 云上重建（014 第 0 步 ③），走官方源重下原料
    "$PY" "$HERE/rebuild_exp013.py" --root "$ROOT" --fetch --workers "$(nproc)" ;;
  *) echo "用法：$0 setup|test|head|fetch|fetch-e3|split|shard|register|rebuild"; exit 2 ;;
esac
