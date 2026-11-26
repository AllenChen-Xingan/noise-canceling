# 第四批：会议室噪声池（D114）· 新训练 build

这份文件给 cloudrun 照着跑。写的人：traindata（2026-11-26）。CPU 机即可。不下载新数据。
分两段：**A 段只读**（诊断与筛查，结果带回给 traindata）→ traindata 改规则、推送 → **B 段登记新 build**。
B 段的命令要等 A 段的结果改完规则才能跑，所以两段之间要停一次。

## 为什么不用 C236（D114 的机器判定）

人定 D114：本机逐秒测 C236 的 250 Hz 单音。只在第 1 秒，就去掉开头 1 秒；第 2–10 秒也有，会议室池就不用 C236。

本机测了 `_tmp/20261126-c236-listen/` 的 3 段（实测，sha256 都对得上）。工具是 `diagnose.py --tone`，判据在 `screen_rules.json` 的 `tone_check`：
250 ± 5 Hz 的频点功率比 150–350 Hz 的中位数高 10 dB 以上，这一秒就算「有」。结果存在 `screen/c236-tone-local3.jsonl`。

| 片段 | 有单音的秒 | 第 1 秒高出 | 第 2 秒 | 第 3 秒 | 第 4–10 秒 |
|---|---|---|---|---|---|
| 1（筛查 keep） | 1 | 28.2 dB | 4.7 dB | −0.9 dB | −0.9–2.6 dB |
| 2（筛查 drop） | 1、2 | 36.7 dB | 13.5 dB | 8.3 dB | 4.7–9.0 dB |
| 3（筛查 drop） | 1、2、3 | 33.1 dB | 17.7 dB | 14.9 dB | 0.7–5.6 dB |

- 3 段里有 2 段第 2 秒以后也有 250 Hz → 按 D114，**会议室池不用 C236**
- ⚠️ 结论依赖阈值。10 dB 是开测前写进 json 的暂定值；如果阈值改成 20 dB，3 段都只在第 1 秒。按 D114，阈值不为凑数而改
- 第 2–3 秒的单音只有约 −63 到 −72 dBFS，比第 1 秒低 35–40 dB。推断：可能是开头那一声的余响，也可能是一直在的弱单音（片段 2 每秒都高 5–9 dB）。没听过
- 样本只有 3 段。A 段第 2 步在云上测全部 620 段，补足样本量

## 会议室的备选来源：BUT 的 3 间办公室

- 原下载计划就写了「会议室用 C236 与 3 间办公室的底噪」（`download-list-but.md` 第 5 节第 4 条）
- 房间：L207 · L212 · Q301。论文写混响时间 RT30 0.61–0.78 秒、体积 98–192 立方米，和小会议室相近（`gap-fill-options.md` 第 3 节，读文献，待核）
- 已在云盘 `/workspace/train-raw/but-reverbdb/`，许可同为 Apache 2.0（组 A7），不用新下载
- L207 有 2 份布置带第 2 个扬声器（放收音机），要排除（已核，见交接页 2026-11-26）
- 已知限制：这是办公室，不是会议室；房间少（3 间）；没有会议突发声（门、椅子、键盘）

---

## A 段（只读，不改仓、不删东西）

### A0 准备

```bash
# 代码：feat/data-batch3 最新提交（推送后发提交号），新 clone 到 /workspace/repo-<提交前 8 位>
D=docs/experiments/exp025-gtcrn-train/data
PY=.venv/bin/python
$PY -m pip install -r $D/requirements-screen.txt
du -sh /workspace; df -h /workspace          # 盘余量请发我（上次只剩约 32 GB）
cd $D && $PY -m unittest discover -s tests -t . 2>&1 | tail -3; cd -     # 全过（skipped 若干）
```

### A1 列出办公室底噪（排除收音机布置）

```bash
mkdir -p /workspace/out/batch4 && cd /workspace
ls train-raw/but-reverbdb/                                       # 请把目录名原样发我
find train-raw/but-reverbdb -path '*L207*/silence/*.wav' -o -path '*L212*/silence/*.wav' -o -path '*Q301*/silence/*.wav' \
  | sort > out/batch4/office-all.txt
: > out/batch4/office.txt; : > out/batch4/office-radio.txt
while read -r f; do
  m=$(dirname "$f")/../../spk_meta.txt
  [ -f "$m" ] || { echo "缺 meta: $f"; break; }
  if grep -q '^\$EnvSpk2' "$m"; then echo "$f" >> out/batch4/office-radio.txt; else echo "$f" >> out/batch4/office.txt; fi
done < out/batch4/office-all.txt
wc -l out/batch4/office-all.txt out/batch4/office.txt out/batch4/office-radio.txt
cut -d/ -f3 out/batch4/office.txt | sort | uniq -c              # 每间房多少条
cd -
```

- 打印「缺 meta」就停，把那行发我
- `office-radio.txt` 应只有 L207 的条目

### A2 逐秒单音：全部 C236 + 办公室

```bash
$PY $D/diagnose.py --root /workspace --list /workspace/out/batch3/c236.txt   --tone --out /workspace/out/batch4/c236-tone.jsonl
$PY $D/diagnose.py --root /workspace --list /workspace/out/batch4/office.txt --tone --out /workspace/out/batch4/office-tone.jsonl
```

两次各打印一行摘要（none · first_only · later_too 各多少条），请原样发我。

### A3 办公室底噪过筛查（场景按 meeting-room 的判据）

```bash
$PY $D/screen.py --root /workspace --list /workspace/out/batch4/office.txt --scene meeting-room \
  --out /workspace/out/batch4/screen-office.jsonl
```

打印的「keep N · drop M」和各理由的次数请原样发我。混响时间一项记「未查」是预期。

### A4 请带回

`/workspace/out/batch4/` 整个目录（只有文本，几 MB）和它的 sha256。然后**停在这里**：B 段等我改完规则、推送新提交。

---

## B 段（A 段结果回来后，traindata 改规则再给提交号）

traindata 在 A 段之后要做的事（本机，不跑重活）：
1. `split_rules.json`：`but-reverbdb-silence` 的 glob 加 3 间办公室；`expect.files` 改成 A1 的总数；筛查表并入办公室的结论
2. `recipe.json`：`noise_pools` 加 `meeting-room`（来源 but-reverbdb-silence，正则只认 3 间办公室）；`target_scene.scene` 加 `meeting-room`，共 4 个
3. `recipe.json`：`canteen` 池加 DEMAND PRESTO（大总管 87 定 C，`scene-gaps.md` 2.5 节；按段核 exclude.json 交集为 0，训练份 0.67 小时一秒没少）
4. `scene_notes.json` 写会议室与食堂的已知限制；补测试
5. 分支情况：
   - 办公室没有开头单音（A2 全是 none）→ 照上面做，不动冻结的 6 个代码文件
   - 办公室也是「只在第 1 秒」→ 要去掉开头 1 秒。这得改 `split.py`（冻结文件之一），改完旧 build 10aa3a9f 在新代码下读不了。先报大总管再做
   - 办公室第 2 秒以后也有单音，或筛查留下太少（不到一半）→ 会议室没有合规来源，报大总管，由人定（例如这次先练 3 个场景）

B 段命令（照第三批）：

```bash
export VDN_ROOT=/workspace VDN_SHARDS=/workspace/shards/batch4     # 新目录；已存在会拒绝
$PY $D/split.py --root /workspace --only but-reverbdb-silence --inventory
$PY $D/split.py --root /workspace --only but-reverbdb-silence
bash $D/cloud_steps.sh shard
bash $D/cloud_steps.sh register
bash $D/cloud_steps.sh pack
$PY $D/scene_table.py --build $D/builds/<新 build id>        # 4 个场景的噪声表
```

- 盘：重打全部分片约再占 23 GB（第三批的推算）。余量不到 30 GB 先停。按 D112，first-batch 可在本次 build valid 并取回核对后删；删前问 trainer
- register 后核：`valid` 为 True；`leak_check.ok` 为 True、`n_hits` 为 0；`noise_scenes.train` 含 `meeting-room`；`missing` 为 `[]`
- 带回：pack 的包与 sha256、build id、`scene_table.py` 的输出
