# Mac 开源实时语音降噪 · 状态板（给 AI · 跨会话唯一接续点）

> 开工先读 `CLAUDE.md` + **`.42cog/` 四份**（`intent` · `real` · `cog` · `meta`）+ 本文件 + `state/memory/MEMORY.md`。
> **非轮规则：每轮有效工作必更新本文件**（倒序追加，新的在上，带日期与 commit hash）。
> 每完成一个可命名的逻辑单元存一次；破坏性操作之前也存一次。这是给你自己留的后路，不是给别人看的历史。

> ## ✅ 第一批调研收尾与拍板（2026-10-30）
> - 002 / 003 / 004 三条线全部验收、收尾、关窗；27 题已拍板 → `docs/plan/001_decisions-20261029-b1.md`
> - ⚠️ 与建议不同的两条：**许可证 GPL-3.0**（D04）· **个性化定为核心**（D08）
> - 已回写：三份取舍卡勾选、三份 must-know 命门改写（待人过目）、删 `resources/P.808`
> - **人要做**：装 BlackHole（`brew install blackhole-2ch` + 重启）· 过目三份命门 · 给远程仓库地址 · 定放权边界
> - **下一步**：经人同意后派 005 PSE 调研；同时可开 M0 直通与「DPDFNet-2 vs GTCRN」离线 AB 听测（D01）
> - 未回写：根目录 `LICENSE`（等远程仓库）· `.42cog/real.md` 许可证一条（留人过目）

> ## ⚠️ 派活巡检（2026-10-29 20:38 · 36949e2）
> - 三个手下**都没跑起来**：`model` 窗口卡在 Claude Code「是否信任此文件夹」弹窗（17:35 起）；`pipeline`、`evaltool` 窗口不存在（派活在第一个窗口后中断）
> - `docs/research/002–004` 仍是 17:33 的空模板；`state/handoff/` 为空
> - **教训**：起手下前先让人在本目录信任一次（或确认已信任），并逐个核对窗口数 = 派活数
> - **20:5x 已重派**：人确认信任目录后三窗都起；三个都已报到；大总管名改为 noise-canceling-69
> - **21:05 003 pipeline 收尾**（验收通过、窗口已关）：开发期 cpal + BlackHole，最终自写 C 回环插件，推理不进插件；交付 `docs/research/003_mac-audio-pipeline/`
> - **21:12 004 evaltool 收尾**（验收通过、窗口已关）：每把尺子主指标 + 交叉指标，分 CI / 本机两层，回归门禁形状已定、数值待人
> - **21:20 002 model 收尾**（验收通过、窗口已关）：GTCRN 为不拍板时的默认，DPDFNet-2 暂定首选需人放宽延迟上限
> - **本批三线全部收尾**；**下一步**：待定清单（`state/coord.md`）分组问人

> ## 🔧 初始化补全（2026-10-29 第一轮 · d54c186）
> - 起草 `.42cog/intent.md`（收敛方向 + 三把尺子：清楚 · 自然 · 轻快）、`real.md`、`cog.md`、`meta.md`
> - 设计文档移入 `vault/raw/20261029-voice-denoise-design.md`；telegram 调研移入 `docs/research/001_telegram-aias/`
> - `scripts/check-tools.sh` 补 Rust 与音频工具（缺 sox）；`rust-toolchain.toml` 锁 1.99.1；`.github/workflows/ci.yml` 最小闸门
> - 收敛方向人已确认（2026-10-29）；自动提交 hook 上线（改动超过 5 行即提交，见 CLAUDE.md 铁律 14）
> - **待人定**：~~① 个性化~~（2026-10-30 定为核心，D08）② 远程仓库地址 ③ 放权边界
> - **下一步**：`vdn-research` 三个决策——起步模型与代码 · Mac 音频管线 · 评测工具链；然后 `vdn-proto` 定标「好的会议声音」
> - 系统做轻量版（参照 `docs/research/001_telegram-aias/` 第五节）：状态只记 board + handoff；评审设轮数上限

> ## 🌱 系统开仓（2026-10-29 第一轮）
> `aias-meta-init` 生成六组骨架：① `README.md` `CLAUDE.md` `.42cog/` `specs/` ② `vault/` `notes/` `resources/` ③ `skills/` `scripts/` `plugin.json` ④ `src/` ⑤ `state/` `docs/` ⑥ `_build/` `_tmp/` `_archive/`（忽略）。
> **收敛方向**（草稿，待人确认）：见 `.42cog/intent.md`——**那句话只有一份，别抄到这里来**。
> **下一步**：确认收敛方向 → `vdn-research` 找依据、排真相源权重 → 回来改这一句。方向第一版粗是正常的，四步是循环。
