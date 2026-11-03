# 现状页 · iface（批次 20261102-b9 第二阶段，issue #65）

> 手下 `iface`，worktree `../noise_canceling-iface`，分支 `feat/param-model-schema`。大总管 `noise-canceling-99`。
> 管的实体：参数描述（`desc.rs` 的导出）· 场景参数表 / 我的配置的文本格式 · 模型接口（`vdn-models`）。

## 做到哪
- [x] 读交代 · 报到（大总管同意顺序）
- [x] 读代码，定方案（见下）
- [ ] 拆 600 行以上的文件：`desc.rs`（930）· `policy.rs`（688）· `src/schema/tests/crosscheck.rs`（870）（大总管 2026-11-02 追加）
- [ ] 参数描述 → `specs/schema/params.table.json`
- [ ] 文本格式：我的配置越界拒收（现在只查有限，**不查范围**——先写会红的测试）；仓里文本文件过检查
- [ ] 模型接口：`vdn-models` 声明输入输出名与形状 → `specs/schema/models.interface.json`；ONNX 实读比对（`#[ignore]`，本机实跑）
- [ ] FE-M 名字只留一个
- [ ] 全量 CI · eval ci 对比 · codex 评审 · PR 草稿

## 方案（我定的技术题，理由）
1. **参数描述导出**：`vdn-schema` 新模块 `params.rs`，把 `desc::ALL` 转成 `params.table.json`（不是 JSON Schema，按 README 规矩 1 命名为 `<实体>.<种类>.json`）。导出函数里对 `ParamDesc` 做穷举解构：描述加字段没导出 → 编译失败。浮点按 f32 最短写法写出（`0.002` 不变成 `0.0020000000949…`）。文件还写出「文本格式里能出现的参数键」（= `ParamField::ALL` 的键），由同一张表派生
2. **文本格式**：仓里目前**没有**已提交的场景参数表 / 我的配置文件（`git grep` 文件头为空）。加两份示例进 `specs/schema/examples/`，测试遍历仓里所有带这两种文件头的已跟踪文件，逐个解析（越界、未知键即红）。以后谁提交这类文件都会被查到
3. **模型接口**：声明（采样率 · hop · 延迟 · 输入输出名与形状）放 `vdn-models` 的 `ModelKind`，适配器打开文件时按声明核对（替换 gtcrn.rs 里手写的名字表）。导出 `models.interface.json`；CI 比对声明 ↔ 导出文件；ONNX 实读 ↔ 声明写成 `#[ignore = "要仓外权重"]`
4. **FE-M 名字正本 = `fastenhancer-m`**（连字符）。理由：命令行、配置 JSON、评测结果文件（`baseline-ci.json` 81 个键）、denoised 都用它；下划线写法只出现在 `ModelId::as_str` 一处，仓里没有用它写的场景表 / 会话记录文件。旧写法 `fastenhancer_m` 只在 `ModelId::parse` 一处兼容

## 转交 / 等大总管
- `vdn-cli` 的 `ModelArg`（`src/cli/src/main.rs:186-198`）写死了 `"fastenhancer-m"`：想改成从 `ModelId` 取。已报大总管，等回复
- 「正本以外不许出现新写法」想加进 `xtask tidy`（8d 在改）：已报大总管，等回复

## 待拍板
（无）
