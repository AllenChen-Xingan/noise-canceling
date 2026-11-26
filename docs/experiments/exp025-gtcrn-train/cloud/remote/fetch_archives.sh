#!/bin/sh
# 在云上机器里跑：把一批数据文件从 URL 下到网络盘，边下边记校验。
# 用法：sh remote/fetch_archives.sh <list file> <dest dir>
# 清单文件：一行一个文件，三列「名字 md5 地址」，空行与 # 开头的行忽略。
# 环境变量（括号里是默认值）：
#   STOP_GB （185）网络盘用量到这里就停，不再下载
#   DU_ROOT （/workspace）用量统计的根目录
#
# 这个脚本只下载和记账，**从不删东西**：删不删是人的事。
# 每个文件立刻把「名字 md5 字节数 ok|BAD UTC时间」追加进 <dest>/checksums.txt，这份文件是耐久记录。
set -eu

if [ $# -lt 2 ]; then
  echo "用法：sh remote/fetch_archives.sh <list file> <dest dir>" >&2
  exit 2
fi

LIST=$1
DEST=$2
STOP_GB="${STOP_GB:-185}"
DU_ROOT="${DU_ROOT:-/workspace}"

md5_of() {  # md5_of <文件>
  if command -v md5sum >/dev/null 2>&1; then
    md5sum "$1" | cut -d' ' -f1
  else
    md5 -q "$1"
  fi
}

bytes_of() {  # bytes_of <文件>
  wc -c < "$1" | tr -d ' '
}

now_utc() { date -u +%Y-%m-%dT%H:%M:%SZ; }
now_s() { date +%s; }

disk_gb() {  # 网络盘用量，整数 GB
  du -s --block-size=1G "$DU_ROOT" 2>/dev/null | cut -f1 | tr -d ' '
}

mkdir -p "$DEST"
RECORD="$DEST/checksums.txt"

while IFS= read -r line || [ -n "$line" ]; do
  case "$line" in
    '' | '#'*) continue ;;
  esac
  # 拆三列：名字 · md5 · 地址（多余空白忽略）
  old_ifs=$IFS
  IFS=' 	'
  set -f
  # shellcheck disable=SC2086
  set -- $line
  set +f
  IFS=$old_ifs
  if [ $# -lt 3 ]; then
    echo "STOP 清单行格式不对（要「名字 md5 地址」）：$line" >&2
    exit 2
  fi
  name=$1
  want=$2
  url=$3

  dest_file="$DEST/$name"
  t0=$(now_s)
  if [ -f "$dest_file" ] && [ ! -e "$dest_file.aria2" ]; then
    echo "--- $name 已在，跳过下载"
  else
    gb=$(disk_gb)
    if [ "${gb:-0}" -ge "$STOP_GB" ]; then
      echo "STOP 盘用量 ${gb}GB 已到 ${STOP_GB}GB（${DU_ROOT}），先停下"
      exit 3
    fi
    aria2c -x8 -s8 -k8M --file-allocation=none --max-tries=10 --retry-wait=10 \
      -d "$DEST" -o "$name" "$url"
  fi
  seconds=$(( $(now_s) - t0 ))

  got=$(md5_of "$dest_file")
  bytes=$(bytes_of "$dest_file")
  if [ "$(echo "$got" | tr 'A-Z' 'a-z')" = "$(echo "$want" | tr 'A-Z' 'a-z')" ]; then
    status=ok
  else
    status=BAD
  fi
  # 立刻记账（先落盘，再看下一个文件）
  printf '%s %s %s %s %s\n' "$name" "$got" "$bytes" "$status" "$(now_utc)" >> "$RECORD"
  echo "=== $name bytes=$bytes seconds=$seconds md5=$got want=$want"

  if [ "$status" != ok ]; then
    echo "STOP md5 不符 $name"
    exit 5
  fi
done < "$LIST"
