#!/usr/bin/env bash
# 云上装 uv（cloud_steps.sh setup 调用）。机器上已有 uv 就直接用；没有就从 uv 官方 GitHub release
# 下载钉死版本的二进制包，按官方 sha256 核对，再解压到 $VDN_ROOT/bin（默认 /workspace/bin）。
# 不用「curl 拿脚本交给 sh」：只下二进制包，哈希不对就拒绝、不解压。
# 起因（2026-11-25）：rust:1.99.1-trixie 镜像里没有 pip，旧写法 `pip install uv` 失败。
# 正本：下面三行常量。sha256 抄自官方 release 的 uv-<目标>.tar.gz.sha256（2026-11-25 核过，与实下的包一致）。
# 升级：改 UV_VERSION，两行 sha256 一起换成新版官方 .sha256 文件里的值。
set -euo pipefail
UV_VERSION=0.13.7
UV_SHA256_X86_64=c5221ff41b7892bfc223543883600125f6a47fe845093e0f5bdff4f359196d83
UV_SHA256_AARCH64=71279f8d3c67241e807b9d8db872d1bf541579fb9f08b7d1eb1682196d1e57dc

uv_target() {  # 机器架构 → 官方包的目标名
  case "$1" in
    x86_64|amd64) echo x86_64-unknown-linux-gnu ;;
    aarch64|arm64) echo aarch64-unknown-linux-gnu ;;
    *) echo "不支持的架构：$1" >&2; return 2 ;;
  esac
}

uv_sha() {
  case "$1" in
    x86_64-unknown-linux-gnu) echo "$UV_SHA256_X86_64" ;;
    aarch64-unknown-linux-gnu) echo "$UV_SHA256_AARCH64" ;;
  esac
}

uv_url() { echo "https://github.com/astral-sh/uv/releases/download/$UV_VERSION/uv-$1.tar.gz"; }

sha256_of() {
  if command -v sha256sum >/dev/null; then sha256sum "$1" | cut -d' ' -f1
  else shasum -a 256 "$1" | cut -d' ' -f1; fi
}

# install_uv <下载地址> <预期 sha256> <目标目录>：下载 → 核哈希 → 解压出 uv · uvx。哈希不对退出码 3，目标目录不动。
install_uv() {
  local url=$1 want=$2 bin=$3 tmp got
  tmp=$(mktemp -d)
  trap 'rm -rf "$tmp"' RETURN
  curl -fsSL --retry 3 -A generic-client -o "$tmp/uv.tar.gz" "$url"
  got=$(sha256_of "$tmp/uv.tar.gz")
  if [ "$got" != "$want" ]; then
    echo "uv 包 sha256 不对，拒绝安装：预期 $want，实得 $got（$url）" >&2
    return 3
  fi
  tar -xzf "$tmp/uv.tar.gz" -C "$tmp" --strip-components=1
  mkdir -p "$bin"
  install -m 755 "$tmp/uv" "$tmp/uvx" "$bin/"
  echo "uv $UV_VERSION 已装到 $bin"
}

main() {
  local bin=${VDN_BIN:-${VDN_ROOT:-/workspace}/bin} target
  if command -v uv >/dev/null; then echo "已有 uv：$(command -v uv)"; return 0; fi
  if [ -x "$bin/uv" ]; then echo "已有 uv：$bin/uv"; return 0; fi
  target=$(uv_target "${VDN_UV_ARCH:-$(uname -m)}")
  # VDN_UV_URL 只给测试换成本地文件；预期哈希始终取上面的常量
  install_uv "${VDN_UV_URL:-$(uv_url "$target")}" "$(uv_sha "$target")" "$bin"
}

[ "${VDN_UV_LIB:-}" = 1 ] || main
