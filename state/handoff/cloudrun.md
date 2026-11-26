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
- 2026-11-24 大总管放行 CPU 下载机（cpu3c，US-MO-1，最长 6 小时）；读了 traindata 的 cloud_steps.sh、fetch.py、sources.json（A1–A5 + E2 共约 18.1 GB）
- 2026-11-24 开 CPU 机 vggy9ggvbi3h1p（4 核，0.12 美元/时）；发现 RUNNING 后容器还要约 1 分钟才能 exec（409），加 wait_exec_ready 与测试
- 2026-11-24 传 traindata 仓库包（git archive fc37e9ba，22 MB，sha256 核对一致）；装 aria2/unzip/curl；setup 通过
- 2026-11-24 test：2 个模块导入失败（rooms.py 缺 EARLY_REVERB_MS / early_rir，只在 evalcmp 分支有）→ 停下报大总管，等选 A（跳过继续下载）或 B（等合并重跑）
- 2026-11-24 发现：网络盘 df 显示集群总量 158 TB，看不到 150 GB 配额，fetch.py 查盘无效（已报，请转 traindata）
- 2026-11-24 大总管选 A：test 时跳过 test_gpu_mix、test_mix_loader，原因是 early_rir 未合入（evalcmp 未合进 main），下载不依赖它们；盘量用 du -sh /workspace 盯，超 120 GB 停
- 2026-11-25 00:21Z：第一批 A1–A5 + E2 共 21 包 18.15 GB 下完（aria2c，大包 4.4–5.2 MB/s），全过 unzip/tar 检查与 fetch.py 核对、已解压；/workspace 33 GB。E3 三个文件第 8 次才过 Wikimedia 429，sha256 与 sources.json 一致。两份 fetched.json 取回到主仓 `_tmp/20261124-cloudrun-fetched/`，读数已发 traindata（它已合并第一批，提交 7bffd12e）。CPU 机 vggy9ggvbi3h1p 已删（估 $0.29），账上 0 台机器；累计约 $1.15 / 上限 $60（账单数据晚约 1 小时，待补实扣）。
- 2026-11-25 上午（tmux 退出后 --resume）：大总管 noise-canceling-df 已删机 8mnlr0uvh5pu4a（split 失败 rc=1：OpenSLR28 的 RWCP_type4_rir_000l.wav 不在任何 rir_list；素材上传停在约 650 MB）。账本核对无误，累计 1.26 美元，平台 0 台。codex 第 2–4 轮的 P1 已修到能落代码的程度，78 条测试过；反复提的三件残余风险交大总管裁决（记录 cloud/reviews/codex-review.md）。等 traindata 修 split 后再开机：重跑 split→shard→register→pack，素材续传 4 路、分批前台跑、先核云上已有块。
- 2026-11-25 03:20Z 前后：第二台 CPU 机 e49nwj60n7hfpg 上做完 traindata 六步（build 49349c158a4defdc）、exp013 云上 build aa9f（与本机 6323 只差浮点舍入，改用 6323 音频）、评测素材 3485 个与 6323 音频 5020 个上传并逐个核对、inputs-verify「原料全对」。已删机，账上 0 台，累计 1.52 美元。
- codex 评审跑到第 14 轮：第 13、14 轮连续无 P0/P1，满足停止线；P2 留 #163–#167、#169、#171。95 条测试全过；与 main（971c7eec）合并无冲突。等大总管放行开 PR（Closes #158）。
- 2026-11-25 下午到晚上（分支 exp/025-cloud-baseline）：
  - CPU 机换镜像 `rust:1.99.1-trixie`（`config.CPU_IMAGE`，钉 digest）。原因：旧镜像 Ubuntu 22.04 的 glibc 2.35 链接不了 vdn-eval 的 ort。新镜像冒烟四样都过：python3、机器内自删、put-secret 假值、编译。
  - H100 冒烟：甲段在 CPU 上全过（122 过、1 跳过）；乙段第 1 步 overfit 一项不过，按收尾取回后删机，交 trainer。
  - 基线 480 段：main 27d2808a 跑完（08:11Z）。识别因一次超时被账本标「待核」后停发，只覆盖 189 段，不一致率 23.6%。结果在 `baseline/runs/20261125-053026-e5sjgks999cqbf/`（2a22515e）与主仓 `_tmp/20261125-cloudrun-baseline/run480/`。
  - 本机睡眠约 10 小时，期间机器到 6 小时时限自删；看门狗晚记账，估价偏高约 0.27 美元。
  - 账上 0 台，累计估 3.22 美元。待命：等 evalcmp 核结果、清 review、补跑识别，然后一起收尾；分支先不开 PR。
- 待办（大总管 2026-11-25 记）：每份仓库各带约 8 GB 的 .venv 是浪费；正式训练前改成一份共享 venv（整包或按 requirements 的 sha256 命名），所有副本共用。与「venv 整包解到容器盘」（装环境分步计时之后）一起做。
- 2026-11-26：派 pi（cloudrun-venvcache，花 0.0162 美元）写了 `cloud/remote/venv_cache.sh`（共享 venv）与 `cloud/remote/fetch_archives.sh`（边下边记 md5），各带测试；验收时打回 1 条 P1（GNU mktemp 不认 `-t` 无 X 模板）和 3 条 P2，修后 132 条测试全过，已合入本分支。还没上云实跑。
  - pi 写的代码 = 提交 ec06f94e..0f91ec80（3 个：ec06f94e、881e5dda、0f91ec80），署名 `Agent: pi`、`Model: deepseek-v4.1-flash-1017-fast`；在 pi 自己的分支 w/cloudrun-venvcache 上提交，快进合入，没被 hook 改署名。以后派 pi 照此记提交号范围；我验收后的提交加一行 `Code: pi (<模型名>)`（大总管 c4 2026-11-26 定）。
- 2026-11-26：派 pi（cloudrun-diskcheck，花 0.0083 美元）给 preflight 加网络盘余量检查：少于 30 GB 不开训练（`config.MIN_FREE_GB`，配额读平台接口）。pi 提交 56086350..1fac34b9（2 个，带 Agent: pi），快进合入；140 条测试全过。
- 2026-11-26：派 pi（cloudrun-fetchnomd5，花 0.0057 美元）给 fetch_archives.sh 加「md5 未知」写法：md5 栏写 `-` 时必须给字节数，只按字节数判，md5 与 sha256 都记进 checksums.txt（mode=bytes-only）。供 BUT ReverbDB（上游没发校验值）用。pi 提交 c8b8a839..46b504ff（2 个），快进合入；145 条测试全过。

## 2026-11-26 01:20–03:30Z（cloudrun）
- 第二批登记：build 6173bf9ca6a6e143，traindata 核过、已进仓。机器 g1wyqz5tsvqvia 已删，约 0.16 美元。
- BUT ReverbDB：字节数 9308593693 · md5 3ad91777… · sha256 d0d14cdd…（网络盘 archives/checksums.txt）。解出 wav 12307 个、txt 1748 个，占 23 GB。压缩包按 c4 的决定删了。
- 识别补跑：480 段已合并，calls +1953，spent +0.1875 美元，吃字有值的段数 82 → 452。取回到主仓 _tmp/20261126-asr-rerun/rerun-20261126-013313/。机器 glx9kkda13gckv 已删，约 0.17 美元。
- 第三批（代码 7badd9a1）：筛查结果 C236 keep 32/620，不到一半，按规则停了，等 traindata 定。机器 jm4m7pg5zmif3n 还开着。first-batch 没删（D112 的条件是第三批成功）。
- 小错：第一次开识别补跑时，提交号是手敲的，敲错了；脚本在 checkout 那一步就停了，没花识别费。以后提交号一律用 git rev-parse 取。
- 03:40–04:27Z 第三批续跑（代码 b81a9645，机器 byvkixm3sfzju6，约 0.09 美元，已删）：只登记 R112，得 build 10aa3a9fb8cef48f（valid，12 个场景）；包 sha256 e8c5c3d0…。C236 诊断 10 条：电平起伏中位数 7.46 dB，频谱峰多在 250 Hz。3 段 C236 按 D113 取回主仓 _tmp/20261126-c236-listen/，不进仓。盘 168 / 200 GB；first-batch 等 traindata 通知后按 D112 删。
- 待办（D112 条件已满足，traindata 核过 build 10aa3a9f）：下一台批下来的机器开机后先做：取回 /workspace/shards/first-batch 的 build 小文件（不含音频）到主仓 _tmp → 删 first-batch → 报盘余量。batch2 不动。删前已问 trainer 有没有在用。下一台训练机开机前必须先删（盘 168/200，剩 32 GB，门槛 30）。c4 会话已不在，开机要新大总管批。
  - trainer 回（2026-11-26）：没有任何运行、脚本、配置在读 first-batch。提到它的只有历史冒烟命令和 smoke_reports/。正式训练的分片路径还是占位，不会指向它。删除前的依赖检查已完成，只差开机。
- 2026-11-26 07:5xZ：派 pi（cloudrun-cap100，花 0.0076 美元）按 D115 把 `config.SPEND_CAP_USD` 从 60 改 100，带测试。pi 提交 4eb3dd62..8121f8c0（3 个，带 Agent: pi），快进合入；README 和 plan-eval-baseline 里的 60 美元由我改；147 条测试全过。
- 2026-11-26 07:38–07:45Z：按 D112 删 first-batch（机器 3h08g5wq4xr6fp，约 0.014 美元）。删前取回 index.json（sha256 dbfa14f1…）和 66 个 tar 的清单与 sha256，放在主仓 _tmp/20261126-first-batch-small/。盘 168 → 153 / 200 GB。
- 2026-11-26 07:48–08:48Z 第四批 A 段（代码 249875ca，机器 sm5qjwcfp8slir，约 0.12 美元，已删）：C236 单音全量 first_only 331 · later_too 289，按 D114 不用；办公室 4154 条 none 3840 · later_too 313；按 traindata 定，每次布置取 01、11、21 三个麦克风位置，共 402 条，筛查 keep 390。结果放在主仓 _tmp/20261126-cloudrun-batch4/（sha256 7df39381…）。另回 traindata：CochlScene 的 Restaurant、Cafe 没解压，压缩包已删，重下约 54 GB。
- 待办：B 段（traindata 给提交号后）重打全部分片约 +23 GB，跑完约 177 GB，会低于 30 GB 开训门槛。开训前先由大总管或人定删 batch2 还是 batch3（D112：batch2 留到第一个正式运行过开发份检查）。
- 开训清单第 8 项（花费重估，按 PR #194 实测，对上限 100 美元）：新 build 出来后做。
- 2026-11-26 14:44–16:08Z 第四批 B 段（机器 p8wburmyl2jz6o，约 0.18 美元，批 ≤0.24，已删）。第一次用代码 a632a660，停在切分：仓里已有的 but-reverbdb-silence 切分表不许覆盖。输出留在网络盘 /workspace/out/batch4b-try1-a632a660。traindata 改用新来源名 but-reverbdb-silence-b4（254ebece），我在同一台机器上重跑：
  - 测试全过；切分 584 / 78 / 66，dev 4 个单元 104 个文件，4 间房各 1 个；分片 2458 秒。
  - 6 个 build 都是 valid，交集都是 0：基线 bc0e9e7b566421f0，另外 5 个是 10ebb73b、36ea9561、574ef5c7、748aee58、966a229d。
  - 打包 sha256 a2b67102…，取回到主仓 _tmp/20261126-cloudrun-batch4b/，校验一致。
- 同一台机器上按 D116 删 batch3（build 10aa3a9f 的分片）。trainer-0e 确认没有依赖。删前取回 93 个文件的名字、字节数和 sha256，共 21.6 GB；index.json 的 sha256 是 b56737da…，放在主仓 _tmp/20261126-batch3-small/（tar sha256 c5e07426…）。盘 176 → 156 / 200 GB，剩 44 GB，过开训门槛。batch2 不动。
- 大总管固定名改为 noise-canceling-meta（人定），以后只报这个名字。trainer 现名 noise-canceling-trainer-0e。
- 累计花费 8.97 / 100 美元，云上 0 台机器。
- 待办：等 trainer 的正式训练命令。开 H100 前做开训清单第 8 项（花费重估），报 meta 批，并要人点头。
- 2026-11-26 18:00–19:21Z 跑 plan-h100-b4（main 98f1cf32，meta 批：甲段 ≤0.1 美元，乙段 1.2 小时、≤7 美元）：
  - 甲段：CPU 机 60b5a4pkpzjzlq，0.05 美元。前提 2、3、4 都核过。测试 500 条通过、跳过 1 条；known-dev-cpu.json 的 sha256 是 d480a196…。
  - Room112 单音补测：248 条，none 210 · later_too 38；jsonl 的 sha256 是 8efa7d68…，已发 traindata。traindata 据此出了第五批。
  - 乙段：H100 o1hkngnpduv1jb，4.32 美元。冒烟 pass（197 秒）；speed-B 299 秒，speed-A 216 秒，两个 dev_gate 都是 0；续训核对 max_abs_diff 是 0。
  - 消融 7 组都是退出 0：每批耗时 535 / 536 / 352 秒。每批 3 个运行在同一个 job 里用后台进程同时跑。
  - 结果在主仓 _tmp/20261126-trainer-b4/（O 包 e6925628…，ablation 包 8aa73703…）。trainer-da 跑了 judge，退出 0。
- 19:22Z 开第五批的 CPU 机 r5i6id8wqrx7tv（批 ≤0.3 美元，2 小时后自删），代码 0466bd88。第 6 步只取回 batch4 的校验值，删不删要等人定。
- 累计花费 13.40 / 100 美元。trainer 现名 noise-canceling-trainer-da。
- 2026-11-26 19:22–20:23Z 第五批（代码 0466bd88，机器 r5i6id8wqrx7tv，0.13 美元，批 ≤0.3，已删）：
  - 测试 483 条通过、跳过 6 条；单音合并核对逐字节一致，keep 188。切分 563 / 78 / 87，4 间房在 dev 里各 1 个。
  - 6 个 build 都是 valid、交集都是 0：基线 df759bb8d606a209，另外 5 个是 4bff0eef、84b1b7ed、94b873a1、cd7a4d17、d09858dd。
  - 打包 sha256 92e922be…。结果在主仓 _tmp/20261126-cloudrun-batch5/。
- 按 D118 删 batch4（trainer-c6 确认没有依赖）：删前取回 index.json（311134757bf6…，和 bc0e9e7b 的记录一致）、94 个分片的 sha256、du 22289901880 字节，打包 sha256 aa74b90d…。盘 181 → 160 / 200 GB，剩 40 GB。网络盘上只剩 batch2、batch5。
- ⚠️ trainer 的 formal.yaml、ablation.yaml 还指着 batch4，formal-runs.yaml 还钉着 bc0e9e7b。trainer 改指第五批之前，不用这几份配置开机。
- 累计花费 13.55 / 100 美元，云上 0 台机器。现名：traindata-75、trainer-c6、大总管 meta。
- 待办：等 plan-h100-formal.md。开 H100 前做开训清单第 8 项的花费重估，报 meta 批，并要人点头。

## 2026-11-26 正式训练甲段（CPU 机）超时 · 教训

- 第一台 CPU 机 w3hx88oodste3e（时限 0.5 小时，0.061 美元）：formal check 过、输出目录不存在、盘剩 40 GB；单元测试实测 1015 s（执行单估 301 s，慢 3 倍多），CPU 参照没算完，机器到时限自删。
- 补跑 2drq1wolg6m8or（大总管批 0.4 小时，最坏约 0.06 美元）：只跑 formal check · CPU 参照 · 量盘，用网络盘上已装好的代码和环境（脚本 scratchpad/formal/fa2.sh）。
- **教训（大总管定）：机器时限按实测最慢的那次加 50% 余量来定，不按执行单的估时。** CPU 机的速度每台不一样；单元测试那一步已见过 301 s 和 1015 s 两个数。
