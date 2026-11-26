# 第四批 B 段：重切 BUT · 重打分片 · 登记基线和 5 个消融变体（共 6 个 build）

这份文件给 cloudrun 照着跑。写的人：traindata（2026-11-26）。CPU 机即可。不下载新数据。抄送大总管 87。
A 段结果和由来见 `cloud-batch4.md`；场景缺口与限制见 `scene-gaps.md`。

## 这一批改了什么（规则与配方，本机已改、已测）

- 会议室（meeting-room）：BUT 3 间办公室 L207 · L212 · Q301 的底噪，只取 01 · 11 · 21 号麦克风。排除前 480 条，L207 收音机布置 78 条由规则排除，筛查加单音后 keep 375 · drop 27（实测，A 段）。C236 不用（D114）
- 食堂（canteen）：PCAFETER 加 PRESTO（大总管 87 定 C）
- 目标场景 4 个：rail-air-cabin · canteen · meeting-room · transit-hub（D113）
- 消融：`recipes/ablation/` 下 5 个变体，由 `ablation_recipes.py` 从 `recipe.json` 生成（#212）
- 冻结的 6 个代码文件没动，所以 build 10aa3a9f 在新代码下仍读得了

## 0. 准备

⚠️ 代码提交号以我消息里发的为准（改名之后的那个，不是 a632a660）。

```bash
# 代码：feat/data-batch3 的提交 33990076（或更新；我发你），新 clone 到 /workspace/repo-<提交前 8 位>
D=docs/experiments/exp025-gtcrn-train/data
PY=.venv/bin/python
export VDN_ROOT=/workspace VDN_SHARDS=/workspace/shards/batch4      # 新目录；已存在会拒绝
du -sh /workspace; df -h /workspace          # 重打分片约再占 23 GB（第三批推算）；开跑前剩余不到 30 GB 先停，问大总管；中途只在剩余不到 10 GB 时停
cd $D && $PY -m unittest discover -s tests -t . 2>&1 | tail -3; cd -  # 全过（skipped 若干）
$PY $D/ablation_recipes.py --check           # 应打印「5 份消融配方与 … 一致」
```

## 1. 重切 BUT 底噪

```bash
$PY $D/split.py --root /workspace --only but-reverbdb-silence-b4 --inventory   # 应为 728 个文件
$PY $D/split.py --root /workspace --only but-reverbdb-silence-b4
```

新来源名 `but-reverbdb-silence-b4`：旧表 `splits/but-reverbdb-silence.json` 是 build 10aa3a9f 在用的，不原地覆盖（2026-11-26 第一次跑在这里被 split.py 拒绝，大总管 87 定用新名字）。旧规则已还原，旧表一个字节不动。

预期摘要：`files` = 375 + 209 = 584（推算：办公室 keep 375 + 酒店房间 keep 209），`excluded_meta` = 78，`excluded_screen` = 27 + 39 = 66。
`per_split` 只有 train 与 dev；4 间房（R112 · L207 · L212 · Q301）在开发份里各至少一个布置。数对不上就停，把摘要发我。

## 2. 分片

```bash
bash $D/cloud_steps.sh shard        # 来源从 recipe.json 导出；5 个变体用同一批分片
```

## 3. 登记 6 个 build（基线 + 5 个变体），每个都过交集检查

```bash
mkdir -p /workspace/out/batch4b
for r in $D/recipe.json $D/recipes/ablation/no-eq.json $D/recipes/ablation/no-clip.json \
         $D/recipes/ablation/no-n_noises.json $D/recipes/ablation/no-reverb.json $D/recipes/ablation/high_snr-on.json; do
  echo "== $r"; $PY $D/loader.py --register --shards "$VDN_SHARDS" --recipe "$r" || { echo "登记失败：$r"; break; }
done 2>&1 | tee /workspace/out/batch4b/register.log
ls $D/builds/                                         # 应多出 6 个新目录
```

每个新 build 请核（用下面这段，一次打完）：

```bash
for b in $D/builds/*/build.json; do $PY -c "
import json,sys; b=json.load(open(sys.argv[1]))
print(b['build_id'], 'valid', b['valid'], 'leak_ok', b['leak_check']['ok'], 'hits', b['leak_check']['n_hits'],
      'demand_segments', b['leak_check']['per_key']['demand_segments'], 'missing', b['missing'],
      'meeting-room' in b['noise_scenes']['train'], b['recipe_sha256'][:12])" "$b"; done
```

- 6 个新 build 都要：`valid` True · `leak_ok` True · `hits` 0 · `demand_segments` 0 · `missing` [] · 第 6 列 True
- 6 个的分片部分相同，只有 `recipe_sha256` 不同（6 个不同的值）
- 任何一个不满足就停，把输出发我

## 4. 场景表 · 打包

```bash
$PY $D/scene_table.py --build $D/builds/<基线 build id> > /workspace/out/batch4b/scenes.md   # 4 个场景
bash $D/cloud_steps.sh pack
```

## 5. 请带回

- 第 1 步摘要、第 3 步 register.log 和核对输出、`scenes.md`
- pack 的包路径与 sha256（含 splits/ 与 builds/）
- `du -sh /workspace`
- first-batch 已于 2026-11-26 07:42Z 按 D112 删除（cloudrun），不用再做
