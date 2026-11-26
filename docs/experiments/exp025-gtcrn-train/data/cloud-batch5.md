# 第五批：Room112 去掉带单音的 21 条 · 重切 BUT · 重打分片 · 登记基线和 5 个消融变体（共 6 个 build）

这份文件给 cloudrun 照着跑。写的人：traindata（2026-11-26）。CPU 机即可。不下载新数据。抄送大总管 noise-canceling-meta。
起因与判法见 `cloud-room112-tone.md`（#221）；场景缺口与限制见 `scene-gaps.md`。

## 这一批改了什么（规则与配方，本机已改、已测）

- 酒店房间（hotel-room）Room112 补了 250 Hz 单音检查：248 条里 later_too 38 条；原 keep 209 条里 21 条有单音（10.0%，超过 5%），
  这一批去掉，keep 188 条（实测，本机 `screen_tone.merge`）
- 新筛查表 `screen/but-reverbdb-silence-b5.jsonl`：只有 Room112 的行并进了单音结论，其他行与旧表逐字节相同（有测试）
- 新来源名 `but-reverbdb-silence-b5`（4 间房，规则除筛查表外与 `-b4` 相同）。`-b4` 的表 build bc0e9e7b 在用，不原地覆盖
- 会议室（办公室 L207 · L212 · Q301）、食堂、目标场景、消融旋钮都和第四批相同
- 冻结的 6 个代码文件没动，所以 build 10aa3a9f、bc0e9e7b 在新代码下仍读得了

## 0. 准备

⚠️ 现在冒烟和消融正在 `/workspace/shards/batch4` 上跑。这一批只**新建** `/workspace/shards/batch5`，不碰 batch4。

```bash
# 代码：feat/data-batch5 的提交（traindata 发提交号；recipe.json sha256 应为 7de5c4b5…），新 clone 到 /workspace/repo-<提交前 8 位>
D=docs/experiments/exp025-gtcrn-train/data
PY=$PWD/.venv/bin/python        # 绝对路径：下面会进数据目录跑测试（codex 第五批第 1 轮 #3）
set -o pipefail
export VDN_ROOT=/workspace VDN_SHARDS=/workspace/shards/batch5      # 新目录；已存在会拒绝
du -sh /workspace; df -h /workspace          # 新分片约占 22 GB（第四批实测）；开跑前剩余不到 25 GB 先停，问大总管；中途只在剩余不到 10 GB 时停
( cd $D && $PY -m unittest discover -s tests -t . 2>&1 | tail -3 ) || { echo '测试没过，停'; exit 1; }  # 全过（skipped 若干）
$PY $D/ablation_recipes.py --check           # 应打印「5 份消融配方与 … 一致」
$PY $D/screen_tone.py --screen $D/screen/but-reverbdb-silence.jsonl --tone $D/screen/room112-tone.jsonl \
    --base $D/screen/but-reverbdb-silence.jsonl --only-regex '/Hotel_SkalskyDvur_Room112/' \
    --check $D/screen/but-reverbdb-silence-b5.jsonl   # 应打印「逐字节一致」和「keep 188 · drop 60（其中因单音新丢 21 条）」
```

## 1. 切 BUT 底噪（新来源名）

```bash
$PY $D/split.py --root /workspace --only but-reverbdb-silence-b5 --inventory   # 应为 728 个文件
$PY $D/split.py --root /workspace --only but-reverbdb-silence-b5
```

只切 `-b5`：别的来源的表都已在仓里，不重写；`-b4` 和 `but-reverbdb-silence` 两张旧表一个字节都不动。

预期摘要（推算：第四批实测数减去 Room112 新丢的 21 条）：

- `files` = 375 + 188 = 563
- `excluded_meta` = 78
- `excluded_screen` = 27 + 60 = 87

`per_split` 只有 train 与 dev。4 间房（R112 · L207 · L212 · Q301）在开发份里各至少一个布置。
来源名变了，哪几个布置进开发份会和第四批不同，这是预期的。数对不上就停，把摘要发我。

## 2. 分片

```bash
bash $D/cloud_steps.sh shard        # 来源从 recipe.json 导出（BUT 只取 -b5）；5 个变体用同一批分片；第四批实测约 2458 秒
```

## 3. 登记 6 个 build（基线 + 5 个变体），每个都过交集检查

```bash
mkdir -p /workspace/out/batch5
for r in $D/recipe.json $D/recipes/ablation/no-eq.json $D/recipes/ablation/no-clip.json \
         $D/recipes/ablation/no-n_noises.json $D/recipes/ablation/no-reverb.json $D/recipes/ablation/high_snr-on.json; do
  echo "== $r"; $PY $D/loader.py --register --shards "$VDN_SHARDS" --recipe "$r" || { echo "登记失败：$r"; exit 1; }
done 2>&1 | tee /workspace/out/batch5/register.log || { echo "登记没全过，停：不要往下核对与打包"; exit 1; }  # 靠上面的 pipefail（codex 第 2 轮 #3）
ls $D/builds/                                         # 应多出 6 个新目录
```

每个新 build 请核（一次打完）：

```bash
for b in $D/builds/*/build.json; do $PY -c "
import json,sys; b=json.load(open(sys.argv[1]))
print(b['build_id'], 'valid', b['valid'], 'leak_ok', b['leak_check']['ok'], 'hits', b['leak_check']['n_hits'],
      'demand_segments', b['leak_check']['per_key']['demand_segments'], 'missing', b['missing'],
      'meeting-room' in b['noise_scenes']['train'], b['recipe_sha256'][:12],
      sorted(b['split_tables']).count('but-reverbdb-silence-b5'))" "$b"; done
```

- 6 个新 build 都要：`valid` True · `leak_ok` True · `hits` 0 · `demand_segments` 0 · `missing` [] · 第 6 列 True · 最后一列 1
- 6 个的分片部分相同，只有 `recipe_sha256` 不同（6 个不同的值）
- 任何一个不满足就停，把输出发我

## 4. 场景表 · 打包

```bash
$PY $D/scene_table.py --build $D/builds/<基线 build id> > /workspace/out/batch5/scenes.md   # 4 个场景
bash $D/cloud_steps.sh pack
```

## 5. 请带回

- 第 1 步摘要、第 3 步 register.log 和核对输出、`scenes.md`
- pack 的包路径与 sha256（含 splits/ 与 builds/）
- `du -sh /workspace; df -h /workspace`

## 6. 删 batch4 之前（先只做取回；删不删等大总管问过人再发话）

新分片加上后，盘剩约 22 GB，低于正式训练要求的 30 GB。所以正式训练前要删 `/workspace/shards/batch4`。
⚠️ 删掉以后，bc0e9e7b 那 6 个 build（冒烟、消融在用）就读不了。要删，必须同时满足三条：

1. 第 3 步的 6 个新 build 全部合格
2. 下面的小文件和校验值已取回本机，并核对一致
3. 冒烟和消融已经跑完，大总管明确说「删」

```bash
# 只读：取回 batch4 的索引与全部分片的校验值（约 22 GB 要算哈希，几分钟）
mkdir -p /workspace/out/batch4-final
cp /workspace/shards/batch4/index.json /workspace/out/batch4-final/
( cd /workspace/shards/batch4 && find . -type f ! -name index.json -print0 | sort -z | xargs -0 sha256sum ) \
    > /workspace/out/batch4-final/shards.sha256
du -sb /workspace/shards/batch4 > /workspace/out/batch4-final/du.txt
sha256sum /workspace/out/batch4-final/index.json        # 应等于 build bc0e9e7b 的 index_sha256 311134757bf6…
tar -czf /workspace/out/batch4-final.tar.gz -C /workspace/out batch4-final && sha256sum /workspace/out/batch4-final.tar.gz
```

- 把 `batch4-final.tar.gz` 和它的 sha256 带回本机，发 traindata 核对
- 第 5 行的哈希对不上就停，不删
- 删的命令等大总管发话时再给，这份文件里不写
