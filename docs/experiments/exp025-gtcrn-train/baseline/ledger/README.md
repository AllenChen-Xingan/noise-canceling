# 识别费账本快照

账本正本在云上网络盘 `/workspace/asr-ledger/asr-openrouter-ledger.json`（大总管 2026-11-25 定）。每次跑完由 cloudrun 取回一份原样快照，提交在这里（铁律：实验的结果一律进 git）。

| 文件 | 何时 | sha256 | 说明 |
|---|---|---|---|
| `20261125-run480-before-clear.json` | 480 段基线跑完、清待核之前 | `1454fe32…`（全文见下） | calls 1170、spent_usd 0.1053、reserved_usd 0.000405，带 review（一次超时 0 字节，费用未知） |

| `20261125-after-clear.json` | cloudrun 用 `clear_review.py --worst-count 3` 清待核之后（main 048ad638，云上 2026-11-25 20:02 UTC） | `7d4c97ec…` | calls 1173、spent_usd 0.105588、reserved 0、review 已删；`adjustments` 记了 3 笔费用未知按 12 秒 × 0.000008 最坏估 = 0.000288 美元（大总管定 3 笔） |
| `20261125-clear_review.log` | 同上，清待核的运行记录 | `dfcf1eeb…` | 前后 sha256、「没有 vdn-eval 进程」核对、新账本全文 |

清待核时脚本另存的备份 `asr-openrouter-ledger.json.before-clear-20261125T200246Z.json` 与上面的 `20261125-run480-before-clear.json` 逐字节相同（sha256 都是 `1454fe32…`），不重复提交。

完整哈希：
- `20261125-run480-before-clear.json`：`1454fe320909ae7c85061ba4febf689cdd56120ad63940e0755c36e7978c501b`

对账（实测，evalcmp 2026-11-25）：calls 1170 = 早先真调用测试 3 次 + 5 段冒烟 30 次 + 480 段缓存里的 1137 条，逐条对得上；每次约 0.00009 美元。OpenRouter 后台当天这把密钥用量 1.104 美元，人说这把密钥别处也用过。
- `20261125-after-clear.json`：`7d4c97ec5dfd87493ff0474d3c956903e90b5e88ecf680483a047d4708a664c8`
- `20261125-clear_review.log`：`dfcf1eebff39444d1dac59e66b7c83e3d13218d8cf41e6c772bae64f213e7f60`
