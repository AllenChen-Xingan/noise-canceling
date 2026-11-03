# 现状页 · schema（批次 20261102-b9，issue #57）

> 手下 `schema`，worktree `../noise_canceling-schema`，分支 `feat/schema-crosscheck`。大总管 `noise-canceling-99`。

## 做到哪
- [x] 读交代 · 报到（大总管同意：新 crate `vdn-schema` · 导出到 `specs/schema/` · 逐字比对挂 cargo test + xtask ci）
- [ ] 机制：`vdn-schema` crate（tool 档）· 导出命令 · 逐字比对测试
- [ ] 共享内存头 schema + 逐项比对 + 版本号守卫
- [ ] Config schema + 示例文件过 schema + 往返测试
- [ ] 评测任务清单 4 份文件的 schema + 过 schema 测试 + task-schema.md 改指针
- [ ] 规约 ADR 004 表 · cog.md 一行 · 全量 CI
- [ ] codex 换谱系评审

## 技术决定（我定，写明理由）
- 导出库 `schemars` 1.x（MIT，默认 draft 2020-12，读 serde 属性，派生即正本）
- 校验库 `boon` 0.6（MIT OR Apache-2.0，支持 2020-12）：只做 `vdn-schema` 的 dev-dependency，不进任何产品 crate。理由：校验器和导出器是两家实现，能抓到「schemars 写的 schema 与 serde 实际行为不一致」
- 共享内存头核心 crate 不引 serde：字段表写成 `vdn-shm` 里的常量表（`offset_of!` / `size_of` 算），用小宏穷举解构，保证漏字段编译失败

## 下一步
建 `src/schema/`。

## 转交
（暂无）

## 待拍板
（暂无）
