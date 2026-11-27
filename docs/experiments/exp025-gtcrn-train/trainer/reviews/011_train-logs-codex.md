# 评审记录 011：训练记录两样（#236，feat/trainer-train-logs）· codex

- 对象：`origin/main...HEAD` 的 diff（`trainlog.py`、`train.py` 接入、`tests/test_trainlog.py`、README）
- 输出与日志：主仓 `_tmp/20261116-b19/codex-trainlogs/`

## 第 1 轮（gpt-6-astra，medium，只读沙箱）：P0×0 P1×2 P2×2

| 编号 | 内容 | 处理 |
|---|---|---|
| 第 1 轮 #1 P1 | soundfile 写盘失败抛 `LibsndfileError`（RuntimeError 子类），`except OSError` 接不住，会停训练 | 属实（本机查 soundfile 0.14.0 的继承链）。交 pi fix2：只在写文件处接 `(OSError, sf.SoundFileError)` |
| 第 1 轮 #2 P1 | 存音频不认停下请求，且排在存 best.pt 之前 | 属实。fix2：传 `tick_stop`，每批、每个文件前查；挪到 `dev_eval()` 最后 |
| 第 1 轮 #3 P2 | 停下时最后一段裁剪窗口没记 | 属实。fix2：循环结束时补记一行 `clip_tail` |
| 第 1 轮 #4 P2 | 「被裁」判据与 torch 不一致：torch 乘 `min(1, max_norm/(norm+1e-6))` | 属实（本机读 torch 源码）。fix2：按同一系数判 |

**收工时的状态（2026-11-27）**：fix2 先红测试已提交（2e620755）；修复改动 pi 做到一半被停，trainer 代提交为 WIP（a3b22d05）。WIP 上 `tests/test_trainlog.py` 18 过；`test_train.py` 与全套没在 WIP 上跑，先红是否每条都由修复转绿未逐条核；README 措辞未改；第 2 轮未跑。
