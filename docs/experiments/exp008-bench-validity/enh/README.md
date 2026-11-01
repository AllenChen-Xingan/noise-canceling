# exp008 · 第二轮系统阶梯：增强脚本约定

> 2026-10-31 · 人定：Hush 预编译 dylib 允许（只作对照、不进产品）；torch 现在装。名额由大总管 noise-canceling-c4 发给 benchsynth。
> 第一轮系统在 `../rank.py` 里直接跑；第二轮系统运行环境各不相同（torch / Rust / 别人的 dylib），所以**先各自把增强音频写成文件，再由 `rank.py --systems <id> --tag r2` 读文件算同一套指标**。

## 每个系统一个脚本 `enh/<id>.py`（Rust 的再加一个小 bin，源码放 `enh/rust/<name>/`）

输入：`python enh/<id>.py <build> [--jobs N] [--limit N]`
- 读 `../builds/<build>/truth.jsonl`，每行 `files.mix.path` 是相对 `<MAIN>/_tmp/exp008/<build>/` 的 16 kHz 单声道 float32 wav（8 s）
- 写 `<MAIN>/_tmp/exp008/<build>-enh/<id>/<同一相对路径>`：**16 kHz 单声道 float32，与输入严格等长，已补偿算法延迟**（样本对齐，SI-SDR 依赖这一点）
- 已存在的输出跳过（可断点续跑）
- 每段新建模型 / 状态从零开始（与第一轮一致）
- 采样率不是 16 k 的模型：外面用 `scipy.signal.resample_poly` 升到模型采样率、跑完降回 16 k
- 跑完写一份记录 `../builds/<build>/enh/<id>.json`（进仓，小文件）：`{"id","family","causal","sr_model","weights":[{"path","sha256","bytes","license","license_source"}],"code_source","delay_comp_samples","delay_check":{...},"runtime":"python 版本 / torch 版本 / 二进制 sha256","n_written","seconds","notes"}`

`<MAIN>` = `/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling`

## 延迟核对（必须做，写进记录的 `delay_check`）

对前 6 段：输出与 `owner` 轨（同目录 `_owner.wav`）在 ±2000 样本内做互相关，峰值滞后应为 0（±1）。不为 0 就补偿后再核。

## 运行环境

| 用途 | 位置 |
|---|---|
| numpy / onnxruntime / soundfile / scipy | `<MAIN>/_tmp/exp002-venv/bin/python` |
| torch 2.15 / torchaudio 2.11 | `<MAIN>/_tmp/exp008-torch-venv/bin/python`（缺包用 `VIRTUAL_ENV=<该目录> uv pip install …` 装进去，不装进别的环境） |
| 模型权重下载 | `~/vdn-data/models/<名>/`（仓外）；HF 缓存设 `HF_HOME=~/vdn-data/hf` |
| cargo 产物 | `CARGO_TARGET_DIR=<MAIN>/_tmp/exp008-cargo`（不要在仓内产生 target/） |

## 边界（违反任何一条就停下报告）

- 不做 git 写操作（hook 自动提交）；只写 `enh/` 下自己的脚本、`../builds/<build>/enh/<id>.json`，以及上面列的仓外目录
- 单个下载 > 500 MB 先停下报告；不签任何协议；不上传任何东西
- `resources/` 只读（别人的代码：调用，不复制进仓）
- 同一处失败 2–3 次就停，写清卡在哪
- 只作对照、不进产品；非因果模型在记录里 `causal: false`
