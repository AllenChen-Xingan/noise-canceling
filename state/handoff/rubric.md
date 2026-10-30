# 现状页 · rubric（批次 20261030-b2）

状态：进行中

## 手上
007 降噪 rubric：评分标准（`specs/rubric/denoise.md`）+ 用它筛候选（`docs/research/007_denoise-rubric/decision.md`）。交代 `_tmp/20261030-b2/rubric.md`（主目录）

## 做到哪
读完前置；与 abtest 对齐 AB 页字段；Workflow 调研 4 路开跑

## 下一步
等调研 → 写 rubric 正文 → 筛选表 → 三视角自审 → 收尾

## 待人拍板
（写 rubric 时列出：权重、否决线、人耳流程、绝对可用性题与设计稿「不打绝对分」的张力）

## 流水
- 2026-10-30 读交代 + common；报到大总管（会话名：abtest=noise-canceling-exp003-2d · pse=noise-canceling-45 · sota=noise-canceling-sota-4f）
- 2026-10-30 读 intent · 001 决策 · 002 路线图 · 004 decision/must-know · exp001/002 · 002 decision · 005 decision · 设计稿 §3/§9/§13 · abtest 分支的 ab_page.html / build_page.py
- 2026-10-30 与 abtest 对齐：现有三问（clearer/natural/eaten）+ 两道陷阱题原样采用；abtest 同意加 usable_A/B（能用/勉强/不能用）、tags_A/B（metallic/pumping_bg/muffled/vacuum/clipped_onsets/pumping_voice）、started_at/answered_at/plays/switches；VoxLock 本批进（终版三候选）；score.py 已用 Qwen3-ASR 1.7B，scores.json 含 noisy 列（del/sub/ins_rate）。已抄送大总管
- 2026-10-30 报 Workflow 计划给大总管，开跑
