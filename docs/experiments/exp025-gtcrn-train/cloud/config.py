"""云训练机的常量：唯一正本。其他文件（README、脚本、测试）只从这里引用，不手抄。

出处：D96 及其更正（RTX 3090 · Secure Cloud · vCPU ≥ 16 · 150 GB 网络盘）· 交代（花费上限 20 美元）。
接口：RunPod API v2（https://api.runpod.io/v2/openapi.json，2026-11-16 取，sha256 前 16 位见 SPEC_SHA256_16）。
"""

API_BASE = "https://api.runpod.io"
SPEC_SHA256_16 = "41f48a46a68fd023"

# 机器（D96 更正）
GPU_TYPE_ID = "NVIDIA GeForce RTX 3090"
CLOUD = "SECURE"
MIN_VCPU_PER_GPU = 16
# 镜像锁版本（034 compute.md 第三节第 4 条）；容器只睡觉，活全靠 exec 发进去
IMAGE = "pytorch/pytorch:2.5.1-cuda12.4-cudnn9-runtime"
CONTAINER_DISK_GB = 30
POD_NAME_PREFIX = "vdn-"

# 网络存储盘（D96）
VOLUME_NAME = "vdn-train"
VOLUME_SIZE_GB = 150
VOLUME_USD_PER_GB_MONTH = 0.07  # 034 compute.md 1.4：150 GB → 每月 10.5 美元
VOLUME_MOUNT = "/workspace"

# 花费（交代：上限 20 美元，到线即停）
SPEND_CAP_USD = 20.0
# 单次开机最长小时数的默认值；开机前按「已花 + 盘费 + 本次最坏」检查不超上限
DEFAULT_MAX_HOURS = 1.0

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
