#!/bin/sh
# 编 mlx-c（MLX 官方 C 绑定），对着 venv 里 pip 装的 MLX（自带 libmlx.dylib 与 mlx.metallib），不从源码编 MLX。
# 原因：本机只有 Command Line Tools，没有 Metal 编译器（xcrun -f metal 失败），从源码编 MLX 出不了 GPU 内核。
# 用法：tools/build-mlx-c.sh <venv> <mlx-c 版本标签> <输出目录>      要名额（cmake 构建）
# 产物：<输出目录>/lib/libmlxc.a 与 include/；apple-rs 用 `MLXC_DIR=<输出目录> cargo build --release --features mlx`
set -e
venv=$1; tag=$2; out=$3
mlxdir=$("$venv/bin/python" -c 'import mlx, os; print(os.path.dirname(mlx.__file__))')
src="$out/src"
[ -d "$src" ] || git clone --depth 1 --branch "$tag" https://github.com/ml-explore/mlx-c "$src"
cmake -S "$src" -B "$out/build" -DCMAKE_BUILD_TYPE=Release -DMLX_C_USE_SYSTEM_MLX=ON \
  -DMLX_DIR="$mlxdir/share/cmake/MLX" -DCMAKE_INSTALL_PREFIX="$out" -DBUILD_SHARED_LIBS=OFF
cmake --build "$out/build" -j 4
cmake --install "$out/build"
echo "mlx-c → $out；libmlx 在 $mlxdir/lib"
