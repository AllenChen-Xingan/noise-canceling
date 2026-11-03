# 012 · tidy 改用 syn 语法树判规则（设计，issue #71）

> 状态：设计稿，待大总管 8d 过目。**只写设计，不写代码**：等 #62（代码卫生）与 #65（参数描述导出，也往 xtask 加规则）都合并后再动手。
> 实体：xtask（tidy 闸门）。人原话（2026-11-02）：「可以真正用ast工具」。
> 依据：#62 的 7 轮 codex 评审（`docs/reviews/062_code-hygiene-codex.md`）· ADR 004（`docs/research/008_rust-architecture/adr/004-ast-guard-not-generate.md`）· AST 报告扩展方向第 8 条（`docs/reviews/ast-complexity-20261102/README.md`）。
> 标记：【测】实测 ·【源】读源码 ·【推】推断。

## 一、为什么现在做

- **触发已满足**。ADR 004 写「同一类规则漏网第三次」再上语法树工具。#62 里「自动识别测试模块」用文本猜模块结构，codex 第 2–7 轮共找到 9 个绕过点【测】，最后放弃，改用文件头标注。
- **现有文本规则的已知漏洞**都写在规则文件头【源】：
  - R-PARAM 只按行查，rustfmt 拆行后查不到
  - R-MIRROR 只比名字
  - R-RATE 查不到算出来的值（`3 * 16_000`）
- **不是重写一切**：只把「要懂 Rust 结构」的判断换成语法树。纯文本就够的规则（`.gitignore`、Cargo 元数据、行数）不动。

## 二、总体做法

1. xtask 加依赖 `syn` 2.x（特性 `full`、`visit`），外加 `proc-macro2`。
   - 锁文件已有 `syn 2.0.119`【源】，不新增版本
   - xtask 仍不依赖任何 vdn crate（ADR 004 / xtask 的 forbid 不变）
2. 新增共用模块 `src/xtask/src/ast/`，一条规则一个文件，按查什么命名：
   - `ast/parse.rs`：读文件 → `syn::File`
     - 解析失败算红，报文件和行号；不静默跳过（同 R-SIZE 第 1 轮 P2 的教训）
   - `ast/modtree.rs`：从 crate 的编译根建模块树（见三.1）
   - `ast/lit.rs`：遍历表达式与宏里的字面量（见三.2）
3. 每条规则改成「遍历语法树 + 一张小规则表」。旧的逐行实现改造期间保留，新旧并跑比对（见五），一致后删旧的。
4. 每个文件 ≤ 600 行（R-SIZE 本身管着）。

## 三、每条规则在语法树上怎么判

### 1. 测试代码识别（恢复 #62 放弃的自动识别）

**建模块树，不猜**：

- **起点**：每个 crate 的编译根（`lib.rs`、`main.rs`、`bin/*.rs`、`bin/*/main.rs`、`tests/*.rs`、`examples/*.rs`、`benches/*.rs`）。现有 `crate_roots` 已列出前四种【源】。
- **展开**：遍历 `syn::ItemMod`。
  - `content` 为 `Some` 的是内联模块：进入子作用域，路径加一段。
  - `content` 为 `None` 的指向文件，按 Rust 规则找：
    - 声明它的文件是 `lib.rs` / `main.rs` / `mod.rs` 时，在同目录找
    - 否则在 `<文件名>/` 下找
    - 外层若有内联模块，路径还要加上它们的名字
    - 找 `name.rs` 或 `name/mod.rs`；有 `#[path = "…"]` 就按它的值，相对声明文件所在目录
  - `r#name` 用 `Ident::unraw` 归一。
- **测试判定**：一个模块是测试代码，当且仅当它或某个祖先模块带 `#[cfg(test)]`（或 `#[cfg(all(test, …))]`），或者它在 `tests/`、`benches/` 编译根下。
- **一个文件可能被多处声明**（例如 lib.rs 在 `cfg(test)` 下声明，main.rs 普通声明）：**只要有一条非测试路径能到它，它就是生产代码**。逐条路径算，不需要旧做法那样「反复排除到不再变化」。
- **函数级**：`#[test]` / `#[cfg(test)]` 函数体算测试代码。
- **文件里有、但任何根都到不了的 `.rs`（孤儿文件）**：报红。它不参与编译，多半是忘了删或忘了声明。

旧文本做法的 9 个绕过点，在树上都自然消失：
- 注释、字符串里的声明：根本不在语法树里
- 内联模块路径：子作用域自动带上
- `r#tests`：用 `unraw` 归一
- 带属性、同一行的写法：语法树不分行

见四.1 的负例清单。

### 2. R-RATE（采样率只写一处）

- 用 `syn::visit` 遍历所有 `Expr::Lit` 里的 `Lit::Int` / `Lit::Float`：
  - 用 `base10_parse` 取数值，值为 16000 或 48000 即命中，不管写法（`48_000u16`、`48000.0_f32`、`0x_BB80`）
  - 后缀白名单、前后字符判断都不要了
- **算出来的值**：`Expr::Binary` 两边都是字面量时做常量折叠（`48 * 1000`、`3 * 16_000`），结果是 16000 / 48000、或等于它们的整数倍 / 整除结果（48000 / 3）时命中。只折叠字面量，不追变量。
- **宏里的字面量**：见四。
- **放行**：
  - 正本文件
  - 单行放行：属性 `#[allow_rate(reason = "…")]` 不是合法 Rust，所以仍用行尾注释 `// tidy: 非采样率：<理由>`。按字面量所在行号取注释，用 `proc_macro2` 的 span-locations 特性拿行号
- **测试代码**：#62 已定「测试也必须引用常量」，不放行，不依赖三.1。

### 3. R-PARAM（参数范围不许在描述表之外写成字面量）

- 遍历：
  - `Expr::MethodCall`：方法名 `clamp` / `max` / `min`，参数都是数字字面量
  - `Expr::Range`：两端都是数字字面量
  - `Expr::Match` 分支体是两个数字字面量的元组（对应旧规则的 `=> (a, b)`）
- 参数名关联：看接收者（`x.floor_db.clamp(…)`）或所在表达式里有没有参数名的路径或字段访问。现在按「同一行」近似，改成「同一表达式子树」，rustfmt 拆行不再影响。
- **新增**：默认值字面量（`floor_db: -12.0` 写在结构体字面量 `Expr::Struct` 里）。只在非测试代码里查；测试里造输入照旧允许（这一处要用三.1 的测试判定）。
- 参数名清单仍从 `desc.rs` 解析，改为解析它的语法树：读 `p(编号, "键名", …)` 调用的第二个参数，替代逐行剥注释再找。

### 4. R-MIRROR（vdn-types 之外不许有镜像类型）

- 现在只比名字。改成比**结构指纹**：
  - `struct` 的字段名 + 类型的规范化文本
  - `enum` 的变体名与字段
  - `trait` 的方法签名
- 判定：
  - **同名且结构相同** → 红（镜像副本）
  - **改了名但结构相同、且字段 ≥ 3 个** → 红（改名的手抄副本，旧规则查不到）
  - **同名但结构不同** → 照旧要求放行注释 `tidy: 同名不同物：<理由>`。#62 的 `ChainVerdict` / `LiveVerdict` 就是这类的反例，改名后不再撞名
- 字段数门槛 3 是建议值：太小会把 `struct Pair(f32, f32)` 这类通用小结构误报。

### 5. R3（实时 crate 禁用 I/O、阻塞调用）

- 遍历 `Expr::Call`（函数路径）与 `Expr::MethodCall`。
- 先建本文件的 `use` 表，把别名还原成全路径：`use std::thread::sleep as nap;` 之后的 `nap(…)` 等于 `std::thread::sleep(…)`。旧规则按文本找 `thread::sleep`，别名能绕过。
- 禁用清单与根 `clippy.toml` 的 `disallowed-methods` 对齐：从 `clippy.toml` 读，不再手抄第二份，并加一条比对测试，符合 CLAUDE.md 第五节第 5 条。
- 整模块 NRT 标注（`#![allow(…)] // NRT: 理由`）照旧有效。
- **测试代码**（三.1 判定）不查。#62 改成「文件头标注」的那些文件，届时可以删掉标注；删不删由当时的 PR 定。

## 四、syn 判不了的，各怎么办

| 判不了的东西 | 为什么判不了 | 办法 |
|---|---|---|
| **宏内部**（`vec![48_000; n]`、`println!("{}", 16000)`、自定义 `macro_rules!` 的展开） | 宏调用在语法树里只是一串 token，没展开 | ① 对 `Macro::tokens` 递归找 `Literal` token 并解析数值，R-RATE 照查（宏里出现采样率字面量就红）；② 先试 `syn::parse2::<Punctuated<Expr, Token![,]>>`，成功就按表达式查，R3、R-PARAM 也能查到；③ 解析不了的宏，R3 只按 token 里的路径片段（`thread` `::` `sleep`）查，宁可多报，用行尾放行注释说明；④ `macro_rules!` 定义体里的字面量照 ① 查 |
| **类型推断 / 名字解析**（`let f = std::thread::sleep; f(d)`、trait 方法到底调了谁、跨模块 `pub use` 再导出） | syn 只有语法，没有类型和名字解析 | ① R3 额外禁止把禁用函数**当值引用**（`Expr::Path` 指向禁用路径但不在调用位置）；② 跨文件的再导出不追：禁止在实时 crate 里 `use` 禁用路径并改名（`use … as`），改名本身就红；③ 方法调用不知道接收者类型，只按方法名 + 禁用清单里的「方法名唯一」项查，其余靠 clippy 的 `disallowed-methods`（它有类型信息）兜底。分工写进 ADR：tidy 管结构与写法，clippy 管类型 |
| **`#[path = "…"]`** | 值是任意相对路径 | 按值解析，相对声明文件所在目录；找不到文件 → 红（不当成不存在）；路径跳出 crate 目录 → 红 |
| **`cfg` 求值**（`#[cfg(any(test, feature = "x"))]`、`cfg_attr`、`#[cfg(not(test))]`） | 不知道构建时开了哪些 feature | 只认三种写法为测试：`cfg(test)`、`cfg(all(test, …))`、`cfg(all(…, test))`。`any(test, …)`、`not(…)`、`cfg_attr` 一律按「可能进生产」处理，宁可多查。feature 门控的代码照生产代码查 |
| **build.rs 生成、`include!` 引入的代码** | 不在源码树里 | 本仓不用 build.rs 生成（ADR 004）；`include!` 出现在非测试代码里 → 红，要求说明 |
| **解析失败**（语法错或用了 syn 不支持的新语法） | — | 报红并给文件与行号；不跳过。cargo build 本身也会先挂 |

## 五、负例清单（必须红）

来源：#62 的 7 轮 codex 评审（第 2–7 轮共 13 条 P1 / P2 涉及绕过）与各规则文件头写明的边界。每条做成一个单元测试：造临时 crate 目录，断言规则报红。正例各配一条，断言不红。

### 1. 测试代码识别

| # | 写法 | 来源 | 期望 |
|---|---|---|---|
| T1 | 块注释里写 `#[cfg(test)]` + `mod worker;` | 第 2 轮 P1 #2 | worker.rs 按生产查 |
| T2 | 字符串里写同样两行 | 第 2 轮 P1 #2 | 同上 |
| T3 | `mod outer { #[cfg(test)] mod inner; }` | 第 2 轮 P1 #2 | `outer/inner.rs` 是测试，`inner.rs` 不是 |
| T4 | lib.rs `#[cfg(test)] mod worker;` + main.rs 普通 `mod worker;` | 第 3 轮 P1 #2 | worker.rs 按生产查 |
| T5 | T4 再加 worker.rs 普通 `mod child;` | 第 4 轮 P1 #1 | worker/child.rs 按生产查 |
| T6 | T4 加 worker.rs 写 `mod inner { mod child; }` | 第 5 轮 P1 #1 | worker/inner/child.rs 按生产查 |
| T7 | main.rs `mod tests { mod child; }`（内联）+ lib.rs `#[cfg(test)] mod tests;` | 第 6 轮 P1 #1 | tests/child.rs 按生产查 |
| T8 | main.rs `mod r#tests;` | 第 6 轮 P1 #2 | tests.rs 按生产查 |
| T9 | main.rs `#[allow(dead_code)] mod tests;` | 第 7 轮 P1 #1 | 同上 |
| T10 | main.rs `fn main() {} mod tests;`（同一行） | 第 7 轮 P1 #1 | 同上 |
| T11 | `#[cfg(test)]` 后隔一个别的项，再 `mod tests;` | 第 7 轮负例 | tests.rs 按生产查 |
| T12 | `#[cfg(any(test, feature = "x"))] mod helper;` | 本设计 四 | helper.rs 按生产查 |
| T13 | `#[path = "../outside.rs"] mod x;` 跳出 crate | 本设计 四 | 红 |
| T14 | 孤儿 `.rs`（没有任何声明） | 本设计 三.1 | 红 |
| 正 | `#[cfg(test)] mod tests;` → tests.rs 与 `tests/more.rs`；worker.rs 里的 `#[cfg(test)] mod wt;` | 第 3、4 轮正例 | 是测试 |

### 2. R-RATE

| # | 写法 | 来源 |
|---|---|---|
| R1 | `0..16000`、`x[..48_000 - l]`、`(0..=48_000)` | 子代理报告（区间写法漏查） |
| R2 | `48000u16`、`16000i16`、`48_000u128`、`16_000_i128` | 第 8 轮 P2 #3 |
| R3 | `48000.0`、`16_000.0_f32` | 第 1 版正例 |
| R4 | `0xBB80`（= 48000）、`0o135600`（= 48000） | 本设计 三.2（文本规则查不到） |
| R5 | `3 * 16_000`、`48 * 1000`、`144_000 / 3` | 规则文件头「边界」 |
| R6 | `vec![0.0; 48_000]`、`assert_eq!(r, 16000)` | 本设计 四（宏内） |
| R7 | 测试模块、`tests/`、`examples/`、底层 crate 里的字面量 | 第 2、3 轮 P2，8d 定「测试也查」 |
| R8 | `// tidy: 非采样率：`（理由为空） | 第 2 轮 P2 #5 |
| 正 | 注释、字符串里的 16000；`160000`、`4800`、`48000.5`；`16_000_000`（纳秒） | 第 1 版反例 |

### 3. R-PARAM · R-MIRROR · R3

| # | 写法 | 来源 |
|---|---|---|
| P1 | `x.floor_db.clamp(\n    -60.0,\n    0.0,\n)`（rustfmt 拆行） | single_source.rs 文件头边界 |
| P2 | `let r = -60.0..0.0;` 后用 `floor_db` 比 | 同上（只在同一表达式子树内查，这条预期仍查不到，写成 `#[ignore]` 记录限制） |
| P3 | 非测试代码里 `Polish { floor_db: -12.0, .. }` | AST 报告方向 8「R-PARAM 加查默认值字面量」 |
| M1 | vdn-types 外 `struct Frame { … }`，结构与正本相同 | R-MIRROR 原规则 |
| M2 | 改名为 `struct FrameCopy`，字段相同（≥ 3 个） | 文件头「改名后的手抄副本查不到」 |
| M3 | 同名不同结构、带放行注释 | 正例（#62 的 `Verdict` 情形） |
| S1 | `use std::thread::sleep as nap; nap(d);` | 本设计 三.5 |
| S2 | `let f = std::thread::sleep; f(d);` | 本设计 四 |
| S3 | `std::fs::read(p)` 写在宏参数里：`dbg!(std::fs::read(p))` | 本设计 四 |
| S4 | 测试模块里的 `thread::sleep` | 正例：不红 |

## 六、怎么量（编译时间 · 运行时间 · 结果一致）

- **编译时间**（syn 可能已被别的依赖编译过，增量代价未知，所以冷、热都量）：
  - 冷：`cargo clean -p xtask && cargo build -p xtask --timings`，记总秒数与 `syn` 一项的秒数
  - 热：改一行 xtask 源码后 `cargo build -p xtask`
  - 改前改后各量 3 次，取中位数。报：机型、系统版本、插电还是电池、同时有没有别的重活（与 common.md 同口径），量之前看 `_tmp/heavy.lock`
- **运行时间**：`target/debug/xtask tidy` 连跑 5 次取中位数；有 `hyperfine` 就用它。分开报解析全仓的耗时与各规则的耗时。
- **预算（建议值，待大总管定）**：
  - xtask 冷编译增加 ≤ 30 s
  - `tidy` 运行 ≤ 3 s（现约 0.6 s【测】，xtask 测试时间）
  - 超预算就报数，不硬上
- **结果一致**：新旧实现并跑（加一个隐藏子命令 `tidy --compare`），对真实仓库输出两份问题清单逐条比对。
  - 差异逐条解释：新的多报 = 旧规则漏网，附写法；新的少报 = 旧规则误报，附理由
  - 差异表写进实现 PR 正文
  - 一致后删旧实现，同一个 PR 里删，不留死代码

## 七、ADR 004 要改的段落

| 位置 | 现在 | 改成 |
|---|---|---|
| 「决定」2 第 5 条 | `xtask tidy（#[test]，文本 / 轻语法检查）` | `xtask tidy（#[test]）：要懂 Rust 结构的规则用 syn 语法树判（模块树、字面量、调用、类型结构）；纯文本就够的（.gitignore、Cargo 元数据、行数）仍按文本` |
| 「决定」2 第 7 条 | `ast-grep 规则留作第二阶段` | `第二阶段已启动（#71，2026-11-02）：选 syn，不选 ast-grep，理由见修订节` |
| 「决定」4 | `dylint 暂缓（同一类规则漏网第三次再上）` | `dylint 仍暂缓：现有规则都不需要类型信息；要类型的那部分交 clippy disallowed-methods。触发条件改为「某条规则必须靠类型信息才能判，且 clippy 配不出来」` |
| 「代价」新增一条 | — | `xtask 多一个依赖 syn（锁文件已有），冷编译变慢（实测数字见 #71 PR）；tidy 规则要懂 Rust 模块规则，模块树代码本身要有负例测试` |
| 文末新增「修订（2026-11-02，#71）」 | — | 触发记录：#62 第 2–7 轮 9 个绕过点；R-PARAM / R-MIRROR 文件头写明的漏洞。选 syn 的理由（下） |

选 syn 的理由（写进修订节）：

1. **Rust 生态内**：用 `cargo test` 就能跑，CI 不用装外部二进制；ast-grep 要另装 CLI，规则用 YAML 写，规则本身的单元测试要另搭。
2. **能建模块树**：测试识别的核心是跨文件的模块路径与 `#[path]`。这要写程序，ast-grep 的模式匹配做不了。
3. **锁文件已有 syn**：不引新的依赖树；cargo-deny 不受影响（实现时跑 `cargo deny check` 确认）。
4. **不选 dylint**：要 nightly 和 rustc 内部 API，升级工具链容易坏；现有规则都不需要类型信息。

## 八、分步与回退

| 步 | 内容 | 验收 |
|---|---|---|
| 0 | 等 #62、#65 合并；merge main | — |
| 1 | `ast/parse.rs` + `ast/modtree.rs` + 测试识别（三.1），负例 T1–T14 | 测试全绿；对真仓列出全部测试文件，人工抽查 |
| 2 | R-RATE 换实现（三.2），R1–R8 | `tidy --compare` 差异表 |
| 3 | R3 换实现（三.5），S1–S4；禁用清单改读 clippy.toml | 同上 |
| 4 | R-PARAM、R-MIRROR 换实现（三.3、三.4），P1–P3、M1–M3 | 同上 |
| 5 | 删旧实现与 `--compare`；改 ADR 004；量编译与运行时间 | `xtask ci` 全绿；cargo-deny 通过；数字写进 PR |

- 每步一个提交、可单独回退；步 1 最大，单独一轮 codex 评审。
- **回退条件**：任一步的负例在两轮评审内仍补不完，或时间超预算 2 倍。那就停在上一步，把剩下的规则留在文本实现，写进 ADR 修订节。

## 九、待定（大总管定）

1. 编译 / 运行时间预算（六）用不用建议值。不回就用建议值，超了只报不挡。
2. R-MIRROR 结构指纹的字段数门槛 3（三.4）。不回就用 3。
3. 孤儿 `.rs` 报红（三.1，T14）会不会撞上现有文件：实现第 1 步先列出来再定。不回就先只报注释、不红。
