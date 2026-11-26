# 第二批（CochlScene 3 类）云上登记命令

这份文件给 cloudrun 照着跑：把已解压在网络盘上的 CochlScene 3 类切分、分片、登记成新的训练 build（D106）。
写的人：traindata（2026-11-25）。用一台 CPU 机即可，不用 GPU。不下载任何东西，不删任何东西。

## 0. 前提

- 代码在分支 `feat/data-batch2`。云上的仓库要先切到这个分支的最新提交（分支要先推到远端，由大总管定谁推）。
- 第一批的分片 `/workspace/shards/first-batch` **不动**。新分片写到新目录 `/workspace/shards/batch2`。
- 盘：现在约 110 / 200 GB。新分片要重打全部来源：第一批约 15 GB，加 CochlScene 训练与开发份转 16 kHz 后约 5.6 GB（推算：17,629 条 × 10 秒 × 32 KB/秒），所以要再占约 21 GB，之后约 131 GB。

```bash
cd /workspace/repo-1831886c            # 云上仓库（cloudrun 记的路径）
git fetch origin && git switch feat/data-batch2 && git pull --ff-only
D=docs/experiments/exp025-gtcrn-train/data
PY=.venv/bin/python
export VDN_ROOT=/workspace VDN_SHARDS=/workspace/shards/batch2
```

## 1. 只读检查（把输出原样发 traindata）

```bash
ls /workspace/train-raw/cochlscene/CochlScene/Train/Subway | head -5
find /workspace/train-raw/cochlscene/CochlScene -name '*.wav' | wc -l          # 应为 17629
$PY -c "import soundfile as sf,sys;print(sf.info(sys.argv[1]))" "$(find /workspace/train-raw/cochlscene/CochlScene/Train/Subway -name '*.wav' | head -1)"
du -sh /workspace
```

采样率不用先知道：`shard.py` 按每个 wav 头里读到的采样率重采样到 16 kHz，loader 读分片时断言是 16 kHz。

## 2. 装环境，跑测试

```bash
bash $D/cloud_steps.sh setup
bash $D/cloud_steps.sh test             # 应为 Ran 200 tests … OK（skipped 若干）
```

## 3. 切分（只切 CochlScene，其余来源用仓里已有的表）

```bash
$PY $D/split.py --root /workspace --only cochlscene --inventory   # 应为「cochlscene：17629 个文件」
$PY $D/split.py --root /workspace --only cochlscene
```

- 预期：打印一行摘要，`files` 17629，`per_split` 只有 `train` 与 `dev`，录音人约 9 : 1。
- 如果报「没有一条 unit_regex 认得出来」：文件名里认不出录音人（`userNNNN`）。**停下**，把第 1 步的文件名发 traindata；不要改规则硬跑（改成按文件切会让同一个录音人跨份）。
- 如果报文件数不是 17629：停下，把 `--inventory` 的输出发 traindata。

## 4. 分片（全部来源重打到新目录）

```bash
bash $D/cloud_steps.sh shard            # 来源默认从 recipe.json 导出，含 cochlscene；目录已存在会拒绝
```

如果报目录已存在：不要删旧目录，换一个名字（如 `VDN_SHARDS=/workspace/shards/batch2-<时间>`）再跑，后面两步用同一个名字。

## 5. 登记

```bash
bash $D/cloud_steps.sh register         # 打印新的 build id
cat $D/builds/<新 id>/build.json | $PY -c "import json,sys;b=json.load(sys.stdin);print(b['valid'],b['noise_scenes'],b['missing'])"
```

- `valid` 必须是 True（交集检查过）。
- `noise_scenes.train` 里必须有 `transit-hub` · `rail-air-cabin` · `cowork` · `open-office` 四个新场景，加第一批的 7 个，共 11 个。少了就停下发 traindata。
- 如果报「分片索引里完全没有来源 …」：第 4 步漏了来源，把第 4 步的输出发 traindata。

## 6. 打包带回

```bash
bash $D/cloud_steps.sh pack             # 打印包的路径与 sha256
```

请把这些发给 traindata：
- 包的路径与 sha256（包里有 `splits/cochlscene.*` 与 `builds/<新 id>/`）
- 第 3 步的摘要行、第 5 步的 build id 和那一行输出
- 第 4 步的最后一行（每个来源的成员数、丢掉的 RIR 条数）

traindata 收到后在本机核对哈希，把切分表与 build 记录进仓，再删掉 `split_rules.json` 里的「未建表」一行。
