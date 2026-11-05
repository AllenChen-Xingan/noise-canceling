# 冻结基准（spectrum_golden）新旧对照 · 调研 026（#119）

为什么改：音色量法（V-TIMBRE）换成 ERB 30 带的标准差，旁报字段从 `dev_0_1k / dev_1_2k / dev_2_4k / dev_4_8k` 换成 `slope_db / max_db / bands / quiet_frames / hi_db / lo_db`，所以 `verifiers` 一行的指纹必须变。

| 行 | 旧（main 量法） | 新 | 结论 |
|---|---|---|---|
| `verifiers` | `c493e149007c4c81`，158 个值 | `f2a50143a54de6f7`，162 个值 | 只因 V-TIMBRE 变 |
| `verifiers_sans_timbre` | `5a0d90dbaeaccdc0`，146 个值 | `5a0d90dbaeaccdc0`，146 个值 | 逐位不变：其余 verifier 一个数都没动 |
| `spectrogram` · `estoi` · `ref_wiener` · `edges` | — | 不变 | — |

`verifiers_sans_timbre` 是在旧代码上先记下的（提交 656380e1，量法改动之前），换量法后重跑不变，这就是「只有音色变了」的证明。
提交：5c1ed9c8（hook 自动提交，同一提交里还有 `parity/erb.txt` 删掉编译行；提交信息没写原因，原因记在本文件）。
