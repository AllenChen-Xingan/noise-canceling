# 方案：云上跑 evalcmp 基线成绩单（480 段整套）

状态：方案，未开机。写于 2026-11-25，cloudrun。等大总管批，并等 evalcmp 分支合进 main 后再开机。

## 一、密钥怎么上机（OPENROUTER_API_KEY）

做法：新加子命令 `rp.py put-secret <机器> <键名>`。

1. 本机在进程里从主仓库 `.env.local` 读出这一行。不经命令行参数，不落临时文件。
2. 调 exec 接口，把值放进请求体的 `env` 字段：exec 的请求字段有 `command`、`cwd`、`env`、`timeoutSeconds`，见 `tests/api_fields.json`。
3. 机器里跑的命令只引用环境变量，不含值：
   `umask 077; printf '%s=%s\n' "$K" "$V" > /root/.env.local.part && mv /root/.env.local.part /root/.env.local`。
4. 核对：在机器里算文件的 sha256，和本机算的比。只打印「已写入，N 位」，不打印值，也不打印哈希。
5. 文件在容器盘 `/root/`，不在网络盘。删机时随容器一起消失。

不会出现密钥的地方：本机命令行参数、本机日志、`costs.tsv`、交接文件、机器里的进程参数（`ps` 看得到的部分）。

**测试**（不带测试不算完成）：
- 用假接口跑 `put-secret`，记录所有 exec 请求的 `command` 字段、CLI 的 stdout 和 stderr、账本全文。里面出现密钥原文、或它的 base64 编码，就判红。
- 本机 `.env.local` 没有这个键时，拒绝并说明，不发请求。
- 机器里写文件失败时报错，错误信息里也不带值（沿用 `rp_client` 的脱敏）。

**残余风险（推断，未核实）**：
- 值会经 HTTPS 交给 RunPod 的 exec 接口。RunPod 会不会记录 exec 请求里的 `env`，查不到文档。
- 在机器里，那条命令运行的一瞬间，`/proc/<pid>/environ` 里有这个值。机器只有我们用，容器里是 root。

如果不接受这两点，替代办法：开机时放进机器的环境变量。代价更大：值会留在平台的机器配置里，GET 机器信息能看到，所以不推荐。

## 二、机器与花费

- CPU 机：cpu3c 4 核，US-MO-1，挂网络盘 bzpbedqdsi，每小时 0.12 美元。
- 时长：evalcmp 估每个模型 1.3–1.5 小时。模型有几个，向 evalcmp 要。如果超过 2 个，3 小时不够，要分两次开，或请你放宽上限。
- 自删时限 3 小时，本机看门狗照开。机器费最多约 0.36 美元，在 1 美元以内。
- ⚠️ **识别走 OpenRouter API，按调用收费，不在 RunPod 的 1 美元里。** 480 段 × 几个模型 × 每段调用几次、每次多少钱，向 evalcmp 要估算。要你另定一个上限，或者说明这笔算在哪里。

## 三、环境

- 代码：main 上合并 evalcmp 之后的提交，放进网络盘新目录 `/workspace/repo-main-<提交>`。
- `vdn-eval` 用 release 编译。CPU 机上要先装 Rust 工具链，装哪个版本按仓库里的 `rust-toolchain` 文件；编译用时未量化，开机后实测。
- 两个评分模型文件：来源（链接或本机路径）和 sha256 向 evalcmp 要。上传后逐个核对。
- 音频：6323 已在 `/workspace/exp013-builds/20261124165427-6323/`，云上仓库根 `_tmp/exp013/` 下有链接。新目录里要重建这个链接。
- Python 依赖：按 evalcmp 给的锁文件装。

## 四、步骤（开机后）

1. 开机，报你机器号和开机时间。
2. 上传代码，`put-secret` 送密钥，装工具链，release 编译。
3. 上传两个评分模型，核 sha256。
4. 按 evalcmp 的命令跑成绩单。每个模型单独一个后台任务，按运行编号等待。
5. 取回成绩单和日志到主仓 `_tmp/<日期>-cloudrun-scorecard/`，不覆盖。报 evalcmp 和你。
6. 删机，报账上 0 台和花费。

## 五、要向 evalcmp 要的

- 要跑几个模型、每个的跑法（命令、参数）
- 两个评分模型文件的来源与 sha256
- OpenRouter 调用次数与费用估算
- 依赖锁文件位置、`rust-toolchain` 版本
