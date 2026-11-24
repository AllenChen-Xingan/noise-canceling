# 现状页 · cloudrun（批次 20261116-b19 · issue #158）

> 分支 `feat/cloud-runpod` · worktree `../noise_canceling-cloudrun` · 大总管 `noise-canceling-e1`
> 管的实体：RunPod 账号与密钥 · 网络存储盘 · 云训练机

## 做到哪
防忘删机两层已做完并实测（机器内自删 1 次成功；本机看门狗起停正常）；纯 CPU 机器支持已写好未开机，等大总管放行报价。累计花费约 0.83 美元（上限 60）。

## 下一步
1. 等大总管放行 CPU 下载机；补账单金额（billed_usd）
2. 自审 + codex 评审；开 PR（Closes #158）前报大总管

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
- 2026-11-24 大总管 noise-canceling-33（重启后）要求：两层防忘删机 · OpenSLR 下载方案报价
- 2026-11-24 写 watchdog.py：本机看门狗（登记表 .watch/，caffeinate -i，空表即退出）+ 机器内自删（平台注入的 RUNPOD_POD_ID / RUNPOD_API_KEY，只删自己）；delete_pod 统一注销登记；cloud/.gitignore 忽略 .watch/
- 2026-11-24 实测自删：H100 7mfuc62ykdmrm3 设时限 108 秒，约 108 秒后自己消失（接口 404）；机器内确有 RUNPOD_API_KEY、RUNPOD_POD_ID（只看了变量名）；补记删机，约 0.15 美元；看门狗随后自行退出
- 2026-11-24 查 CPU 目录：US-MO-1 cpu3c 库存 HIGH，0.03 美元/核/时；OpenSLR 28 页面无 MD5、无美国镜像（主链接即 trmal，另有 elda、CN magicdatatech）
- 2026-11-24 加 --cpu（cpu3c × 4 核，挂同一块盘）与测试；测试共 44 条全过
