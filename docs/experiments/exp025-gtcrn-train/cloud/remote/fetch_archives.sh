#!/bin/sh
# 在云上机器里跑：把一批数据文件从 URL 下到网络盘，边下边记校验。
# 用法：sh remote/fetch_archives.sh <list file> <dest dir>
# 清单文件：一行一个文件，三或四列「名字 md5 地址 [字节数]」，空行与 # 开头的行忽略。
#   md5 是 32 位十六进制 → 按 md5 判（给了字节数就再对一次字节数）。
#   md5 写 '-' → 上游没发 md5，第四列字节数**必填**，只按字节数判。
#   别人的校验值绝不自己编：上游没给就记实算值，等以后能对上再钉。
# 环境变量（括号里是默认值）：
#   STOP_GB （185）网络盘用量到这里就停，不再下载
#   DU_ROOT （/workspace）用量统计的根目录
#
# 这个脚本只下载和记账，**从不删东西**：删不删是人的事。
# 每个文件立刻把「名字 md5 sha256 字节数 ok|BAD UTC时间 md5|bytes-only」追加进
# <dest>/checksums.txt，这份文件是耐久记录（md5 与 sha256 都在，以后能拿去钉）。
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

sha256_of() {  # sha256_of <文件>
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | cut -d' ' -f1
  else
    shasum -a 256 "$1" | cut -d' ' -f1
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

# 开工前一整遍把清单查完：清单不对就一个字节都不下。
# 参数：<清单文件>；不合格时打印原因并以 2 退出。
check_list() {
  while IFS= read -r line || [ -n "$line" ]; do
    case "$line" in
      '' | '#'*) continue ;;
    esac
    old_ifs=$IFS
    IFS=' 	'
    set -f
    # shellcheck disable=SC2086
    set -- $line
    set +f
    IFS=$old_ifs
    if [ $# -lt 3 ]; then
      echo "STOP 清单行格式不对（要「名字 md5 地址 [字节数]」）：$line" >&2
      exit 2
    fi
    if [ "$2" = "-" ] && [ $# -lt 4 ]; then
      echo "STOP 清单行：md5 未知时必须给字节数：$line" >&2
      exit 2
    fi
  done < "$1"
}

check_list "$LIST"

mkdir -p "$DEST"
RECORD="$DEST/checksums.txt"

while IFS= read -r line || [ -n "$line" ]; do
  case "$line" in
    '' | '#'*) continue ;;
  esac
  # 拆四列：名字 · md5 · 地址 · 字节数（多余空白忽略）
  old_ifs=$IFS
  IFS=' 	'
  set -f
  # shellcheck disable=SC2086
  set -- $line
  set +f
  IFS=$old_ifs
  if [ $# -lt 3 ]; then
    echo "STOP 清单行格式不对（要「名字 md5 地址 [字节数]」）：$line" >&2
    exit 2
  fi
  name=$1
  want=$2
  url=$3
  want_bytes=""
  if [ $# -ge 4 ]; then
    want_bytes=$4
  fi

  if [ "$want" = "-" ]; then
    mode=bytes-only
  else
    mode=md5
  fi

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
      -d "$DEST" -o "$name" "$url" < /dev/null
  fi
  seconds=$(( $(now_s) - t0 ))

  got=$(md5_of "$dest_file")
  got_sha=$(sha256_of "$dest_file")
  bytes=$(bytes_of "$dest_file")

  status=ok
  reason=""
  if [ "$mode" = md5 ]; then
    if [ "$(echo "$got" | tr 'A-Z' 'a-z')" != "$(echo "$want" | tr 'A-Z' 'a-z')" ]; then
      status=BAD
      reason=md5
    fi
  fi
  if [ -n "$want_bytes" ] && [ "$bytes" != "$want_bytes" ]; then
    status=BAD
    reason=bytes
  fi

  # 立刻记账（先落盘，再看下一个文件）
  printf '%s %s %s %s %s %s %s\n' \
    "$name" "$got" "$got_sha" "$bytes" "$status" "$(now_utc)" "$mode" >> "$RECORD"
  echo "=== $name bytes=$bytes seconds=$seconds md5=$got want=$want sha256=$got_sha mode=$mode"

  if [ "$status" != ok ]; then
    if [ "$reason" = bytes ]; then
      echo "STOP 字节数不符 $name"
    else
      echo "STOP md5 不符 $name"
    fi
    exit 5
  fi
done < "$LIST"
