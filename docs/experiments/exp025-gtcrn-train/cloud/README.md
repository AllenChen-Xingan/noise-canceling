# exp025 · cloud：RunPod 云训练机脚本

用途：开关云训练机、在机器里跑命令、传文件、记费用。所有跑模型的活都在这里开机跑（D97）。
数字的正本是 `config.py`；本文只说怎么用。`tests/test_cloud.py` 会核对本文的关键数字和 `config.py` 一致。

## 机器与花费（正本 `config.py`，依据 D96 及其更正）

- 卡：`NVIDIA GeForce RTX 3090` 一张，Secure Cloud，要求 vCPU ≥ 16（分到的少于 16 个就自动删）。
- 镜像：`pytorch/pytorch:2.5.1-cuda12.4-cudnn9-runtime`，容器只运行 `sleep <最长秒数>`，活都用 exec 发进去。
- 网络存储盘：`vdn-train`，150 GB，挂在 `/workspace`；每月约 10.5 美元（0.07 美元 / GB / 月）。
- 花费上限：20 美元。开机前检查「已删机器的花费 + 在开机器到现在的花费 + 网络盘到今天的摊销 + 本次最坏（每小时价 × 最长小时数）」，超了就不开。
- 删机一律用删除（DELETE），不用停止（stop）：停止后机器自带盘照收钱（034 compute.md 1.2）。

## 用法

密钥：先读环境变量 `RUNPOD_API_KEY`，没有就读主仓库根的 `.env.local`。密钥不打印、不进日志。

```sh
cd docs/experiments/exp025-gtcrn-train/cloud
python3 rp.py offers                                   # 3090 价格、有货且能建盘的机房
python3 rp.py volume-create --dc <机房> --yes          # 建 150 GB 网络盘（花钱，先经大总管放行）
python3 rp.py up --max-hours 2 --yes                   # 开机并挂盘；打印机器编号
python3 rp.py sh <机器> 'nvidia-smi; nproc'            # 短命令（≤ 25 秒）
python3 rp.py job-start <机器> train 'cd /workspace && python3 train.py'
python3 rp.py job-wait <机器> train --timeout 7200     # 等后台命令结束，打印日志尾
python3 rp.py put <机器> 本地文件 /workspace/远端路径   # 传上去，核 sha256
python3 rp.py get <机器> /workspace/ckpt.pt ckpt.pt    # 取回来，核 sha256
python3 rp.py down <机器>                              # 删机并记账
python3 rp.py reap                                     # 兜底：删掉所有 vdn- 开头的机器
python3 rp.py spend                                    # 按账本汇总已花
python3 rp.py probe --dc <机房> --no-volume --max-hours 0.25 --yes   # 试开一台测速，测完自动删
```

- 长命令放后台：exec 接口单次最多等 25 秒；`job-start` 把输出写进 `/root/jobs/<名字>.log`，退出码写进 `.rc`。
- 传文件走 exec：上传每次约 1 MB，取回每块 512 KB；适合代码、检查点、结果表。大数据集直接在机器里从源站下载到网络盘。
- 防忘删的三道保险：
  1. 脚本在 `probe` 和开机失败时用 finally 删机。
  2. 容器到最长时限就退出，算力停止计费。
  3. `reap` 删掉所有 `vdn-` 开头的机器。

## 费用账本 `costs.tsv`

每次建盘、开机、删机记一行。列名的正本是 `config.COSTS_TSV_COLUMNS`。
- `est_usd`：每小时价 × 开机小时数（估算）。
- `billed_usd`：RunPod 账单金额。账单有延迟，有了就补填；汇总时优先用账单。

## 接口

RunPod API v2（`https://api.runpod.io`）。脚本用到的字段抽成了 `tests/api_fields.json`，里面记着规范文件的 sha256 前 16 位（`config.SPEC_SHA256_16`）。测试核对请求体字段都在规范里。规范更新后，重新抽一次再跑测试。

## 测试

```sh
python3 -m unittest discover -s docs/experiments/exp025-gtcrn-train/cloud/tests
```

全部用假 API（`tests/fake_runpod.py`），不连网、不开机、不花钱。

## 试开结果（测速）

尚未试开。结果会写进 `probes/<时间>.json`，并在这里汇总。
