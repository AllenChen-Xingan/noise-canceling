# 方案：云上跑 evalcmp 基线成绩单（480 段整套）

状态：大总管已批（2026-11-25），put-secret 已写好并有测试（`tests/test_secret.py`）；等 evalcmp 分支合进 main 后再开机。写于 2026-11-25，cloudrun。

## 一、密钥怎么上机（OPENROUTER_API_KEY）

做法：新加子命令 `rp.py put-secret <机器> <键名>`。

1. 本机在进程里从主仓库 `.env.local` 读出这一行。不经命令行参数，不落临时文件。
2. 调 exec 接口，把值放进请求体的 `env` 字段：exec 的请求字段有 `command`、`cwd`、`env`、`timeoutSeconds`，见 `tests/api_fields.json`。
3. 机器里跑的命令只引用环境变量，不含值（正本是 `secret_ops.py` 的 `WRITE_SH`）：目标是目录、或父目录实际不在 `/root/` 下就拒；`mktemp` 独占建临时文件，先在临时文件上核 sha256 和权限 600，对了才原子替换；失败只删本次的临时文件，旧文件不动。
4. 只打印「已写入，N 位」，不打印值，也不打印哈希。报错里不回传任何远端正文和原始异常，只报方括号里的结果码（OK 以外：DIR / OUTSIDE / BAD / LEFTOVER / UNKNOWN）和中文说明。
5. 文件在容器盘 `/root/`，不在网络盘。删机时随容器一起消失。

不会出现密钥的地方：本机命令行参数、本机日志、`costs.tsv`、交接文件、机器里的进程参数（`ps` 看得到的部分）、报错与异常链。

**测试**（不带测试不算完成）：
- 用假接口跑 `put-secret`，记录所有 exec 请求的 `command` 字段、CLI 的 stdout 和 stderr、账本全文。里面出现密钥原文、或它的 base64 编码，就判红。
- 本机 `.env.local` 没有这个键时，拒绝并说明，不发请求。
- 机器里写文件失败、超时、本机 `.env.local` 读出错时报错，报错和异常链里都不带值（不回传远端正文）。

**残余风险（推断，未核实）**：
- 值会经 HTTPS 交给 RunPod 的 exec 接口。RunPod 会不会记录 exec 请求里的 `env`，查不到文档。
- 在机器里，那条命令运行的一瞬间，`/proc/<pid>/environ` 里有这个值。机器只有我们用，容器里是 root。

大总管已接受这两点；这批成绩单跑完后轮换密钥。没采用的替代办法：开机时放进机器的环境变量。值会留在平台的机器配置里，GET 机器信息能看到，更差。

**怎么给 vdn-eval 用**：vdn-eval 读仓库根 `.env.local` 或同名环境变量；仓库根在网络盘上，所以不放那里。后台任务开头用 `set -a; . /root/.env.local; set +a` 读进环境变量，再跑成绩单。

**真机冒烟（开机后第一步，先用假值）**：用一个假键名和随机假值，在机器上跑 `put-secret` 的四种情况：正常写入、目标是目录、父目录链接到 `/workspace`、已有旧文件且核对失败。再加一种：让临时文件删不掉（目录设成只读），看是否报 LEFTOVER。核对结果码和旧文件是否保留，都按预期才送真密钥（评审 secret-ops 第 1 轮 P2 #6：本机 macOS 跑不了 Linux 那段 sh）。
送真密钥后，在机器里只查权限和位数（`stat -c %a`、`wc -c`），不打印内容。

## 二、机器与花费

- CPU 机：cpu3c 4 核，US-MO-1，挂网络盘 bzpbedqdsi，每小时 0.12 美元。
- 这次只跑 1 个模型：原版 GTCRN 基线（大总管定）。
- 时长：evalcmp 按本机 Apple M 系列 5 段冒烟推算 1.3–1.5 小时；云上 x86 单核速度没量化，可能更慢。所以先用 `baseline/subset.py` 裁 5 段冒烟，量出每段秒数再决定跑整套。
- 上限：这次照旧 3 小时、1 美元，自删时限 3 小时，本机看门狗照开。以后跑成绩单的 CPU 机单次上限改为 6 小时、1.5 美元（大总管定，技术题），一台机器上串着跑多个模型；都在人定的 60 美元总上限内。
- 识别费：走 OpenRouter，上限 15 美元（D103），由 evalcmp 的账本按每次调用的实际金额（`usage.cost`）累计，不进 RunPod 的 `costs.tsv`。evalcmp 估：每段 3 条 × 2 次 = 6 次，480 段共 2880 次，每次约 0.00009 美元，合计约 0.26 美元。

## 三、环境

- 代码：main 上合并 evalcmp 之后的提交，放进网络盘新目录 `/workspace/repo-main-<提交>`。跑法以 main 上 `docs/experiments/exp025-gtcrn-train/baseline/README.md` 第一到三节为准。
- 编译：Rust 按仓库根 `rust-toolchain.toml`（1.99.1，带 rustfmt、clippy），`cargo build --release -p vdn-eval`。编译时 ort 要联网下载 Linux 预编译库。用时未量化，开机后实测。
- 被测模型：GTCRN 权重 `gtcrn_simple.onnx`，来自 github.com/Xiaobin-Rong/gtcrn 提交 502ebfab64da7c4a9af78dcb9c6ceef1ebb01c73 的 `stream/onnx_models/`，535190 字节，sha256 b4718df6…ebcb87。在机器里从上游下载，下完核 sha256；`VDN_GTCRN_ONNX` 指向它。
- 两个评分模型，从本机上传，上传后逐个核 sha256：
  - SCOREQ：本机 `~/.cache/scoreq/onnx-models/adapt_nr_telephone.onnx`，378049307 字节，sha256 d493ba8a…307300b4。放到机器家目录同一相对路径。
  - Distill-MOS：本机 `~/vdn-data/models/distillmos/distill_mos_v7-seg122880.onnx`，17613491 字节，sha256 b8e2dc18…0f881d2928。这份是本项目自己导出的，只能传本机这份。
  - vdn-eval 载入前自己核哈希，对不上就判「不知道」；跑完要看成绩单里这两项有没有值。
- 音频：6323 已在 `/workspace/exp013-builds/20261124165427-6323/`，新目录的仓库根 `_tmp/exp013/` 下要重建链接。
- Python：成绩单本身不用；差距表 `gap.py` 只用标准库。

## 四、步骤（开机后）

1. 开机，报你机器号和开机时间。
2. 上传代码，`put-secret` 送密钥，装工具链，release 编译。
3. 上传两个评分模型，核 sha256。
4. 先 5 段冒烟，量每段秒数；按它估整套用时，在上限内才跑 480 段：
   `set -a; . /root/.env.local; set +a; export VDN_GTCRN_ONNX=<路径>; target/release/vdn-eval scorecard --system gtcrn --no-safety --qa --out docs/experiments/exp025-gtcrn-train/baseline/runs/<时间>-<机器名>`。
   后台任务按运行编号等待。
5. 取回成绩单和日志到主仓 `_tmp/<日期>-cloudrun-scorecard/`，不覆盖。报 evalcmp 和你。
6. 删机，报账上 0 台和花费。

## 五、已向 evalcmp 要到的

第三、四节里的命令、文件来源与哈希、识别调用估算，都来自 evalcmp（2026-11-25 回复）。
