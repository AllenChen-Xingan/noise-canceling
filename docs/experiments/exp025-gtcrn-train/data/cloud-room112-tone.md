# 补测：酒店房间 Room112 底噪的 250 Hz 单音（并进 trainer 执行单 plan-h100-b4.md 甲段 CPU 机）

写的人：traindata（2026-11-26）。只读，不改仓里任何文件，不下载新数据。CPU 机即可，约 10 分钟（按第四批 A 段办公室 402 条的耗时推算）。
起因：codex 评审第 1 轮指出，筛查表里 Room112 的 248 条都没有单音检查结果，其中 keep 的 209 条在 build bc0e9e7b566421f0 的训练里。
大总管 noise-canceling-meta 定：选 A，补测放进甲段，不另开机。

## 判法（补测前定死，不看结果再改）

- 带回的结果由 traindata 在本机用 `screen_tone.merge` 并进 Room112 的筛查行，数 keep 的 209 条里有几条被单音判成 drop
  （`screen_rules.json` 的 `tone_check`：`first_only` 和 `later_too` 都算有）。
- **不超过 5%（≤ 10 条）**：第一批正式训练照用 bc0e9e7b，开训前清单第 2 项的限制里写明条数。
- **超过 5%（≥ 11 条）**：正式训练前出新 build（新切分表、新来源名、重新分片、重新登记）。冒烟和消融的结果照样有效。

## 命令

```bash
# 代码：feat/data-batch3 的最新提交（traindata 发提交号），与甲段其他步骤同一份 clone
D=docs/experiments/exp025-gtcrn-train/data
PY=.venv/bin/python
$PY -m pip install -r $D/requirements-screen.txt          # 甲段已装过可跳过
mkdir -p /workspace/out/room112-tone
# 清单从仓里的筛查表取（正本），不在云上重新 find
$PY -c "
import json,sys
for l in open('$D/screen/but-reverbdb-silence.jsonl'):
    p=json.loads(l)['path']
    if '/Hotel_SkalskyDvur_Room112/' in p: print(p)" > /workspace/out/room112-tone/room112.txt
wc -l /workspace/out/room112-tone/room112.txt          # 应为 248
$PY $D/diagnose.py --root /workspace --list /workspace/out/room112-tone/room112.txt --tone \
    --out /workspace/out/room112-tone/room112-tone.jsonl
shasum -a 256 /workspace/out/room112-tone/room112-tone.jsonl
```

- 清单不是 248 条就停，把数字发 traindata
- diagnose 打印一行摘要（none · first_only · later_too 各多少条），原样发 traindata

## 请带回

- `room112-tone.jsonl`（约 1.5 MB，按办公室 402 条推算）和它的 sha256、diagnose 的摘要行
- 发给 traindata，抄送大总管 noise-canceling-meta

## 结果（2026-11-26）

- 补测已回仓：`screen/room112-tone.jsonl`，248 行，sha256 `8efa7d6812ed61ecb707648aa0fbb84949ff03114ab4b7cdf7ce344d2396eb73`（提交 9ca0c162）
- diagnose 摘要（云上原样带回）：共 248 条；逐秒电平标准差中位数 1.74 dB；最常见的频谱峰 49 Hz（147 次）；单音逐秒判定 none 210 条 · first_only 0 条 · later_too 38 条
- 合并摘要（本机 `screen_tone.merge`，判据取 `screen_rules.json` 的 `tone_check`）：共 248 条：keep 188 · drop 60（其中因单音新丢 21 条）
- 判定：后来几秒有单音的 38 条里，老表 keep 的 209 条中命中 21 条（10.0%），≥ 11 条 → 超过 5% → 按判法出新 build（第五批）。
  处置：新来源名 `but-reverbdb-silence-b5` + 新筛查表 `screen/but-reverbdb-silence-b5.jsonl`（只有 Room112 那 248 行换成并了单音结论的，其余 1022 行与老表逐字节相同）；
  老表 `screen/but-reverbdb-silence.jsonl` 与 build bc0e9e7b 一个字节不动。
