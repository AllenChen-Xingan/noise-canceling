# 「AST / 统一表示」做了之后：好处 · 复杂度 · 扩展方向

> 分析员（子代理），2026-11-02。只读代码，未编译、未跑测试（机器上有人在做计时基准）。
> 对比：做之前 = `26baa80`；做之后 = `origin/main` = `0604143`（含 PR #60、PR #63）。
> 两次之间 main 上只合了这两个 PR【测】`git log --first-parent 26baa80..origin/main`。
> 来源标记：【测】我量出来的 ·【读】读代码文档 ·【推】推断。

## 结论

- **「同一件事写几处」的复杂度确实降了**：参数默认值从 6 处降到 1 处，镜像类型从 13 个降到 0 个【测】。
- 手写、无校验的 JSON 数据文件从 4 份降到 0 份；5 份 schema 全部由 Rust 导出并逐字比对【测】【读】。
- **代价是体量涨了**：`src/` 的 Rust 行数 +13.6%，多 1 个 crate，锁文件多 48 个依赖包【测】。
- 净判断：改一个东西要动几处、会不会悄悄不一致，这类复杂度降了；读代码要懂的概念多了。前者更伤人，所以净值为正【推】。
- 证据强度：中。结构数字是实测；「开发更快、bug 更少」没有数据，未量化。

「代码语法树守规矩」那一半：tidy 是**逐行文本检查**，不是语法树解析【读】。
`src/xtask/Cargo.toml` 只依赖 `serde_json`，没有 `syn`；ADR 004 原文就写「文本 / 轻语法」，`ast-grep` 留作第二阶段。

## 一、已经兑现的好处（代码里已有，不是计划）

1. **镜像副本删光**【测】【读】
   - `src/speaker/src/types.rs` · `src/polish/src/mirror.rs` · `src/scene/src/mirror.rs` 三份删除，共 341 行。
   - 三个 crate 改为直接依赖 `vdn-types`。
   - tidy 新规则「不许镜像」（R-MIRROR）挡新增：不许有 `mirror.rs`，不许与 `vdn-types` 公开类型同名。
2. **参数有了一张正本表**【读】`src/types/src/desc.rs`
   - 58 个参数，每个带：稳定编号 · 键名 · 单位 · 最小 · 最大 · 默认 · 说明。
   - 之前只有 `ParamSet` 的 6 个参数有成文范围（`ParamField::range`）【测】。
   - 现在 58 个都有，speaker / polish 的参数第一次有了范围。
3. **越界值第一次在更多入口被拦下**【读】
   - 衰减下限（`floor_db`）以前只在 `ParamSet` 查范围。
   - engine 的 `SafetyConfig`、speaker 门控与包络、命令行都不查【读】。
   - 现在 5 个使用方都走 `FLOOR_DB.check / clamp`。
   - 这是行为上的安全收益，不只是整洁。
4. **tidy 挡范围字面量**（R-PARAM）【读】`src/xtask/src/single_source.rs`
   - 在描述表之外，一行里同时出现参数名和数字范围就报红。
5. **数据文件有了 schema 与交叉校验**【读】`specs/schema/README.md`、`src/schema/tests/crosscheck.rs`
   - 共享内存头：字段名 · 偏移 · 大小与 `offset_of!` 逐项比对。
   - 改布局不升版本 → 红；版本历史只许末尾追加。
   - Config：示例过 schema；范围边界与 `Config::validate` 逐例一致。
   - 评测任务清单：`ci.json` · `qa.json` · `levels.json` · `baseline-ci.json` 都过 schema。
   - 坏任务（拼错键、未知 verifier）schema 与解析器判定一致。
   - 校验器用另一家实现 `boon`，能抓到「schema 与 serde 行为不一致」。
6. **文档里的第二份字段表删了**【测】
   - `docs/experiments/exp007-eval-suite/task-schema.md` 删掉 37 行手写字段表，改成指针。
7. **评测清单解析器变薄**【测】
   - `src/eval/src/suite.rs` 397 → 333 行；格式交给 `files.rs` 的 serde 类型（12 处 `deny_unknown_fields`）。
8. **测试变多**【测】`#[test]` 349 → 415 个（+66）。

## 二、量化对比

| 指标 | 之前 26baa80 | 之后 origin/main | 变化 | 来源 |
|---|---|---|---|---|
| 同名类型多处定义（全仓） | 34 个名字 | 24 个名字 | −10 | 【测】find-duplicates |
| 其中 `src/` 内 ≥2 处 | 27 个名字 · 60 处 | 16 个名字 · 41 处 | −11 名字 · −19 处 | 【测】 |
| 其中镜像类（`mirror.rs` / `speaker/types.rs` 引起） | 13 个名字 | 0 | −13 | 【测】 |
| `floor_db` 默认值写死的位置 | 6 个文件 | 1 个（`desc.rs`） | −5 | 【测】grep |
| `floor_db` 范围定义位置 | 1 处（只管 `ParamSet`） | 1 处（管 5 个使用方） | 覆盖 1 → 5 | 【测】【读】 |
| `floor_db` 出现的文件数（`src/`） | 21 | 22 | +1 | 【测】 |
| `floor_db` 出现次数（`src/`） | 88 | 165 | +77 | 【测】 |
| `open_db` 默认值位置 | 5 处 | 1 处 | −4 | 【测】 |
| `hangover_s` 默认值位置 | 6 处 | 1 处 + 1 个测试夹具 | −4（产品代码 −5） | 【测】 |
| 有成文范围的参数 | 6 个 | 58 个 | +52 | 【测】 |
| 手写、无校验的 JSON 数据文件 | 4 份 | 0 份 | −4 | 【测】 |
| 有 schema 交叉校验的数据文件 | 0 份 | 6 份（4 份清单 + 2 份配置示例） | +6 | 【测】 |
| 由 Rust 导出、逐字比对的 schema | 0 份 | 5 份 | +5 | 【测】 |
| `src/` Rust 总行数 | 32,951 | 37,438 | +4,487（+13.6%） | 【测】 |
| 其中产品代码（粗分） | 约 24,113 | 约 26,578 | +2,465（+10.2%） | 【测】按首个 `#[cfg(test)]` 切 |
| 其中测试 / 示例（粗分） | 约 8,976 | 约 11,005 | +2,029 | 【测】 |
| crate 数 | 19 | 20 | +1（`vdn-schema`） | 【测】 |
| `src/` 改动 | — | 64 个文件 · +5,322 / −788 行 | 净 +4,534 | 【测】git diff --stat |
| 超过 600 行的文件 | 11 | 14 | +3 | 【测】 |
| `Cargo.lock` 包数 | 204 | 252 | +48 | 【测】 |
| `#[test]` 个数 | 349 | 415 | +66 | 【测】 |

`floor_db` 出现次数变多是好事【读】：新增的多是 `FLOOR_DB.check / clamp / default` 引用和测试。
字面量少了，引用多了。「出现的文件数」这个指标本身衡量不了复杂度。

### 改一个参数，要改几个文件

衰减下限（`floor_db`），同时改默认值和范围【测】：

- 之前：6 个文件。`types/params.rs` · `engine/engine.rs` · `cli/main.rs` · `speaker/gate.rs` · `speaker/envelope.rs` · `scene/mirror.rs`。
- 之前只改范围：1 个文件，但只有 `ParamSet` 生效，其余 4 个使用方不查范围。
- 之后：1 个文件（`src/types/src/desc.rs` 第 208 行），5 个使用方都跟着变。

人声停后保持（`hangover_s`），同时改默认值和范围【测】：

- 之前：6 个文件。`types/params.rs` · `speaker/envelope.rs` · `scene/mirror.rs` · `engine/engine.rs` · `cli/main.rs`（150 ms）· `dsp/envelope.rs` 测试夹具。
- 之后：1 个文件。`dsp/envelope.rs:132` 的测试夹具仍写 `0.15`，只影响测试。`vdn-dsp` 不依赖 `vdn-types`。

反例：配置的块长范围（`Config::BLOCK_RANGE`）【测】

- 之前 1 处，之后仍是 1 处 Rust 常量。
- 之后多了一步：跑 `cargo run -p vdn-schema` 重导 `config.schema.json` 并提交。
- 忘了重导会被测试挡下。这里改动成本没降，换来的是外部数据文件有了校验。

## 三、复杂度增加的地方（诚实账）

1. **新代码体量**【测】
   - `vdn-schema`：+1,492 行，其中测试 `crosscheck.rs` 870 行。
   - `xtask` 新规则：+794 行（`single_source.rs` 789 行）。
   - `vdn-types`：+1,132 行，主要是 `desc.rs` 930 行（58 条参数数据 + 105 行测试）。
   - 镜像删了 341 行；scene crate 净减 139 行，是唯一净减的 crate。
2. **新概念要学**【读】
   - 参数描述 `ParamDesc` 与单位 `Unit`，以及「编号永不复用」的规矩。
   - tidy 两条新规则和两种豁免注释（`tidy: 同名不同物` · `tidy: 非参数范围`），全仓已用 13 处【测】。
   - schema 的工作流：改类型 → 重导 → 提交；`--check`；布局指纹登记表 `HISTORY`。
   - CI 的可信基线环境变量 `VDN_SCHEMA_BASE`，和 `fetch-depth: 0`（拉完整历史）。
3. **新依赖**【测】`schemars`（C 档 / tool 档用）、`boon`（只在测试）；锁文件多 48 个包。
4. **新的手写表**【测】
   - speaker 4 个参数结构、polish 1 个，各写一张「描述 ↔ 取值 / 赋值」对照表。
   - 这让查重脚本多报了 `Get` / `Set` 两个名字（5–6 处，都是测试里的局部类型别名）。
   - 这些表是校验本身，但写法重复 5 次【推】。
5. **文件超长**【测】`desc.rs` 930 · `crosscheck.rs` 870 · `single_source.rs` 789 行，都超过 600 行上限（#62 在管）。
6. **规则的盲区**【读】`single_source.rs` 第 11 行
   - R-PARAM 只抓「同一行里有参数名 + 数字范围」；默认值字面量 tidy 不查，靠各 crate 单测比对。
   - 逐行文本匹配，换行写法、宏里的写法可能漏【推】。
7. **范围是 AI 定的**【读】`desc.rs` 头注释写「范围待标定」。越界现在会被拒收，范围定窄会误拦合法值【推】。

## 四、以后怎么扩展

每条都写：把哪种「乘法」变「加法」 · 要加什么 · 收益 · 代价 · 前置条件。

| # | 方向 | 乘法 → 加法 | 要加的节点或 schema | 收益 | 代价 | 前置 |
|---|---|---|---|---|---|---|
| 1 | 处理图描述（#64） | N 种处理节点 × M 个入口（命令行实时 · 评测离线 · 守护进程 · speaker-lab）各自拼链 → 每个入口读同一张图 | `vdn-types` 加图描述：节点 = 种类 · 参数（引用 `ParamDesc` 编号）· 版本；整图有内容 ID；导出 `graph.schema.json`；建好的 Pipeline 与描述比对名字 / 顺序 / 延迟 | 新加一种处理 = 加一种节点；入口不改 | 约 500–1,000 行【推】；`StageChain` 要能报出自己的结构 | #64 合并 |
| 2 | 会话记录记图 ID → 回放 | K 种线上问题 × 每次手工复现 → 一个回放器 | `SessionHeader` 加图内容 ID 与模型 ID；记录导出 schema；评测加「按记录回放」任务种类 | 用户说「这里吃字了」，能按当时的图和参数离线重跑 | 图 ID 必须稳定；记录不含音频（D19），回放要配合本地片段 | 方向 1 |
| 3 | 参数描述导出 + 一张表生成取值 / 赋值（#65 前半） | P 个参数结构 × 6 处表示（`Default` · 范围检查 · 夹值 · 测试对照表 · 命令行默认 · 文本格式）→ 1 张表 | `desc.rs` 导出 `params.schema.json`；场景参数表 / 我的配置的文本格式与之比对；5 张手写对照表换成一个 `macro_rules!` | 加一个参数只写一行；手写对照表消失 | 宏降低可读性；ADR 004 的触发条件（参数超过约 10 个）已满足：现有 58 个 | #65 |
| 4 | 模型接口三方一致（#65 后半） | N 个模型 × 3 处声明（ONNX 文件 · `vdn-models` 声明 · 命令行 / 评测名字）→ 1 个正本 + 2 条比对 | `model-io.schema.json`：输入输出名 · 形状 · 采样率；FE-M 名字只留一个正本 | 换模型 = 换一片叶子，形状错在 CI 就红 | 无权重时只能 `#[ignore]` + 人工步骤 | #65 |
| 5 | 在图上做静态查询与校验 | 每加一条检查 × 每个入口各写一遍 → 在图上算一次 | 节点带延迟、实时档位、所需采样率；图上算总延迟并对比预算（现链路 36 ms，D74）、查实时安全、导出 Mermaid 图 | 不跑音频就能判「这张图超不超延迟预算」；人能看到处理链长什么样 | 节点元数据要和实际实现比对，否则又多一份手写表 | 方向 1 |
| 6 | 场景路由挂在图上 | S 个场景 × P 个参数 × 各自校验 → 场景表只引用参数编号 | 场景参数表引用 `ParamDesc` 编号与图 ID，过 schema；路由切换 = 换图或换参数覆盖层 | 场景表写错键或越界，加载前就拒收 | 场景类目还没定（005 第三节） | 方向 1、3 |
| 7 | 评测任务引用图 | T 个任务 × G 种配置手写组合 → 任务 × 图 ID 自动展开 | 任务清单加 `graph` 字段，替代现在的 `safety: bool` 等开关；schema 跟着导出 | 对比实验 = 换图 ID；结果能按图版本追溯 | 已提交的基线结果要兼容旧格式 | 方向 1 |
| 8 | 守规矩从逐行文本升级到语法树 | 每条规则 × 每种写法（换行 · 宏 · 别名）各补一个特例 → 按语法结构匹配一次 | xtask 引 `syn` 或用 `ast-grep`；R-PARAM 加查默认值字面量；R-MIRROR 改按类型结构判 | 豁免注释和漏报变少 | xtask 编译变慢；按 ADR 004，同一类漏网犯到第三次再上 | 漏网记录 |

建议的先后【推】：1 → 3 → 2 → 5，其余跟着需求走。方向 1 是其他方向的地基。

## 五、复现命令

```sh
M=/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling
git -C $M fetch -q origin
git -C $M worktree add --detach <before> 26baa80
git -C $M worktree add --detach <now> origin/main

# 同名类型（统计 src/ 内 ≥2 处的名字）
python3 /Users/allenchen/xingan-chen/B-MyCreate/2-aartifacts/skills/single-source-truth/scripts/find-duplicates.py <dir> --only types --json

# 改动量与行数
git -C <now> diff --stat 26baa80 origin/main -- src/ | tail -1
find <dir>/src -name '*.rs' | xargs cat | wc -l
find <dir>/src -name '*.rs' -exec wc -l {} + | awk '$1>600 && $2!="total"' | wc -l
grep -r '#\[test\]' <dir>/src | wc -l
grep -c '^name = ' <dir>/Cargo.lock

# 参数散布
grep -rl floor_db <dir>/src | wc -l
grep -rn 'floor_db\s*[:=]\s*-\?[0-9]\|hangover_s\s*[:=]\s*[0-9]\|open_db\s*[:=]\s*-\?[0-9]' <dir>/src
grep -c '^pub const [A-Z_]*: ParamDesc' <now>/src/types/src/desc.rs

# 数据文件
git -C <dir> ls-files '*.json' | grep -v '^docs/'

# 收尾
git -C $M worktree remove <before>; git -C $M worktree remove <now>
```

「产品代码 / 测试」的粗分：每个 `.rs` 文件按第一个 `#[cfg(test)]` 切开，`tests/` · `examples/` 整个算测试。只是近似值。
