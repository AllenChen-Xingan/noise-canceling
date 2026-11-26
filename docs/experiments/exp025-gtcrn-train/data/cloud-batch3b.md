# 第三批续跑（只登记酒店房间 R112 · 诊断 C236 · 取 3 段给人听）

这份文件给 cloudrun 下次开机照着跑（人定 D113，2026-11-26）。写的人：traindata。CPU 机即可。
不下载新数据，不删任何东西（first-batch 按 D112 等本次 build 核为 valid、产物取回核对后再删，另行通知）。

这次 build **只用来提前验证数据管线，不正式训练**（D110：全部就绪再开训）。正式训练等会议室池登记后的那一批。

## 0. 准备

- 代码：`feat/data-batch3` 的最新提交（推送后我发提交号）。新 clone 到 `/workspace/repo-<提交前 8 位>`；共享 venv 加筛查模型包：

```bash
D=docs/experiments/exp025-gtcrn-train/data
PY=.venv/bin/python
export VDN_ROOT=/workspace VDN_SHARDS=/workspace/shards/batch3
$PY -m pip install -r $D/requirements-screen.txt
du -sh /workspace                                   # 超过 160 GB 先停，问大总管
```

## 1. 测试

```bash
cd $D && $PY -m unittest discover -s tests -t . 2>&1 | tail -3; cd -      # 全过（skipped 若干）
```

## 2. C236 诊断（只读）

每个扬声器布置取 1 号麦克风的第一段，共 10 条：

```bash
mkdir -p /workspace/out/batch3b
grep '/01/silence/silence_16kHz_10sec.v00.wav$' /workspace/out/batch3/c236.txt > /workspace/out/batch3b/c236-diag.txt
wc -l < /workspace/out/batch3b/c236-diag.txt       # 应为 10
$PY $D/diagnose.py --root /workspace --list /workspace/out/batch3b/c236-diag.txt --out /workspace/out/batch3b/c236-diag.jsonl
```

请把打印的摘要和 `c236-diag.jsonl` 带回。

## 3. 取 3 段 C236 给人听（人已同意下载，只作评估，不进训练、不进仓）

```bash
mkdir -p /workspace/out/batch3b/listen
cp /workspace/train-raw/but-reverbdb/VUT_FIT_C236/MicID01/SpkID01_20190503_S/21/silence/silence_16kHz_10sec.v01.wav /workspace/out/batch3b/listen/1-keep_SpkID01_mic21_v01.wav
cp /workspace/train-raw/but-reverbdb/VUT_FIT_C236/MicID01/SpkID05_20190503_S/25/silence/silence_16kHz_10sec.v01.wav /workspace/out/batch3b/listen/2-drop-tuningfork_SpkID05_mic25_v01.wav
cp /workspace/train-raw/but-reverbdb/VUT_FIT_C236/MicID01/SpkID10_20190503_S/20/silence/silence_16kHz_10sec.v00.wav /workspace/out/batch3b/listen/3-drop-speechfrac_SpkID10_mic20_v00.wav
sha256sum /workspace/out/batch3b/listen/*.wav
```

三段各 10 秒、16 kHz，共约 1 MB。取回到主仓 `_tmp/20261126-c236-listen/`（不进仓），并把 sha256 一起发来。
挑法：1 = 筛查 keep 的；2 = drop，前 5 个标签首位是 Tuning fork、电平起伏居中；3 = drop，说话帧占比最高（0.042）。

## 4. 切分（只切 BUT 的 R112）

```bash
$PY $D/split.py --root /workspace --only but-reverbdb-silence --inventory     # 应为 248 个文件
$PY $D/split.py --root /workspace --only but-reverbdb-silence
```

- 筛查结果已在仓里（`screen/but-reverbdb-silence.jsonl`），不用重筛
- 预期摘要：`files` = 209（248 减去筛掉的 39），`excluded_meta` 为 0（R112 没有收音机布置），`excluded_screen` = 39；`per_split` 只有 train 与 dev，开发份至少一个布置

## 5. 分片 · 登记 · 打包

```bash
bash $D/cloud_steps.sh shard
bash $D/cloud_steps.sh register
bash $D/cloud_steps.sh pack
```

register 后核：
- `valid` 为 True
- `noise_scenes.train` 有 12 个场景：第二批的 11 个加 `hotel-room`；**没有** `meeting-room`
- `scene_groups.train` 仍是 `{"cowork": ["cowork", "open-office"]}`
- `missing` 为 `[]`

## 6. 请发 traindata

- 第 2 步的摘要与 jsonl
- 第 3 步三段的 sha256（文件取回到 `_tmp/20261126-c236-listen/`）
- 第 4 步的摘要行
- 第 5 步 shard 的最后一行、build id、四样核对、pack 的路径与 sha256
- 盘用量（`du -sh /workspace`）
