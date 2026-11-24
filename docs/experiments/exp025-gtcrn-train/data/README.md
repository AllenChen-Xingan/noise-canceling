# exp025 · data — training data for the GTCRN fine-tune

Download, split, and mix the training data on the cloud box; the Mac only runs code and unit tests (D88 · D97).
Owner: worker `traindata` (issue #156). Plan: `docs/plan/014_training-kickoff.md`, step 0 ③ and step 2.

## 一、这一节有什么用

这里写数据怎么下、怎么切、怎么混，以及 trainer 怎么取数据。每样东西的正本在哪个文件，见第二节。

## 二、文件与正本

| 文件 | 管什么 | 正本 / 第二处表示怎么核 |
|---|---|---|
| `sources.json` | 要下载的每个文件：地址 · 字节数 · 上游 md5 · 许可 | 正本。`fetch.py --head` 核服务器报的大小 |
| `fetched.json` | 第一次在云上下完记下的 sha256（上游没给的那些） | 云上生成后进仓 |
| `split_rules.json` | 切分比例与单位 | 是 027 `decision.md` 第六节表的第二处表示；`test_split_leak.Rules` 逐行比对 |
| `splits/<来源>.jsonl` + `.json` | 每个文件进哪一份、sha256、段界 | 云上生成后进仓；摘要里记 jsonl 的哈希，读时核 |
| `recipe.json` | 混音旋钮、默认值、出处 | 正本。早期混响长度**不在这里**：正本是 exp013 `rooms.py` 的 `EARLY_REVERB_MS` 与 `early_rir` |
| `mix.py` | `plan`（读片段 + 抽随机数）与 `render`（CPU 参考版） | CPU 版是混音算法的正本 |
| `gpu_mix.py` | 同一套步骤的 torch 批量版（GPU） | `test_gpu_mix` 对拍 CPU 版：逐样本差 ≤ 1e-4；电平与有声区间逐个对拍 exp013 `dsp.py` |
| `leakcheck.py` | 训练原料和 exp013 评测素材求交集 | 正本是 exp013 build `20261103230756-17f5` 的 `exclude.json`；测试核它等于 exp013 `leak.exclude_table` 的输出 |
| `shard.py` | 转 16 kHz、打 tar 分片、写成员偏移索引 | — |
| `loader.py` | 给 trainer 的接口；登记训练 build | `builds/<id>/build.json` 进仓 |
| `exp013_inputs.py` · `exp013_inputs.json` | exp013 的 1,994 个原料逐文件 sha256 | 文件清单和 exp013 `pools.index_hash()` 比对，也和旧 build 的 `material_index` 比对 |
| `rebuild_exp013.py` | 云上重建 exp013，逐文件核 4,540 个旧哈希 | 报告写 `rebuild/exp013-<时间>.json` |
| `cloud_steps.sh` · `requirements.txt` | 云上的操作步骤与钉死的版本 | 版本与 exp013 旧 build 一致（`test_rebuild`） |

## 三、切分与隔离

切分规则照 027 第六节，分四份：训练 · 开发 · 终测原料池 · D63 测试份。
测试份只用来测，不进任何合成原料。
切法是确定的：每个单位（说话人、房间或文件）按 `sha256(种子:来源:单位)` 排序，再按比例分。
分配用最大余数法，所以各份个数加起来正好等于总数。

交集检查查的是训练、开发、终测原料池三份里能取到的**全部**原料，不只是某一次抽到的那些。
检查对照 exp013 的 `exclude.json`，查这几类：说话人 · DEMAND 时段 · ESC-50 · 实测 RIR · 房间噪声 · 中文与音乐 · 合成函数。
exclude.json 里只要出现不认识的键，检查就直接红。

## 四、给 trainer 的接口（2026-11-16 与 trainer 商定，2026-11-24 改成在 GPU 上混音）

```python
from loader import torch_iterable, render_on_gpu, DevSet
it = torch_iterable(shards, "recipe.json", split="train", seed=0, seg_s=4.0, batch_size=32, start_batch=0)
dl = DataLoader(it, batch_size=None, num_workers=N)      # worker 只读片段 + 抽随机数（mix.plan）
for b in dl:
    out = render_on_gpu(b, "cuda")                       # 在 GPU 上批量混音（gpu_mix.render_batch）
    out["noisy"], out["clean"]                           # [B, T] float32，在 GPU 上
    out["replay"], out["near_silent"]                    # bool [B]
    out["owner_ivs"], out["interf_ivs"]                  # 每段一个区间 list，样本下标，左闭右开
it.build_id                                              # 写进训练运行记录
```

- 第 k 批只由 (seed, k, batch_size) 决定，和 num_workers 无关。从第 k 批续训时传 `start_batch=k`；如果换了 batch_size，批的内容会变
- `clean` 只留直达声加前 100 ms 早期混响（`rooms.early_rir`）。均衡同时作用在 `clean` 上；削波只作用在 `noisy` 上
- `owner_ivs` 用 exp013 `dsp.intervals` 算（和评测真值同一个正本）。算的时机：混响、均衡、电平调整之后，削波之前。区间换成 10 ms 掩码的那一步由 trainer 做
- 第一批配方里没有能听清的旁人说话（D89），所以 `interf_ivs` 一律为空
- `render="cpu"` 走旧路，在 worker 里直接用 CPU 渲染（对拍和没有显卡时冒烟用）。`DevSet(shards, recipe, n=2000)[i]` 是 CPU 渲染，结果固定

## 五、混音旋钮（数据消融时一次只动一个）

旋钮的默认值和出处见 `recipe.json`。所有旋钮：目标场景 · 回放 · 信噪比 · 电平 · 随机均衡 · 削波 · 噪声段数 · 近静音段 · 混响。
**目标场景**默认为空，等缺口表（014 第 1 步）出来再定。
**近静音段**默认关：只在 trainer 配了防误压损失时才开（Taherian 2023 p.4）。
官方 SEtrain 混音代码里 `alpha = 10**(-snr*1.5/20)`，所以它的实际信噪比是名义值的 1.5 倍（读源码得知）。本配方按真实信噪比取值。

## 六、云上步骤（RunPod H100，网络盘 `/workspace`）

```bash
bash cloud_steps.sh setup      # uv 建 .venv，装 requirements.txt
bash cloud_steps.sh test       # 全部单元测试；有 CUDA 时 GPU 对拍在 GPU 上跑
bash cloud_steps.sh head       # 只核服务器上的文件大小，不下载
bash cloud_steps.sh fetch      # 第一批 A1–A5 + DEMAND：下载约 18.1 GB，解压峰值约 37 GB；开下之前先查盘
bash cloud_steps.sh split      # 先列出目录清单（QUT 和 TUM 包内的结构还没核），再切
bash cloud_steps.sh shard
bash cloud_steps.sh register   # 交集检查，登记训练 build
bash cloud_steps.sh rebuild    # 重建 exp013，逐文件核 4,540 个哈希
```

`fetch` 要等大总管放行再跑。
下面这些文件要提交进仓：`fetched.json`、`splits/`、`builds/<id>/build.json` 和 `rebuild/*.json`。
跑完以后，把它们从云上拷回来提交。

## 七、已知局限

- QUT-NOISE 和 TUM 风噪：压缩包里的目录结构还没核，`split_rules.json` 里的通配符要等云上 `--inventory` 列出清单后再确认。另外 QUT 的服务器没有声明支持断点续传；脚本遇到不支持时会从头重下
- OpenSLR 28：包里的点声源噪声取自 MUSAN，里面各个来源的许可没有逐个核，所以第一批不用
- 测试状态：本机共 72 条，全过（torch 用 CPU 后端）。云上 GPU 对拍还没跑
- 依赖：测试要用 evalcmp 在分支 `feat/eval-paired-compare` 上加进 exp013 `rooms.py` 的 `early_rir`。那个分支合进 main 之前，在本分支上跑会红
