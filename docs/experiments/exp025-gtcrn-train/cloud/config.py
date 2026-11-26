"""云训练机的常量：唯一正本。其他文件（README、脚本、测试）只从这里引用，不手抄。

出处：D96（Secure Cloud · vCPU ≥ 16 · 150 GB 网络盘）· D101 定 60 美元、D115 改 100 美元（2026-11-26）· D102（L4 开不出 → H100 · US-MO-1）。
接口：RunPod API v2（https://api.runpod.io/v2/openapi.json，2026-11-16 取，sha256 前 16 位见 SPEC_SHA256_16）。
"""

API_BASE = "https://api.runpod.io"
SPEC_SHA256_16 = "41f48a46a68fd023"

# 机器（D102）
GPU_TYPE_ID = "NVIDIA H100 80GB HBM3"
DATA_CENTER = "US-MO-1"
# 目录价（2026-11-24 实测）；开机时目录价超过它 20% 就不开
USD_PER_HR_EXPECTED = 4.39
PRICE_TOLERANCE = 1.2
CLOUD = "SECURE"
# 纯 CPU 机器（下载数据到网络盘用，大总管 2026-11-24 方案）：同机房最便宜的一档
CPU_FLAVOR_ID = "cpu3c"
CPU_VCPU = 4  # 必须是 2 的幂（openapi BaseCpuConfig）
CPU_USD_PER_VCPU_HR_EXPECTED = 0.03  # 目录价（2026-11-24 实测）；4 核 ≈ 0.12 美元 / 小时
# D102：H100 平台最低 8 个；分到的少于它就删，多于它照实记
MIN_VCPU_PER_GPU = 8
# 镜像锁版本（034 compute.md 第三节第 4 条）；容器只睡觉，活全靠 exec 发进去
IMAGE = "pytorch/pytorch:2.5.1-cuda12.4-cudnn9-runtime"  # 显卡机用（训练要 torch + CUDA）
# CPU 机镜像（大总管定 2026-11-25）：上面那个是 Ubuntu 22.04（glibc 2.35），vdn-eval 用的 ort 2.0.0-rc.15 预编译库
# 链接不过（缺 __isoc23_strtol 等，实测）。改用 Debian 13 的官方 Rust 镜像：glibc 2.41，Rust 1.99.1 与 rust-toolchain.toml 一致。
# 写到 tag + digest（Docker Hub 多架构索引的 digest，2026-11-25 查）；机器内自删要 python3，开机后先冒烟。
CPU_IMAGE = "rust:1.99.1-trixie@sha256:8a4e90c7946b74626488b1446f712c0fe0ed1a596cbd3d871684bc9c5433ae07"
CONTAINER_DISK_GB = 30
POD_NAME_PREFIX = "vdn-"

# 网络存储盘（D96）
VOLUME_NAME = "vdn-train"
VOLUME_SIZE_GB = 150
VOLUME_USD_PER_GB_MONTH = 0.07  # 034 compute.md 1.4：150 GB → 每月 10.5 美元
VOLUME_MOUNT = "/workspace"

# 花费（D101 定 60 美元、D115 改 100 美元：到线即停）
SPEND_CAP_USD = 100.0
# 单次开机最长小时数的默认值；开机前按「已花 + 盘费 + 本次最坏」检查不超上限
DEFAULT_MAX_HOURS = 1.0
VOLUME_MAX_GB = 200  # 扩盘上限（人定 2026-11-25：放不下可从 150 扩到 200 GB）
MIN_FREE_GB = 30  # 网络盘剩少于这么多 GB 就不开训练：盘满时训练状态和 run.json 都写不进去
VOLUME_RESERVE_HOURS = 720  # 建盘时花费上限按先留一个月盘费检查（评审第 1 轮 #2）
WATCHDOG_GRACE_S = 300  # 本机看门狗比机器内自删晚 5 分钟；花费上限也按「时限 + 宽限」算最坏

# exec 接口限制（openapi PodExecRequest / PodExecResult）
EXEC_TIMEOUT_MAX_S = 25
EXEC_ARG_MAX_CHARS = 4096
EXEC_ARGS_MAX = 256
EXEC_STDOUT_CAP = 1 << 20
# 取回文件每块原始字节数：base64 后约 4/3 倍，须远低于 1 MiB 输出上限
FETCH_CHUNK_BYTES = 512 * 1024
# 上传每次的 base64 参数块数（每块 ≤ 4096 字符，留几个位置给 sh -c 本身）
PUT_ARGS_PER_CALL = 240

COSTS_TSV_COLUMNS = (
    "ts_utc", "action", "resource_id", "gpu", "data_center",
    "usd_per_hr", "started_utc", "ended_utc", "hours", "est_usd", "billed_usd", "note",
)

KEY_NAME = "RUNPOD_API_KEY"


def volume_usd_per_month(size_gb: int = VOLUME_SIZE_GB) -> float:
    return round(size_gb * VOLUME_USD_PER_GB_MONTH, 4)
