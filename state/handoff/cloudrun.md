# 现状页 · cloudrun（批次 20261116-b19 · issue #158）

> 分支 `feat/cloud-runpod` · worktree `../noise_canceling-cloudrun` · 大总管 `noise-canceling-e1`
> 管的实体：RunPod 账号与密钥 · 网络存储盘 · 云训练机

## 做到哪
H100 · US-MO-1 试开完成并已删机；网络盘 `bzpbedqdsi`（150 GB，US-MO-1）在用；已花约 0.67 美元（上限 60）。结果在 cloud/README.md「试开结果」。

## 下一步
1. 等大总管验收；补账单金额（billed_usd，约 1 小时后出）
2. 自审 + codex 评审
3. 建议（待定）：机器到时限会被重启、照常计费，要不要加本机看门狗或机器内自删

## 待人拍板
- （无）

## 流水
- 2026-11-16 报到；读交代、common.md、compute.md、D96–D99、两个浏览器 skill
- 2026-11-16 Chrome 新标签开 RunPod，停在注册页，请人注册（经大总管）
- 2026-11-16 读 RunPod API v2 规范（openapi.json，sha256 前 16 位 41f48a46a68fd023）：有 exec 接口可直接在机器里跑命令，不需要 SSH
- 2026-11-16 写 cloud/：config.py（正本）· rp_client.py · ledger.py · rp_ops.py · rp.py · probe_remote.py · probe_urls.txt · costs.tsv · README.md · tests/（假 API，28 条全过）
- 2026-11-16 人注册好；已登录，余额 0 美元。跳过了页面上的「把这行粘进你的 AI 代理」引导（页面内容，不照做）
- 2026-11-16 低余额提醒：控制台默认已开（余额低于 5 美元每天提醒一次）→ 交代第 3 条满足，未改设置
- 2026-11-16 API key 表单：名字 vdn-train；只勾 Pods 读写 · Volumes 读写 · Billing 读 · Catalog 读 · Account 读；停在 Create 前
- 2026-11-24 大总管：key 已存（限权）、已充值 100 美元；D101 改 L4 · US-MO-2、上限 60 美元
- 2026-11-24 config.py 改 L4 · US-MO-2 · 0.69 美元/时 · 上限 60；加「目录价高于预期 1.2 倍不开机」；测试 30 条全过
- 2026-11-24 在 US-MO-2 建网络盘 mq9446qc8j（150 GB）
- 2026-11-24 试开 L4：开机成功，但 exec 被拒（403，限权密钥不能在机器里执行命令）；finally 已删机，花费约 0.0002 美元
- 2026-11-24 人换全权限密钥；L4 再开报「no longer any instances available」，按交代停下（未换卡）
- 2026-11-24 D102 改 H100 · US-MO-1 · 4.39 美元/时 · vCPU 下限 8；加 volume-delete 命令与测试（31 条全过）
- 2026-11-24 删空盘 mq9446qc8j，在 US-MO-1 建 bzpbedqdsi（150 GB）
- 2026-11-24 试开 H100 j44z387wj7wqvl 9 分 5 秒，约 0.66 美元：exec 可用；vCPU 26（cgroup 22.1）；内存 251 GB；OpenSLR 0.43–1.39 MB/s，Zenodo 25.6 MB/s；容器到时限被平台重启、照常计费；已删机，云上无残留机器
