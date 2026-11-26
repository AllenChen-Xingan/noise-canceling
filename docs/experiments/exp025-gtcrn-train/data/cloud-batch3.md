# 第三批（BUT 会议室与酒店房间底噪）云上命令

这份文件给 cloudrun 照着跑：BUT 底噪先过机器筛查，再切分、分片、登记成新的训练 build（issue #205，人定 D110 · D111）。
写的人：traindata（2026-11-26）。CPU 机即可。只在第 0 步装两个 pip 包、PANNs 第一次运行时自己下一个模型文件；不下其他数据，不删任何东西。

## 0. 准备

- 代码：分支 `feat/data-batch3` 的最新提交（推送后我把提交号发你）。新 clone 到 `/workspace/repo-<提交前 8 位>`，用共享 venv
- 盘：BUT 已解在 `/workspace/train-raw/but-reverbdb/`。新分片要重打全部来源，推算再占约 23 GB：
  - 第一批加 CochlScene 约 21 GB（照第二批实测的量）
  - BUT 两个房间的训练与开发份：868 条 × 60 秒 × 32 KB/秒 ≈ 1.7 GB，排除收音机和筛掉的之后更少
  - 开跑前请 `du -sh /workspace`；超过 160 GB 就先停，问大总管旧分片（`first-batch`、`batch2`）删不删
- 装筛查要的两个模型包，并把实际装上的版本发我（我钉进 requirements）：

```bash
D=docs/experiments/exp025-gtcrn-train/data
PY=.venv/bin/python
export VDN_ROOT=/workspace VDN_SHARDS=/workspace/shards/batch3
$PY -m pip install silero-vad panns-inference
$PY -m pip freeze | grep -iE 'silero|panns|torch|librosa|torchlibrosa'
```

- PANNs 第一次运行会把 CNN14 模型文件（约 300 MB）和标签表下到 `~/panns_data/`。它用 `wget` 下载（读源码的旧记录）；镜像里没有 `wget` 就先装上

## 1. 测试

```bash
cd $D && $PY -m unittest discover -s tests -t . 2>&1 | tail -3; cd -     # 应为 Ran 276 tests … OK（skipped 若干）
```

## 2. 列出候选（已排除放收音机的布置）

```bash
mkdir -p /workspace/out/batch3
$PY $D/split.py --root /workspace --only but-reverbdb-silence --list-files > /workspace/out/batch3/but-list.txt
wc -l < /workspace/out/batch3/but-list.txt                 # 868 减去收音机布置的条数
grep /VUT_FIT_C236/ /workspace/out/batch3/but-list.txt > /workspace/out/batch3/c236.txt
grep /Hotel_SkalskyDvur_Room112/ /workspace/out/batch3/but-list.txt > /workspace/out/batch3/r112.txt
wc -l /workspace/out/batch3/c236.txt /workspace/out/batch3/r112.txt
```

如果报「找不到 meta 文件」：某个布置目录里没有 `spk_meta.txt`。**停下**，把那条报错发我。

## 3. 机器筛查（人声 · 场景标签 · 平稳度）

```bash
$PY $D/screen.py --root /workspace --list /workspace/out/batch3/c236.txt --scene meeting-room --out /workspace/out/batch3/screen-c236.jsonl
$PY $D/screen.py --root /workspace --list /workspace/out/batch3/r112.txt --scene hotel-room  --out /workspace/out/batch3/screen-r112.jsonl
mkdir -p $D/screen
cat /workspace/out/batch3/screen-c236.jsonl /workspace/out/batch3/screen-r112.jsonl > $D/screen/but-reverbdb-silence.jsonl
wc -l $D/screen/but-reverbdb-silence.jsonl                 # 应等于第 2 步的总行数
```

- 两次都会打印「keep N · drop M」和各条理由的次数，请原样发我
- **任一房间 keep 少于一半，先别往下跑**，把两段打印发我。判据阈值全是暂定，可能要先调
- 会议室这一类的「混响时间」一项会记「未查」，这是预期：这次只筛底噪，不喂冲激响应

## 4. 切分（只切 BUT 底噪，其余来源用仓里已有的表）

```bash
$PY $D/split.py --root /workspace --only but-reverbdb-silence
```

预期：打印一行摘要，含 `excluded_meta`（收音机布置的条数）、`excluded_screen`（筛掉的条数）、`unit_from`。`per_split` 只有 train 与 dev。两个房间在开发份里都至少有一个布置。

## 5. 分片 · 登记 · 打包

```bash
bash $D/cloud_steps.sh shard            # 来源默认从 recipe.json 导出，含 but-reverbdb-silence；目录已存在会拒绝
bash $D/cloud_steps.sh register
bash $D/cloud_steps.sh pack
cp $D/screen/but-reverbdb-silence.jsonl /workspace/out/batch3/   # pack 不收 screen/，单独带回
```

register 后核这四样：
- `valid` 为 True
- `noise_scenes.train` 有 13 个场景，比第二批多 `meeting-room` 和 `hotel-room`
- `scene_groups.train` 仍是 `{"cowork": ["cowork", "open-office"]}`
- `missing` 为 `[]`

## 6. 请发 traindata

- 第 0 步的 pip 版本行
- 第 2 步的行数（总数 · C236 · R112）
- 第 3 步两段打印
- 第 4 步的摘要行
- 第 5 步 shard 的最后一行、新 build id、上面四样的核对结果、pack 的路径与 sha256
- 取回本机：pack 的包、`screen/but-reverbdb-silence.jsonl`、`/workspace/out/batch3/` 下的日志

traindata 收到后核对，把切分表、筛查表、build 记录进仓，删掉 `split_rules.json` 里「未建表」那一行。
