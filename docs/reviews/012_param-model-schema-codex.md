# 评审 012 · 参数描述导出 · 模型接口三方一致 · FE-M 名字唯一（codex · gpt-6-astra · read-only）

> 对象：`git diff origin/main...HEAD`（分支 `feat/param-model-schema`，issue #65；diff 模板，附 `state/handoff/iface.md` 与重点说明）。**最新一轮：第 6 轮（2026-11-02）· 总评「修改」· 本轮无 P0/P1（连续第 1 轮），待第 7 轮确认**。第 1–3 轮审 #65 已达停止线；之后合并 main、加做 #68，从第 4 轮起续审。
> 累计处理：P1×3、P2×14，逐条回到源码核实，17 条全部属实；16 条全改，第 5 轮 #1 部分接受（理由见该轮，第 6 轮 codex 未再坚持）。没有留 issue。新冒出的需要人定的事：没有。
> 作者：手下 iface。

## 第 1 轮（2026-11-02）· 总评「修改」

原文：本 worktree 的 `_tmp/reviews/012-param-model-schema-codex-1.md`（不入库）。effort medium，耗时 70 秒（脚本末行：`result: P0×0 P1×1 P2×4 P3×0`）。行号指评审时的版本。
本轮 P0/P1 共 1 条 · 停止线：未满足 · 留作 issue 的 P2：无（4 条都已改）。隐私扫描干净（48 个文件）。

- **P1｜src/models/src/interface.rs:103 接口核对漏了元素类型**（第 1 轮 #1）
  - 核实：属实。`TensorRead` 只记名字和形状；同名同形的 f16 或整数张量能通过核对，适配器却按 f32 预分配，错误要到推理时才出
  - 处理：`TensorRead` 加 `dtype`（经 ort 的 `tensor_type()` 读出），核对时要求等于 `interface::DTYPE`（"f32"）。导出文件的 `dtype` 也改为取这个常量。比对逻辑抽成纯函数 `check_read`，新测试 `check_read_rejects_any_difference`：元素类型、不是张量、顺序、形状、名字任一不同都拒。本机真权重重跑 `tests/interface.rs` 2 条仍全过
- **P2｜src/xtask/src/model_name.rs:134 碰到 `#[cfg(test)]` 就停查整个文件**（第 1 轮 #2）
  - 核实：属实。`#[cfg(test)] mod tests;` 后面的生产代码会全部漏查
  - 处理：只跳过该属性管的那一项：后续属性、跨行签名、以 `;` 结尾的项或一个花括号块（按去掉字符串和注释后的代码数括号）。新负例 3 条：`mod tests;` 之后的函数、花括号块之后的常量、跨行签名的测试函数之后的常量，都要报
- **P2｜src/xtask/src/model_name.rs:107 正本名读不出时静默漏查**（第 1 轮 #3）
  - 核实：属实。只认同一行的 `ModelId::X => "名字"`；写成 `Self::X` 或换行时只收旧写法，非空检查照样通过
  - 处理：`watched_names` 返回 `Result`：每个旧写法都要找到正本名，否则报错（tidy 报红）。按整段文本匹配，容许换行和 `Self::` 写法。新测试：两个模型、换行、`Self::` 写法都读得出；删掉正本名那一行就报错
- **P2｜src/types/src/desc/mod.rs:418 「全部登记进 ALL」只扫写死的 5 个文件**（第 1 轮 #4）
  - 核实：属实。在 `desc/` 下新加一个描述文件而忘了登记，测试不读它
  - 处理：测试运行时读 `src/desc/` 目录下实际存在的全部 `.rs`。比对改成只在 `ALL` 那一段里找 `&X,`（原来在全文里找，别的数组里出现也算）。抽出 `unlisted` 函数，加负例：新文件的常量没进 `ALL`、只出现在别的数组里，都报出来
- **P2｜src/schema/src/params.rs:66 导出的文本键和解析器行为不一致**（第 1 轮 #5）
  - 核实：属实。`other_keys` 两种格式共用，写着 `name` / `user`，但我的配置不认这两个键；测试也没逐个试
  - 处理：改为按格式、按小节（`top` · `user` · `fine`）分别导出能写的键。测试取所有小节出现过的键加 4 个不该认的键，在每个格式的每个小节里逐个试：列出的都认，没列出的都拒（共 2 个格式 × 3 个小节 × 15 个键）

### 引用抽查

- codex 列的「不符」：无
- codex 列的「核实属实」：4 条
- 无法核实：2 条。① `table.rs` 的全部校验路径：第 1 轮 #5 的新测试逐键试了两种格式的各小节（合法值）。~~越界用例两个小节都有~~——**这句当时写错了**：越界只试了我的配置的 `user` 和场景表的 `fine`，第 2 轮 #3 指出后已补全。② 全量通过数字：改完后本机重跑 `xtask ci` 全绿

### 是否再审

本轮有 P1 → 改完开第 2 轮，提示词附本记录。

### 过程中的意外（如实记）

修第 1 轮 #5 时，我用脚本替换 `src/schema/src/params.rs`，替换范围算错（用了空串），文件被写坏。auto-commit hook 把坏文件提交了一次（12dd5f82）。下一次提交（0403a824）已从 c1236a91 恢复，再重新改。坏版本只在本地分支，没推送；最终内容已逐行对照 c1236a91。

## 第 2 轮（2026-11-02）· 总评「修改」

原文：`_tmp/reviews/012-param-model-schema-codex-2.md`（不入库）。effort medium，耗时 57 秒（`result: P0×0 P1×0 P2×3 P3×0`）。
本轮 P0/P1：0 条 · 停止线：连续第 1 轮无 P0/P1 · 留作 issue 的 P2：无（3 条都已改）。
发出前的小插曲：第 1 轮原文以「P1」开头，`file` 命令把它当成 PBM 位图，隐私扫描因「不是纯文本」拒发。我在原文开头加了一行标题，内容没动，再发出去。这是评审脚本的误判，记在这里，供改 skill 时参考。

上一轮处理的验证结果：codex 抽查 6 条「核实属实」，确认第 1 轮 #1、#3、#4、#5 已修好；#2 只修了一半（见本轮 #1）。

- **P2｜src/xtask/src/model_name.rs:155 属性和项写在同一行时，后面的生产代码仍漏查**（第 2 轮 #1）
  - 核实：属实。`#[cfg(test)] mod tests;` 这一行进入跳过状态后，项已在本行结束，但状态没复位，下一行的生产函数也被跳过；第 1 轮补的负例都是属性和项分两行
  - 处理：跳过逻辑抽成 `test_lines`：本行属性之后的剩余部分当场按「项」处理；开头连写的多个属性逐个剥掉（`strip_attr`，按方括号配对）。新负例：`#[cfg(test)] mod tests;` 与 `#[cfg(test)] #[path = …] mod tests;` 之后的生产代码都要报；新正例：同一行的测试常量不报
- **P2｜src/xtask/src/model_name.rs:44 跨行字符串逃过 R-MODEL**（第 2 轮 #2）
  - 核实：属实。`strings_in` 逐行重置状态，跨行字符串、行尾 `\` 续行拼出的名字都查不到
  - 处理：新写整文件的字符串词法 `string_literals`：跨行、行尾续行（去掉换行和下一行开头的空白）、`r"…"` / `r#"…"#` / `b"…"` 都认；注释与字符字面量 `'"'` 里的引号不开字符串；报告行号取字符串起始行。新负例 4 条（续行、普通跨行、原始字符串跨行、字符字面量之后同行的字符串），新正例 1 条（注释与生命周期里的引号）
- **P2｜docs/reviews/012_param-model-schema-codex.md:32 记录说越界覆盖两个小节，与代码不符**（第 2 轮 #3，含抽查「不符」一条）
  - 核实：属实。越界只试了我的配置的 `user` 和场景表的 `fine`
  - 处理：`out_of_range_values_are_rejected` 改为两种格式 × `user` / `fine` 两个小节 × 每个参数的上下界外一点与边界值。第 1 轮记录里的那句话已划掉并注明

### 引用抽查

- codex 列的「不符」：1 条，即本轮 #3，已改
- codex 列的「核实属实」：6 条
- 无法核实：`table.rs` 的全部路径、全量测试数字。改完后本机重跑 `xtask ci` 全绿

### 是否再审

本轮无 P0/P1，但只连续 1 轮 → 开第 3 轮确认。

## 第 3 轮（2026-11-02）· 总评「修改」

原文：`_tmp/reviews/012-param-model-schema-codex-3.md`（不入库）。effort medium，耗时 65 秒（`result: P0×0 P1×0 P2×1 P3×0`）。
本轮 P0/P1：0 条 · 停止线：**满足**（脚本 `stop-line:` 行：本轮与上一轮都无 P0/P1）· 留作 issue 的 P2：无（这 1 条已改）。
发出前同样在第 2 轮原文开头加了一行标题（以「P2」开头被 `file` 误判成图片）。

上一轮处理的验证结果：codex 抽查 9 条「核实属实」，确认第 2 轮 #1、#2、#3 都已修好，也确认搬移的测试一条没丢（crosscheck 25 条、policy 9 条、原 single_source 10 条、描述表 58 条内容不变）。

- **P2｜src/xtask/src/model_name.rs:305 普通转义让 R-MODEL 漏查**（第 3 轮 #1）
  - 核实：属实。`"\nfastenhancer-m"` 被取成 `nfastenhancer-m`，名字前面紧挨字母 n，边界判断放行；`\t` 同理
  - 处理：字符串词法里按 Rust 的意思还原转义（`\n` `\t` `\r` `\0` `\xNN` `\u{…}`），再判边界。新负例 3 条：`\n`、`\t` 紧挨名字，以及用 `\u{66}` 拼出的名字

### 引用抽查

- codex 列的「不符」：无
- codex 列的「核实属实」：9 条
- 无法核实：`table.rs` 的全部路径、全量测试数字。改完后本机重跑：`xtask ci` 全绿（428 过 / 0 败 / 7 忽略）；`vdn-schema --check` 一致；`eval ci` 与改动前相同；真权重下 vdn-models 17 条全过

### 是否再审

停止线满足，评审结束。本轮那条 P2 改动小、有负例钉住，没再开第 4 轮。

## 第 4 轮（2026-11-02）· 总评「修改」

对象扩大：大总管把 origin/main（含处理图 #69）合进本分支（4f622582），之后我修了合并后的两处红，又做了 issue #68（模型节点带权重版本）。重点说明见 `_tmp/reviews/012-focus-r4.md`（不入库）。
原文：`_tmp/reviews/012-param-model-schema-codex-4.md`（不入库）。effort medium，耗时 90 秒（`result: P0×0 P1×1 P2×3 P3×0`）。
本轮 P0/P1：1 条 · 停止线：未满足 · 留作 issue 的 P2：无。

- **P1｜src/engine/src/pipeline.rs:186 模型名核对能绕过**（第 4 轮 #1）
  - 核实：属实。`with_stages` 只比级串；拿 FE-M 图的 `stages` 去构造跑 GTCRN 的 Pipeline 能成功，原图还能直接交给 `SessionHeader::set_graph`。只有另调 `Pipeline::graph` 才比模型名
  - 处理：新类型 `CheckedGraph`（字段私有），只能经 `ProcessingGraph::verify(model, chain)` 得到；`verify` 一次核对挂点与名字、模型节点 = `Model::name()`、时域段 = 实际级串。`Pipeline::graph` 返回 `CheckedGraph`。`SessionHeader::set_graph` 只收 `CheckedGraph`。~~所以没核对的图在类型上就进不了会话记录~~——**这句当时说过头了**：会话头的 `graph` / `model` 仍是公开字段，可直接赋值；第 5 轮 #1 指出后改为私有。新测试 `foreign_graph_cannot_be_verified_against_running_model`：别的模型的图用正在跑的模型核对报 `ModelName`；级串不同报 `Extra`
- **P2｜src/xtask/src/model_name.rs:55 字符串里的伪 `mod` 声明会豁免生产文件**（第 4 轮 #2）
  - 核实：属实。原来在保留字符串的文本里找 `#[cfg(test)]`
  - 处理：改为在去掉字符串的代码行里定位属性和 `mod`，只从原文取 `#[path]` 的值（且代码里确有 `#[path`）。负例：字符串内容为 `#[cfg(test)] mod real;`，紧接真 `mod real;`，`real.rs` 不进测试名单
- **P2｜src/xtask/src/model_name.rs:68 `#[path]` 写在 `#[cfg(test)]` 前面时认不出**（第 4 轮 #3）
  - 核实：属实
  - 处理：往前收紧挨着的「整行只有属性」的行（`only_attrs`），往后收到 `;` 或 `{`，整项一起解析，不依赖属性顺序。正例：`#[path = "x_tests.rs"]` 在前、`#[cfg(test)]` 在后
- **P2｜src/xtask/src/scan.rs:306 `\u{6_6}` 这种带下划线的 Unicode 转义仍漏**（第 4 轮 #4）
  - 核实：属实
  - 处理：解析前去掉下划线；负例 `"\u{6_6}astenhancer-m"`

### 引用抽查

- codex 列的「不符」：无
- codex 列的「核实属实」：6 条
- 无法核实：会话解析器全部兼容路径、全量测试数字。改完后本机重跑：`xtask ci` 全绿（469 过 / 0 败 / 7 忽略）；真权重 vdn-models 18 条全过

### 是否再审

本轮有 P1 → 改完开第 5 轮。

## 第 5 轮（2026-11-02）· 总评「修改」

原文：`_tmp/reviews/012-param-model-schema-codex-5.md`（不入库）。effort medium，耗时 45 秒（`result: P0×0 P1×1 P2×1 P3×0`）。
本轮 P0/P1：1 条 · 停止线：未满足 · 留作 issue 的 P2：无。

- **P1｜src/types/src/session.rs:40 第 4 轮 #1 的绕过还没封住**（第 5 轮 #1，含抽查「不符」一条）
  - 核实：前一半属实，后一半部分接受
  - 前一半「会话头的 `graph` / `model` 是公开字段，能直接赋值再写出」：属实。处理：两个字段改私有，只能经 `set_graph(&CheckedGraph)` 或读文件（历史记录照常能读）填；读用 `graph()` / `model()`，清空用 `clear_graph()`。仓里没有别处直接写这两个字段
  - 后一半「`Pipeline::with_stages` 只查级串，应改收整张图并在构造时比模型名」：部分接受，不改。理由：
    1. `with_stages` 只收时域段的种类，构造出的 Pipeline 不声称自己是哪张图、跑的哪个模型；它报出的图只能经 `Pipeline::graph(model, artifact)` 拿到，这一步要求模型名一致
    2. 进会话记录（#68 要防的就是「记录里的图和实际不符」）现在只有 `CheckedGraph` 一条路，类型上绕不过
    3. 改成收整张图，引擎里十几处用 `Identity` 等替身模型的测试都会被模型名比对挡住，要另造一套假模型名，改动面大、收益只是把同一次比对提前
    - #68 原文「模型节点名与 `Model::name()` 不一致 → 构造期报错」：图的构造（`verify` / `Pipeline::graph`）就在构造期、非实时，满足
- **P2｜src/xtask/src/model_name.rs:118 外层内联模块里声明的测试模块，路径解析错了**（第 5 轮 #2）
  - 核实：属实。`mod outer { #[cfg(test)] mod real; }` 的文件是 `outer/real.rs`，原来按根层 `real.rs` 豁免
  - 处理：新函数 `inline_prefixes` 按代码行（字符串已去掉）数花括号，记下每行所在的内联模块路径，解析时拼上。负例：内外同名，根层的 `real.rs` 不豁免

### 引用抽查

- codex 列的「不符」：1 条（第 4 轮 #1 的处理结论说过头），已在第 4 轮记录里划掉并注明
- codex 列的「核实属实」：5 条
- 无法核实：全量测试数字。改完后本机重跑 `xtask ci` 全绿（469 过 / 0 败 / 7 忽略）

### 是否再审

本轮有 P1（前一半已改，后一半第一次驳回）→ 开第 6 轮，让 codex 看驳回理由。同一条若第二次被驳回，就停下交大总管 / 人裁决。

## 第 6 轮（2026-11-02）· 总评「修改」

原文：`_tmp/reviews/012-param-model-schema-codex-6.md`（不入库）。effort medium，耗时 56 秒（`result: P0×0 P1×0 P2×2 P3×0`）。
本轮 P0/P1：0 条 · 停止线：连续第 1 轮无 P0/P1 · 留作 issue 的 P2：无。

上一轮处理的验证结果：codex 确认会话头两个字段已私有、读历史记录的入口保留；对第 5 轮 #1 后一半的驳回写了「本轮没有新证据支持重复报告」，没再提。

- **P2｜src/xtask/src/model_name.rs:149 内联模块里的 `#[path]` 仍按顶层解析**（第 6 轮 #1）
  - 核实：属实
  - 处理：`#[path]` 在顶层相对文件所在目录；在内联模块里相对「模块目录 + 内联模块路径」。负例：`mod outer { #[cfg(test)] #[path = "t.rs"] mod tests; }` 解析到 `outer/t.rs`
- **P2｜src/xtask/src/model_name.rs:41 同一文件既被生产 `mod` 引用、又被测试 `mod` 引用时，整份被豁免**（第 6 轮 #2）
  - 核实：属实
  - 处理：`test_module_files` 改为从每一条 `mod 名字;` 声明出发，按属性分成测试引用和生产引用两组，只豁免「只被测试引用」的文件。负例：`mod both;` 与 `#[cfg(test)] #[path = "both.rs"] mod again;` 同指 `both.rs`，不豁免

### 引用抽查

- codex 列的「不符」：无
- codex 列的「核实属实」：5 条
- 无法核实：全量测试数字。改完后本机重跑 `xtask ci` 全绿（469 过 / 0 败 / 7 忽略）

### 是否再审

本轮无 P0/P1，只连续 1 轮 → 开第 7 轮确认。
