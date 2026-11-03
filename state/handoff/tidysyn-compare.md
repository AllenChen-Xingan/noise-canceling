# #71 迁移比对记录（tidysyn）

每条规则迁移前，先用旧实现导出它对真实仓库的结果；迁移后用新实现按同样格式导出，逐条比对。
- 原始导出在 `_tmp/tidysyn/`（不入库）：`*-before.txt` / `*-after.txt`
- 导出用的是临时 `#[ignore]` 测试，比完就删
- 基线是 main 9f34aec4

## 1. 测试范围（`test_scope.rs` → `ast::modtree`）

导出格式：
- `FILE <路径>`：整份是测试代码的文件
- `LINES <路径> [行号…]`：生产文件里的测试行

旧版导出共 123 个文件，分两部分比：

| 部分 | 文件数 | 一致 | 不一致 |
|---|---|---|---|
| xtask 以外 | 113 | 111 | 2（见下） |
| xtask 里我没改的文件（denycheck · file_size · main · no_mirror · param_range · tidy · todo） | 7 | 7 | 0 |
| xtask 里我改过的文件（model_name 重写 · rate_literal 加临时导出 · test_scope 删除） | 3 | — | 内容变了，不可比 |

第 1 步汇报里写的「119 一致 + 3 处差异」漏数了 `src/xtask/src/main.rs`：当时它多了一行 `mod ast;`，后面的测试行号整体后移 1 行，内容没变。后来 `mod test_scope;` 删掉，行号又对回来了。

不一致的 2 处（都已逐条解释）：

| 文件 | 旧 | 新 | 结论 |
|---|---|---|---|
| `src/denoised/src/lib.rs` | 168–169 | 168–169、**171–173** | 171–173 行是 `#[cfg(all(test, debug_assertions))]` 下的 `#[global_allocator] static A …`，只在测试编译。旧版只认字面写的 `#[cfg(test)]`，**属于旧规则漏报，新规则更准**（PR 单列） |
| `src/types/src/graph.rs` | 399–402 | **398**–402 | 398 行是该项前面的 `///` 文档注释；语法树把文档注释算作项的一部分。无害 |

迁移中修过的 1 处：
- `src/shm/src/wait.rs` 26–27 行是函数体里带 `#[cfg(test)]` 的语句（`tests::gap();`）
- 新逻辑第一版漏了，补上「生产函数体里带测试属性的语句」后一致；单测 `test_lines_in_prod_file` 覆盖

建树结果：孤儿 `.rs` 0 个，建树错误 0 个。

## 2. R-MODEL（文本 → 语法树）

- 真仓正常状态 0 处违规，没法比。导出时临时把 `gtcrn` 也算进被查名字，扩大命中面
- 结果：旧 2 条、新 2 条，同文件同行（`src/models/src/lib.rs:119`、`:120`），**逐条一致**
- 旧单测里有几段片段不是合法的 Rust（单独一行属性、顶层 `let`），新实现要先解析成语法树，所以把它们包进函数或结构体。测的意图没变

## 3. R-RATE（逐行文本 → 语法树 + 常量折叠）

- 导出口径：全仓被认成采样率字面量的位置，不看放行注释，也不排除正本文件，这样才有东西可比
- 旧 4 处、新 4 处，**逐条一致**：
  - `src/rt-thread/examples/rt_bench.rs:248`、`src/scene/examples/scene_offline.rs:133`：带「非采样率」放行注释
  - `src/types/src/rate.rs:7`、`:10`：正本
- 新导出多 1 处 `src/xtask/src/rate_literal.rs:32`：这是本次新写的 `const RATES: [f64; 2] = [16000.0, 48000.0];`。旧实现那里写的是字符串数组，所以查不到，不是规则差异。已加放行注释「本规则要认的两个数本身」
- 新规则多查的写法（旧规则查不到，真仓里目前都没有）：
  - 十六进制、八进制、二进制（`0xBB80`）
  - 两边都是字面量的算式（`3 * 16_000`、`48 * 1000`、`144_000 / 3`）
  - 各有负例：R4、R5

## 4. R3（逐行找文本 → 语法树路径 + use 别名还原）

- 导出口径：实时 crate（A / B 档）`src/` 下每个文件被认出的行，**不看整模块 NRT 标注**（真仓有 NRT 的文件里才有命中，不这样就没东西可比）
- 旧 7 行；新 12 行。其中 6 行相同：`capture/src/main.rs:307`、`:394`，`engine/src/fault.rs:76`，`models/src/artifact.rs:12`，`shm/src/owned.rs:19`，`shm/src/wait.rs:68`

差异：

| 位置 | 旧 | 新 | 结论 |
|---|---|---|---|
| `src/denoised/src/tests.rs:224` | 报 | 不报 | 这个文件经 `#[cfg(test)] mod tests;` 引入，模块树判为测试代码。旧规则只认「`#[cfg(test)]` 紧跟 `mod … {`」之后的行，认不出外置测试文件。**新规则更准** |
| `src/shm/src/owned.rs:30 · 31 · 86 · 87 · 98 · 99` | 不报 | 报 | `use std::fs::{File, TryLockError};` 之后，这几行用的 `File` / `TryLockError` 还原后就是 `std::fs::File` 等。旧规则只按文本认出了 `use` 那一行。**旧规则漏报**；这个文件是整模块 NRT，真跑 tidy 时照旧跳过 |

迁移中修过的 1 处误报：
- `src/speaker/src/judge.rs` 有个局部变量叫 `log`（`log.prev_sim = …`）。新规则第一版把单独一个标识符 `log` 当成了 `log` 路径，报了 9 行
- 改成「没经过 use 别名还原的单个标识符不算路径」后消除，负例已加进 `comments_strings_and_other_paths_pass`

另外：禁用清单 `BANNED` 和根 `clippy.toml` 里标 R3 的 `disallowed-methods`（`std::thread::sleep`）有比对测试 `banned_list_matches_clippy_toml`。

真跑 `xtask tidy`：通过。

## 5. R-PARAM（逐行文本 → 语法树，同一条语句里找参数名）

**键名**：58 个，`ParamDesc` 常量 58 个，新旧解析**逐条一致**。

**数字范围**（导出口径：被认作「数字写的范围」的位置，不管有没有参数名；真仓 0 违规，不这样就没东西可比）：
- 旧 159 处，新 152 处
- 新认出的旧也都认出（新没有多报）
- 少的 7 处都是端点为算式的范围：`nonfinite_input.rs:42 · 128`、`eval/system.rs:375`、`polish/tests/behavior.rs:136 · 265`、`scene/tests/behavior.rs:56 · 75`，写法如 `0..62 * 8`、`12 * hop_io + 100..12 * hop_io + 1000`。旧文本匹配截到中间一段（`0..62`、`100..12`）当成了数字范围，**属于旧规则误报**

迁移中补过的 1 处：宏参数里的范围（`assert!((500..510).contains(&n))` 等 9 处）。新规则第一版不看宏参数；改成「能解析成逗号分隔的表达式就照查」后一致。负例已加 `assert!((-60.0..=0.0).contains(&floor_db));`

新增 P3：默认值字面量。按大总管 8d 定：
- 收窄成「数值等于描述表里该参数的默认值才报」，本 PR 直接上线就红
- 本意是预设、碰巧等于默认的，行尾写 `// tidy: 预设值，不随默认变：<理由和出处>`；理由为空照红，有负例

真仓命中 6 处，逐条判断如下。改后 vdn-polish、vdn-scene、vdn-speaker-lab 的测试和改前**逐条一致**（30 过 / 0 败）：

| 位置 | 字面量 | 判断 | 依据 | 处理 |
|---|---|---|---|---|
| `src/polish/src/params.rs` 94 · 95 · 96 | `agc: 1.0` · `agc_target_dbfs: -26.0` · `agc_range_db: 10.0`（`preset` 的 `base` 闭包） | 本意用默认 | 各场景预设只调前四项，AGC 三项在所有预设里都一样；`DEFAULT` 本身就写 `desc::POLISH_AGC*.default` | 改成 `desc::POLISH_AGC.default` 等，值不变 |
| `src/polish/src/params.rs` 100 | `agc: 1.0`（`tone >= 20` 只开 AGC） | 预设，碰巧等于默认 | 这里的 1.0 是开关「开」（描述表：≥ 0.5 视为开），不是「用默认」；默认若改成关，这个预设不能跟着变 | 保留，加放行注释 |
| `src/scene/src/weights.rs` 22 | `HYSTERESIS { confirm_s: 1.0, … }` | 训练脚本产出，与默认值一致已有测试比对 | 文件由 `docs/experiments/exp006-scene-sim/train.py` 的 `--hyst` 生成，文件头写「别手改」；`src/scene/tests/behavior.rs:90` 断言它等于 `Hysteresis::default()` | 保留，加放行注释；同时改 `train.py` 让它生成这一行时就带上同样的注释（训练要 venv 与数据，这次不重跑；手改的这一行与脚本将生成的逐字相同） |
| `src/speaker-lab/src/run.rs` 254 | `release_s: 0.3`（激进版 `GateParams`） | 预设，而且撞的是另一个同名参数 | 大总管 2026-10-30 定的「激进版：慢收 0.3 s」；`GateParams` 对应 `gate.release_s`（默认 0.5），0.3 是另一个参数 `release_s` 的默认，只是短名相同 | 保留，加放行注释 |

## 6. R-MIRROR（逐行文本 → 语法树 + 结构指纹）

- 导出口径：vdn-types 公开类型名清单；全仓同名定义（把放行标记抹掉，不看放行）；新规则另导出「改名但结构相同」的定义
- 类型名清单：**逐条一致**
- 同名定义：旧 1 处、新 1 处，**一致**（`src/speaker/src/vad.rs:88` 的 `Hysteresis`，已写「同名不同物」理由）
- 改名副本（新规则才有）：真仓 0 处
- 有负例 M1–M3：改名、字段换顺序、带 `vdn_types::` 前缀的副本都报；字段少于 3 个、结构不同的不报

## 7. 时间（改前 · 改后）

测量条件：Mac17,3 · macOS 27.0.1 · 插电 · 无别的重活；每项连量 3 次（tidy 5 次）取中位数。

| 项 | 改前（main 9f34aec4） | 改后 | 预算（已定：超了只报不挡） |
|---|---|---|---|
| xtask 全依赖冷编译（独立 `CARGO_TARGET_DIR`） | 1.59 s | 4.48 s（+2.89 s） | 增加 ≤ 30 s：过 |
| xtask 本身冷编译（`cargo clean -p xtask`） | 0.52 s | 0.92 s | — |
| xtask 热编译（touch 一个文件） | 0.21 s | 0.26 s | — |
| `xtask tidy` 运行（调试构建） | 0.563 s | **0.632 s** | ≤ 3 s：过 |
| `cargo test -p xtask` 运行 | — | 0.69 s | — |

中间过程：
- 第一版没做优化，tidy 跑 3.92 s，超了预算
- 做了两件事：
  1. 调试构建里给 `syn` / `proc-macro2` / `quote` 单独开 `opt-level = 3`（根 `Cargo.toml` 的 `[profile.dev.package.*]`），降到 1.03 s
  2. 同一次 tidy 里每份源码只解析一次（`ast::parse_src` 按内容缓存）、模块树只建一次，降到 0.632 s
- 全依赖冷编译多出的 2.9 s 主要是给 syn 开优化的代价
- 这个 profile 设置对全工作区生效：用到 syn 的派生宏（serde_derive 等）第一次编译也会稍慢，只在冷编译时付一次
