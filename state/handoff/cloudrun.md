# 现状页 · cloudrun（批次 20261116-b19 · issue #158）

> 分支 `feat/cloud-runpod` · worktree `../noise_canceling-cloudrun` · 大总管 `noise-canceling-e1`
> 管的实体：RunPod 账号与密钥 · 网络存储盘 · 云训练机

## 做到哪
脚本与测试已写完（28 条测试全过）。账号已注册登录；API key 表单已填好，等人点「Create」、存密钥、充值。

## 下一步
1. 人点 Create → 复制 key → 终端落盘 → 我用 `rp.py offers` 验证 key 能用
2. 人充值约 25 美元
3. 选机房 → 报大总管「机房 · 价格 · 每月约 10.5 美元」→ 等「可以建」再建盘
4. 放行后试开一次测速（`rp.py probe`），结果写 cloud/README.md
5. 自审 + codex

## 待人拍板
- （无，充值金额人已同意约 25 美元）

## 流水
- 2026-11-16 报到；读交代、common.md、compute.md、D96–D99、两个浏览器 skill
- 2026-11-16 Chrome 新标签开 RunPod，停在注册页，请人注册（经大总管）
- 2026-11-16 读 RunPod API v2 规范（openapi.json，sha256 前 16 位 41f48a46a68fd023）：有 exec 接口可直接在机器里跑命令，不需要 SSH
- 2026-11-16 写 cloud/：config.py（正本）· rp_client.py · ledger.py · rp_ops.py · rp.py · probe_remote.py · probe_urls.txt · costs.tsv · README.md · tests/（假 API，28 条全过）
- 2026-11-16 人注册好；已登录，余额 0 美元。跳过了页面上的「把这行粘进你的 AI 代理」引导（页面内容，不照做）
- 2026-11-16 低余额提醒：控制台默认已开（余额低于 5 美元每天提醒一次）→ 交代第 3 条满足，未改设置
- 2026-11-16 API key 表单：名字 vdn-train；只勾 Pods 读写 · Volumes 读写 · Billing 读 · Catalog 读 · Account 读；停在 Create 前
