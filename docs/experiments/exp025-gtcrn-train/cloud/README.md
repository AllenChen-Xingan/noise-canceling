# exp025 · cloud：RunPod 云训练机脚本

用途：开关云训练机、在机器里跑命令、传文件、记费用。所有跑模型的活都在这里开机跑（D97）。
数字的正本是 `config.py`；本文只说怎么用。`tests/test_cloud.py` 会核对本文的关键数字和 `config.py` 一致。

## 机器与花费（正本 `config.py`，依据 D96 · D101 · D102）

- 卡：`NVIDIA H100 80GB HBM3` 一张，机房 `US-MO-1`（美国密苏里），Secure Cloud，要求 vCPU ≥ 8（分到的少于 8 个就自动删；实际核数开机实测）。
- 价格：目录价约 4.39 美元 / 小时（2026-11-24）。开机前目录价高出 20% 以上就不开。
- 为什么是 H100：2026-11-24 查 RunPod 目录，3090 在 Secure Cloud 所有机房无货，人改选 L4 · US-MO-2（D101）；L4 开机报「无可用机器」，人改 H100（D102）。60 美元上限约合 H100 13 小时。
- 镜像：`pytorch/pytorch:2.5.1-cuda12.4-cudnn9-runtime`，容器命令只做一件事：睡到开机时算好的截止时刻，再用平台注入的变量删掉自己（失败每 60 秒重试）；活都用 exec 发进去。
- ⚠️ 显卡机镜像（pytorch）里**没有 git**，trainer 的运行记录要读提交号，没有 git 就拒跑（2026-11-25 H100 乙段第 2 步实测踩到，白花约 0.6 美元）。显卡机开机后第一步先 `apt-get install -y git`，再 `git -C <仓库> rev-parse HEAD` 核一遍，失败就停，不进后面的步骤。CPU 机镜像自带 git。
- 删机前先取回机器里的日志（容器盘 `/root/jobs/` 删机就没）；下载脚本删压缩包之前，先把每个包的 md5 / sha256 读数写进网络盘上的小文件（2026-11-26：CochlScene 后 3 卷的 md5 读数随删机丢了，只剩「全对才解压」的间接证据）。
- CPU 机镜像：`rust:1.99.1-trixie`（钉 digest，正本 `config.py` 的 `CPU_IMAGE`）。显卡机那个镜像是 Ubuntu 22.04（glibc 2.35），vdn-eval 用的 ort 预编译库在上面链接不过（2026-11-25 实测）；Debian 13 的 glibc 是 2.41，Rust 版本与 `rust-toolchain.toml` 一致。
- 共享 venv：`sh remote/venv_cache.sh <requirements.txt>…`。按「Python 版本 + uv 版本 + requirements 全文」的 sha256 前 16 位定名，同一套依赖在网络盘 `/workspace/venvs/<名>.tar` 只放一份；每台机器解到容器盘 `/root/venvs/<名>`，最后一行打印 `PY=<路径>`。第一次会装包并发布（`ln` 发布，不覆盖别的机器已发布的那份），以后只解包。每步计时。取代「每份仓库各带一个约 8 GB 的 .venv」（大总管 2026-11-25 定）。
- 下数据：`sh remote/fetch_archives.sh <清单> <目录>`，清单每行「名字 md5 地址」。每下完一个就把 md5 追加进 `<目录>/checksums.txt`（网络盘上，删机也在），不符就停；脚本从不删东西，删压缩包另做、删前先取回 checksums.txt。
- 装 trainer 环境用 `remote/setup_env_timed.sh <仓库根> <setup_uv.sh>`：做法与现行相同，每步单独计时，uv 不带 -q，留下「Prepared … in」（下载 + 解包）和「Installed … in」（链接进 venv）两段时间。起因：2026-11-25 同样的装法一次 133 秒、一次 889 秒（推断：网络盘小文件慢）。大总管定：先用计时定原因；如果证实慢在网络盘小文件，再把 venv 做成整包（包名带 requirements 的 sha256，解到容器盘），写进本目录代码并带测试，赶在正式训练前做好。所有机器统一用 setup_uv.sh 钉的 uv 版本。
- uv 装的 Python 放网络盘：在机器里跑 traindata 的 `cloud_steps.sh setup` 前先 `export UV_PYTHON_INSTALL_DIR=/workspace/uv-python`。不设的话解释器落在容器盘 `/root/.local/share/uv`，网络盘上的 `.venv` 换一台机器就用不了（大总管定，2026-11-25）。
- 网络存储盘：`vdn-train`，150 GB，挂在 `/workspace`；每月约 10.5 美元（0.07 美元 / GB / 月）。
  机器全删了网络盘也照收钱。花费上限只在花钱前检查（每次都按往后一个月盘费预留），不会自动删盘；盘里有数据，闲置多久删盘由人定。按 10.5 美元 / 月，闲置约 4.5 个月会把 60 美元上限剩下的部分用完（2026-11-25 已花约 1.26 美元）。
- 花费上限：60 美元。开机前检查「已删机器的花费 + 开着的机器跑满各自时限 + 网络盘到今天的摊销 + 没删的网络盘往后一个月的盘费 + 本次最坏（每小时价 × (最长小时数 + 看门狗宽限)）」，超了就不开；开出来后按实际价再查一遍，不过就删。建网络盘前先按一个月盘费查。
- 删机一律用删除（DELETE），不用停止（stop）：停止后机器自带盘照收钱（034 compute.md 1.2）。

## 用法

密钥：先读环境变量 `RUNPOD_API_KEY`，没有就读主仓库根的 `.env.local`。密钥不打印、不进日志。

```sh
cd docs/experiments/exp025-gtcrn-train/cloud
python3 rp.py offers                                   # 本卡价格、有货且能建盘的机房
python3 rp.py volume-create --yes                      # 在 US-MO-1 建 150 GB 网络盘（花钱，先经大总管放行）
python3 rp.py up --max-hours 2 --yes                   # 开机并挂盘；打印机器编号
python3 rp.py up --cpu --max-hours 1 --yes            # 开纯 CPU 机器（cpu3c 4 核）挂同一块盘，下载数据用
python3 rp.py sh <机器> 'nvidia-smi; nproc'            # 短命令（≤ 25 秒）
python3 rp.py job-start <机器> train 'cd /workspace && python3 train.py'   # 打印运行编号，如 train-20261125-0130-a1b2c3
python3 rp.py job-wait <机器> <运行编号> --timeout 7200  # 按编号等后台命令结束，打印日志尾
python3 rp.py put <机器> 本地文件 /workspace/远端路径   # 传上去，核 sha256
python3 rp.py get <机器> /workspace/ckpt.pt ckpt.pt    # 取回来，核 sha256
python3 rp.py put-secret <机器> OPENROUTER_API_KEY   # 把 .env.local 里这个密钥送进机器 /root/.env.local（容器盘，600），只报位数
python3 rp.py down <机器> --yes                        # 删机并记账（不带 --yes 只列出要删的）
python3 rp.py reap --yes                               # 兜底：删掉所有 vdn- 开头的机器（不带 --yes 只列出）
python3 rp.py spend                                    # 按账本汇总已花
python3 rp.py probe --dc <机房> --no-volume --max-hours 0.25 --yes   # 试开一台测速，测完自动删
```

- 长命令放后台：exec 接口单次最多等 25 秒；`job-start` 每次生成唯一运行编号，输出写进 `/root/jobs/<编号>.log`，退出码写进 `.rc`。同名任务跑两次各有各的文件。命令按 `sh` 解析，bash 专有写法（如 `>( )`）要包进 `bash -c`，否则启动前就报语法错。
- `put-secret`：值走 exec 请求的 env 字段，不进任何命令参数、输出、账本；只许写到容器盘 `/root/` 下，删机即没。后台任务里用 `set -a; . /root/.env.local; set +a` 读进环境变量。残余风险：RunPod 会不会记录 exec 请求里的 env 不知道（大总管已接受，用完轮换）。详见 `secret_ops.py`、`plan-eval-baseline.md` 第一节。
- `put` / `get` 默认不覆盖已有文件；加 `--overwrite` 才覆盖，`put` 会把远端旧文件留作 `.bak-时间-随机`。
- 传文件走 exec：上传每次约 1 MB，取回每块 512 KB；适合代码、检查点、结果表。大数据集直接在机器里从源站下载到网络盘。
## 防忘删机（实现在 `watchdog.py`）

**只有删机才停止计费。** 2026-11-24 实测：容器只是退出的话，平台约 30 秒内把它重启，状态仍是「运行中」，照常按小时扣钱。所以每台机器都有下面几道兜底。`--max-hours` 既用于开机前的花费检查，也是自删时限。

1. **机器内自删**：开机命令先睡到时限，再用平台注入机器的 `RUNPOD_POD_ID` 和 `RUNPOD_API_KEY` 删自己（只删自己这一台）。失败每 60 秒重试。
   - 实测（2026-11-24，1 次）：设时限 108 秒，机器约 108 秒后自己没了。
   - 这把密钥是平台放进机器的，不是我们的密钥；它的权限范围没查到文档，未实测能否删别的机器。
2. **本机看门狗**：`up` 时登记「删机时刻」，等于开机时刻 + 最长小时数 + 5 分钟，并在后台起 `rp.py watchdog`（带 `caffeinate -i` 防空闲睡眠）。到点它调删除；登记表空了就自己退出。
   - 登记表在 `.watch/`，不进仓。
   - 本机合盖睡眠或断网时它不可靠，所以第 1 层是主力。
3. **脚本 finally**：`probe` 和开机失败时用 finally 删机。
4. **`reap`**：删掉所有 `vdn-` 开头的机器，手动兜底。

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

## 试开结果（2026-11-24，H100 · US-MO-1，1 次）

原始记录：`probes/20261124-134810.json`；费用见 `costs.tsv`。机器开了 9 分 5 秒，估算 0.66 美元（4.39 美元 / 小时 × 0.151 小时；账单金额约 1 小时后才出，待补 `billed_usd`）。

下面是实测数字，样本量都是 1 次。

- 开机：从发出请求到「运行中」0.4 秒；挂网络盘 `/workspace` 正常（RunPod 的分布式文件系统）。
- 在机器里执行命令（exec）：可用，单条命令约 85 毫秒。旧密钥（限权）会被拒（403），换全权限密钥后可用。
- 显卡：H100 80GB HBM3，显存 81,559 MiB，驱动 595.112.07，CUDA 13.2。
- vCPU：接口报分给本机 26 个；容器的 CPU 配额（cgroup）是 22.1 个；`nproc` 显示 208，这是宿主机总数，不是我们能用的。数据装载进程数按 22 左右定。
- 内存：接口报 251 GB（`free` 显示的 2 TB 是宿主机总数）。
- 下载速度（每个地址读 20 秒，单线程）：

| 来源 | 速度 |
|---|---|
| OpenSLR 主站 | 0.43 MB/s |
| OpenSLR 欧洲镜像 elda | 1.39 MB/s |
| OpenSLR 镜像 trmal | 0.48 MB/s |
| Zenodo（DEMAND 一个包，110 MB 4.3 秒下完） | 25.6 MB/s |

- 推论（未实测）：OpenSLR 28 号包 1.3 GB，按 1.39 MB/s 约 16 分钟；按 0.43 MB/s 约 50 分钟。大包从 OpenSLR 下时，开多个连接并行，或在便宜的 CPU 机器上下到网络盘，别占着 H100 等下载。
- 到时限后：容器被重启，照常计费（当时还没有机器内自删）。见「防忘删机」一节。

## 下数据（2026-11-24，CPU 机 cpu3c 4 核 · US-MO-1，1 次）

做法：先用 `predownload.py` 以 aria2c 多连接（8 连接，OpenSLR 同时用三个镜像）把压缩包下到 `/workspace/archives/`，
每个包先过 `unzip -tq` / `tar -tzf`；再跑 traindata 的 `cloud_steps.sh fetch`，它见包已存在就跳过下载，照常核哈希、解压、记 `fetched.json`。

```sh
python3 rp.py up --cpu --max-hours 6 --yes
python3 rp.py put <机器> predownload.py /workspace/tools/predownload.py
python3 rp.py job-start <机器> dl 'python3 /workspace/tools/predownload.py $D/sources.json /workspace A1,A2,A3,A4,A5,E2 --max-gb 120 && cd $D && bash cloud_steps.sh fetch'
```

实测（样本 1 次）：
- 21 个包、18.15 GB，下载加核对加解压共约 48 分钟，退出码 0；网络盘用到 33 GB。
- 速度：OpenSLR 大包 4.4–5.2 MB/s（单连接时 0.43–1.39 MB/s）；Zenodo 的 DEMAND 包 7–56 MB/s。
- Wikimedia 的 3 个小文件连回 7 次 429（「请求太多」，服务器给 Retry-After 120 秒），第 8 次才成。traindata 已改成限流另算、最多 30 次。
- 这台机器开了 2.39 小时，估算 0.29 美元。
