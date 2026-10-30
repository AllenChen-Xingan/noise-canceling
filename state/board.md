# Mac 开源实时语音降噪 · 状态板（给 AI · 跨会话唯一接续点）

> 开工先读 `CLAUDE.md` + **`.42cog/` 四份**（`intent` · `real` · `cog` · `meta`）+ 本文件 + `state/memory/MEMORY.md`。
> **非轮规则：每轮有效工作必更新本文件**（倒序追加，新的在上，带日期与 commit hash）。
> 每完成一个可命名的逻辑单元存一次；破坏性操作之前也存一次。这是给你自己留的后路，不是给别人看的历史。

> ## 🔧 初始化补全（2026-10-29 第一轮 · d54c186）
> - 起草 `.42cog/intent.md`（收敛方向 + 三把尺子：清楚 · 自然 · 轻快）、`real.md`、`cog.md`、`meta.md`
> - 设计文档移入 `vault/raw/20261029-voice-denoise-design.md`；telegram 调研移入 `docs/research/001_telegram-aias/`
> - `scripts/check-tools.sh` 补 Rust 与音频工具（缺 sox）；`rust-toolchain.toml` 锁 1.99.1；`.github/workflows/ci.yml` 最小闸门
> - 收敛方向人已确认（2026-10-29）；自动提交 hook 上线（改动超过 5 行即提交，见 CLAUDE.md 铁律 14）
> - **待人定**：① 个性化（只保留主人声音）：推后 / 核心 / 不做（默认推后）② 远程仓库地址 ③ 放权边界
> - **下一步**：`vdn-research` 三个决策——起步模型与代码 · Mac 音频管线 · 评测工具链；然后 `vdn-proto` 定标「好的会议声音」
> - 系统做轻量版（参照 `docs/research/001_telegram-aias/` 第五节）：状态只记 board + handoff；评审设轮数上限

> ## 🌱 系统开仓（2026-10-29 第一轮）
> `aias-meta-init` 生成六组骨架：① `README.md` `CLAUDE.md` `.42cog/` `specs/` ② `vault/` `notes/` `resources/` ③ `skills/` `scripts/` `plugin.json` ④ `src/` ⑤ `state/` `docs/` ⑥ `_build/` `_tmp/` `_archive/`（忽略）。
> **收敛方向**（草稿，待人确认）：见 `.42cog/intent.md`——**那句话只有一份，别抄到这里来**。
> **下一步**：确认收敛方向 → `vdn-research` 找依据、排真相源权重 → 回来改这一句。方向第一版粗是正常的，四步是循环。
