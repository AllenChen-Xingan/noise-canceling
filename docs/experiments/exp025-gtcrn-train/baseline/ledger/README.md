# 识别费账本快照

账本正本在云上网络盘 `/workspace/asr-ledger/asr-openrouter-ledger.json`（大总管 2026-11-25 定）。每次跑完由 cloudrun 取回一份原样快照，提交在这里（铁律：实验的结果一律进 git）。

| 文件 | 何时 | sha256 | 说明 |
|---|---|---|---|
| `20261125-run480-before-clear.json` | 480 段基线跑完、清待核之前 | `1454fe32…`（全文见下） | calls 1170、spent_usd 0.1053、reserved_usd 0.000405，带 review（一次超时 0 字节，费用未知） |

完整哈希：
- `20261125-run480-before-clear.json`：`1454fe320909ae7c85061ba4febf689cdd56120ad63940e0755c36e7978c501b`

对账（实测，evalcmp 2026-11-25）：calls 1170 = 早先真调用测试 3 次 + 5 段冒烟 30 次 + 480 段缓存里的 1137 条，逐条对得上；每次约 0.00009 美元。OpenRouter 后台当天这把密钥用量 1.104 美元，人说这把密钥别处也用过。
