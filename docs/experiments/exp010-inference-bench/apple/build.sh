#!/bin/sh
# 编译苹果侧测量程序到 ./build/（-O，单文件，不进库）
set -e
cd "$(dirname "$0")"
mkdir -p build
clang -O2 -c alloc_hook.c -o build/alloc_hook.o
swiftc -O -import-objc-header alloc_hook.h bench_coreml.swift build/alloc_hook.o -o build/bench_coreml
echo build/bench_coreml
clang -O2 -Wall bench_bnns.c build/alloc_hook.o -framework Accelerate -o build/bench_bnns
echo build/bench_bnns
clang -O2 -dynamiclib alloc_hook.c -o build/libvdn_alloc.dylib   # 给 mlx_gtcrn.py 用 ctypes 载入
echo build/libvdn_alloc.dylib
