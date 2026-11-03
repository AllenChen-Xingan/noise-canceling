# apple-rs · 苹果引擎基准（从 Rust 调，D68）

用途：按 `docs/research/019_apple-inference/standard.md` 的口径，从 Rust 测 Core ML / BNNS Graph 跑 GTCRN 等模型。
字段以 rustbe 的 `../bench/src/main.rs` 为唯一真相源，由 `tests/fields.rs` 机器比对（CLAUDE.md 第五节第 5 条）。

## 构建与测试

```sh
cargo build --release
cargo test --release                       # 单元测试（JSON 数值、分位数口径、GTCRN 布局）+ 字段交叉校验
python3 -m unittest tests/test_power_parse.py   # 耗电解析（计时窗、覆盖率、缺项、不排名）
./target/release/exp010-apple selftest     # 分配计数器自检（要能数到 libc malloc 与 Rust 分配）
```

## 跑基准（要名额）

- 全套：`tools/run-all.sh`（转模型 → 转储 → 构建 → 3 轮交替基准）
- 实时线程设置 × 耗电：`tools/rt-matrix.sh`，要人 sudo，命令见 `tools/HUMAN-RUN-power.md`
- 对前台性能的影响：`tools/impact.sh`（BE-IMPACT）
- 认人声纹：`tools/ecapa.sh`

计时与计数分开跑：计时那次不加 `--count-malloc`；量 BE-MEM 那次加 `--parity-frames 10`。
在 zsh 里不要把多个参数放进一个变量再展开（zsh 不分词，会变成一个参数）。
