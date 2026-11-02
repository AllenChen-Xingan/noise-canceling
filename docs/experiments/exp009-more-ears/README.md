# exp009 · 更多人耳：用公开众包评分给机器分与作者一人定位

> 2026-10-31 · ears（批次 20261031-b6）· 分支 `research/017-more-ears` · issue #42
> 人原话：「那有没有可能用llm的模拟或者用任何办法去覆盖更多的人耳的反馈？……如果只有我一个人的话，我觉得肯定不够」「我们必须做到这样」。D51：「不用我的微调，就是校准我和大家的听感就好」。
> 调研与方案：`docs/research/017_more-ears/decision.md`；逐线笔记 `docs/research/017_more-ears/notes/`。

## 结论先说

结果见 `results/crowd_ears.md`（第一段，只用标签）与 `results/author_vs_crowd.json`（第二段，要名额，跑完回填本节）。

1. **公开众包评分已经能当「众人耳」用**：URGENT 2024（23 系统 · 6,900 条 · 每条 8 票）、URGENT 2025（26 系统 · 23,400 条）、CHiME-7 UDASE（真实多人嘈杂 · 32 名固定听众）。只下纯文本标签（约 30 MB），不下音频。
2. **最接近众人耳的现成预测器是 SCOREQ**（verifier 候选 V-CROWD-PRED）：对众人均值的同源句配对一致率 URGENT 2024 0.695、URGENT 2025 0.708、CHiME-7 真实录音 0.767；DNSMOS 分别只有 0.641 / 0.657 / 0.485。
3. **作者一人的 0.48 不说明作者偏好特殊**：CHiME-7 上 DNSMOS 对单个固定听众的一致率中位 0.454（0.29–0.69）；按「12 屏 × 每屏 4 对」抽 48 对（作者实际是 9 屏、去重 20 对，见第 6 条），DNSMOS 对一个普通听众 ≤ 0.48 的概率是 0.55。0.48 是 DNSMOS 在真实录音上本来就差，加上 48 对样本太小。
4. **一个人和其余人的一致率约 0.79**（CHiME-7 32 人，中位 0.794，范围 0.603–0.861）：这是「作者像大众」的参照线。4 人均值对另 4 人升到 0.80，人数加多收益很快饱和；**系统排名**要的是条数：URGENT 2024 23 个系统，每系统 50 条 × 1 票时 τ 0.67，100 条 × 4 票 0.87。
5. **单人要多少对才够**：SCOREQ 对单人的一致率，95% 区间半宽 48 对时 ±0.16，100 对 ±0.09，400 对 ±0.06（每屏 4 对）。
6. **第二段（2026-11-02）：作者 vs 大众处在边缘，现在不能说作者偏离**。预注册口径：作者对 SCOREQ 0.375 [0.14, 0.64]（48 对 · 9 屏）< 0.516，判「有证据」。自审后加的敏感性分析：作者答卷去重后只有 20 对、9 屏都是同 3 个模型，按同构结构重抽参照线为 0.389，作者 0.40 不过线（普通听众 3.1% 这么低）。三台评分机器彼此一致 0.74–0.80（同类训练来源，不独立）。最大一处分歧是作者多选 GTCRN、SCOREQ 与 Distill-MOS 平均分都是 DPDFNet-2 最高；信号上 DPDFNet-2 停顿处背景压得更低，GTCRN 人声音色略接近原声（`results/compare_models.json`，`../../research/017_more-ears/notes/I-why-scorers-pick-dpdfnet.md`）。详见 `results/author_vs_crowd.json` · `results/author_diagnosis.json`
7. **大众已评的盲听两题**（build `20261102132015-ccd3`）：URGENT 2026 配对打分集里 1 中 1 英、众人意见一致的两对，揭盲表在 `builds/20261102132015-ccd3/`；答卷回来放 `results/`

这些数字只说明公开数据上的情况；**在我们自己的片段上，任何预测器都还没有过线（尚无判据，待测）**。

## 预注册（第二段跑之前写死，2026-10-31）

verifier **V-AUTHOR-VS-CROWD**（QA 层 · 人工层定标用）：
- 输入：作者某一轮盲听答卷 + 作者实际听到的音频。用 SCOREQ（natural · NR）打分，按 exp007 定标配对口径算作者与 SCOREQ 的方向一致率（剔除平局）
- 参照：`crowd_ears.json` → `chime7.listener.author_like_48.scoreq`，即「CHiME-7 固定听众 × 同样 48 对」抽样分布（2.5% / 中位 / 97.5% = 0.516 / 0.72 / 0.893）
- 判：作者一致率 < 2.5% 分位（0.516）→「作者偏离大众：有证据（SCOREQ 代理）」；否则「无证据」。置信度：低（众人分布来自英语 ACR，作者是 2AFC + 中英混合；SCOREQ 在我们片段上的准确度未测）
- 不改阈值、不挑子集；平局两种口径都报，判只看「剔除平局」那一个
- 事后发现（2026-11-02 自审）：参照按 12 屏 × 每屏 4 个独立配对抽，作者实际 9 屏、同 3 个模型、去重 20 对，参照线偏严。下一轮预注册改用同构参照、按去重配对计数（`diagnose_author.py` 已实现）

## 怎么跑

```bash
python3 docs/experiments/exp009-more-ears/fetch_labels.py          # 下标签 → ~/vdn-data/public-mos/，写 data-manifest.json（哈希进仓）
<主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp009-more-ears/crowd_ears.py   # 第一段，约 3 分钟，不占名额
<主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp009-more-ears/regen_exp003.py # 重新生成作者听过的音频（要名额）
<主目录>/_tmp/exp009-venv/bin/python docs/experiments/exp009-more-ears/score_author.py # 第二段（要名额；SCOREQ 首次下载 ONNX 378 MB 到 ~/.cache/scoreq/；12 段 × 4 个版本 = 48 个文件（每个约 10.6 秒、16 kHz），三个评分模型合计墙钟 41 秒 CPU）
<主目录>/_tmp/exp009-venv/bin/python docs/experiments/exp009-more-ears/diagnose_author.py   # 事后诊断 + 同构敏感性分析
<主目录>/_tmp/exp009-venv/bin/python docs/experiments/exp009-more-ears/compare_models.py    # 信号对比：背景残留 / SI-SDR / 音色 / 高频
<主目录>/_tmp/exp009-venv/bin/python docs/experiments/exp009-more-ears/build_listen.py <build-id>  # 盲听两题页面（先把 2 行 ccr 数据与 4 个 flac 放进 _tmp/exp009-listen/<build-id>/raw/）
# exp009-venv：uv venv --python 3.11；uv pip install scoreq distillmos soundfile numpy onnxruntime librosa pandas requests tqdm torchcodec（torchaudio 新版读音频要 torchcodec）。包的确切版本见 `results/env-exp009.txt`

前置与注意：
- 输入音频：`regen_exp003.py` 读主目录 `_tmp/20261030-exp004/src/`（git 忽略），要先在主目录按 exp004 README 生成 src；它与 exp003 的 clips.json 逐字相同（已核）
- DNSMOS 用主目录 `resources/DNS-Challenge/DNSMOS/`（git 忽略，按 004 克隆）
- `score_author.py` 发现 `results/exp003_predictors.json` 已存在就读缓存；要真重打分加 `--rescore`
- 两个首次运行别并行：`import scoreq` 本身会下载权重，两个进程同时下会读到半截文件（报 INVALID_PROTOBUF）
- 权重哈希（sha256）：SCOREQ `~/.cache/scoreq/onnx-models/adapt_nr_telephone.onnx` d493ba8a…7300b4（378,049,307 B）；Distill-MOS `<venv>/site-packages/distillmos/weights/distill_mos_v7.pt` b18b3ac6…027031a40
- 重生成音频与作者所听一致的两重证据：增益 / 响度 / 峰值差 0.0 dB（`regen_check.json`），DNSMOS 重算与 exp003 差 0.0（`author_vs_crowd.json`）
```

## 文件

| 文件 | 作用 |
|---|---|
| `fetch_labels.py` · `data-manifest.json` | 下载与哈希：vmc2026-track1-meta@ebe09d8 四个子集 + CHiME-7 两个 CSV（HTTP Range 从 3.49 GB 包里只抽 1.7 MB） |
| `crowd_ears.py` → `results/crowd_ears.{json,md}` | A 预测器 vs 众人 · B 固定听众 vs 留一 / 预测器 vs 单人 · 评审团人数曲线 · 作者式 48 对抽样 · 要多少对 · 系统排名要多少评分 |
| `regen_exp003.py` → `results/regen_check.json` | 用 exp003 的 enhance.py 原样重新生成 build f8c3 的音频（输出到 `_tmp/`，不碰 exp003 目录），逐条核增益 / 响度 |
| `score_author.py` → `results/exp003_predictors.json` · `results/author_vs_crowd.json` | SCOREQ / Distill-MOS / DNSMOS 打分 + V-AUTHOR-VS-CROWD |
| `diagnose_author.py` → `results/author_diagnosis.json` | 事后诊断（非预注册）：三台机器彼此一致率、作者各模型胜负、去重 + 同构参照的敏感性分析 |
| `compare_models.py` → `results/compare_models.json` | 12 段上各模型的信号差别（停顿处背景、SI-SDR、音色距离、高低频占比） |
| `build_listen.py` → `builds/<build-id>/` | 大众已评的盲听题：揭盲表 · 清单 · 源行数据（音频与页面在 `_tmp/exp009-listen/`） |
| `results/env-exp009.txt` | exp009-venv 的包版本 |

## 口径与局限

- 配对 = 同一条源句、两个不同系统；任一边打平剔除；区间按源句聚类 bootstrap（种子 20261030）
- CHiME-7 只用 OVRL 一张量表。上游 `vmc2026-track1-meta` 的 CHiME-7 MOS 把 SIG / BAK / OVRL 三表混在一起（`resources/urgent2026_challenge_track2/scripts/data/chime-7-udase-eval/data_prep.py:42-58`），本实验已绕开；是否向上游报告要人定
- URGENT 2024 没有听众 ID，「单票」两边多半是不同的人，只作下界（0.62）
- TMHINT-QI 测试集同源句配对只有 322 对、6 个方法，所有预测器都在 0.45–0.53，样本太小，不下结论
- 公开集里没有 GTCRN / DeepFilterNet / FastEnhancer 这类小因果模型的输出（未查到）
- 许可：URGENT 2024 / 2025 为 NC，只评测；本目录只放汇总数，不再分发标签原文
