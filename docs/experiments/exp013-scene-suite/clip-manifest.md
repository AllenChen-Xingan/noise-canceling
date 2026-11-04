# exp013 · 片段清单格式（草稿 v0 · 2026-11-03 · 待 scorecard 对齐）

> 用途：场景测试集每段一条记录，`vdn-eval` 与成绩单按它读音频、按场景和声音分组出分。
> 落点：每个 build 一份 `docs/experiments/exp013-scene-suite/builds/<build-id>/clips.jsonl`（每行一段）+ 一份 `manifest.json`（整批）。
> 正本：定稿后字段的正本是生成器里的 Python 定义（`manifest.py`）；本页是说明，测试比对两边字段名，对不上即红。若 scorecard 要在 Rust 里定类型、由它导出 schema，则 Rust 类型是正本，本线改为读那份 schema 校验（CLAUDE.md 第五节 5 条），请 scorecard 定。
> 单位：时间用 16 kHz 样本号（整数，左闭右开），另给秒（只供人看）；电平用 dBFS；距离用米。

## 一、每段一行（`clips.jsonl`）

| 字段 | 类型 | 含义 |
|---|---|---|
| `id` | 字符串 | 片段 id，形如 `cafe-c03`（地点键 + 档 + 序号），build 内唯一 |
| `scene` | 字符串 | 021 `catalog.md` 主表的地点键 |
| `tier` | `common` / `stress` | 常见档 / 压力档 |
| `device` | 字符串 | 022 `devices.md` 的设备键：`mba-builtin` 或 `external-mic` |
| `mic_mode` | 字符串 | 022 第二节：`standard`（`external-mic` 填 `unknown`）|
| `pair` | 字符串 | 同一份素材的另一设备版本的 `id`（`mba-builtin` ↔ `external-mic` 配对）|
| `lang` | `en` / `zh` | 主说话人语种 |
| `polarity` | `suppress` / `preserve_all` | `suppress` = 只留主说话人；`preserve_all` = 近处几人都留（只有 `meeting-room`，待人定）|
| `dur_samples` · `sr` | 整数 | 段长、采样率（16000）|
| `audio` | 对象 | 各轨相对主目录的路径：`mix` · `owner`（混响 + 设备后，**评测用干净参考**）· `owner_dry` · `stems`（键 = 声音实例 id）· `floor`（设备底噪）|
| `has_clean_ref` | 布尔 | 有无干净参考（本集恒为真）|
| `additive` | 布尔 | 分轨相加是否等于 mix（削过波的段为假，此时 `stems` 是削波前的）|
| `targets` | 数组 | 要留的说话人：`[{"spk": "<说话人 id>", "dist_m": 0.5, "azimuth_deg": 0, "level_dbfs": -27.3, "iv": [[起, 止), …]}]`。`suppress` 段只有 1 个 |
| `sounds` | 数组 | 每个干扰声音实例一条，见第二节 |
| `nominal` | 对象 | 名义值：`snr_db` · `sir_db`（无旁人时 `null`）· `t60_s` · `owner_dist_m` · `owner_level_dbfs` |
| `measured` | 对象 | 实测值（设备链之后、独立算法量）：`snr_db` · `sir_db` · `t60_s`（由所用 RIR 算）· `drr_db` · `overlap_ratio`（主说话人与旁人语音同时出现的比例）|
| `owner_iv` · `interf_iv` | 区间数组 | 016 §4 的样本级区间：从**混响后**轨算，帧能量 > 全段最大帧 −35 dB，两端各外扩 2 帧（40 ms）。`interf_iv` = 所有旁人语音的并集 |
| `chain` | 对象 | 设备链各步实际抽到的参数（频响曲线是否平坦、倾斜量、峰值滤波、底噪比、方位衰减、是否削波…）|
| `seed` | 整数 | 本段种子；`build.py --only <id>` 用它单段重现 |
| `hash` | 对象 | 每条轨的**样本内容** sha256（float32 小端字节，不是 wav 文件哈希，016 S1）+ `wav_sha256`（文件哈希，供 `vdn-eval` 现有的哈希核对用）|

## 二、声音实例（`sounds[]` 每项）

| 字段 | 含义 |
|---|---|
| `inst` | 实例 id，段内唯一，如 `bed0`、`talker1`、`dog0`；同时是 `audio.stems` 的键 |
| `cls` | 声音类别键（正本 `sounds.json`），如 `bed-cafe` · `coffee-machine` · `talker` · `tv-voice` · `dog-bark` |
| `group` | `bed`（底噪）/ `pattern`（有规律）/ `event`（突发）/ `speech`（旁人语音、电视、广播）|
| `is_speech` | 布尔；为真的算进 SIR 与 `interf_iv`，为假的算进 SNR |
| `iv` | 活动区间（样本，规则同 `owner_iv`）|
| `level_dbfs` | 活动段电平（设备链之后）|
| `rel_db` | 相对主说话人的电平差（活动段对活动段）|
| `dist_m` · `azimuth_deg` | 声源距离、相对正前方的方位角；扩散声（底噪、babble）填 `null` |
| `src` | 素材出处：`{"kind": "librispeech", "spk": "1089", "utts": [...]}` / `{"kind": "demand", "env": "SCAFE", "ch": 3, "start": 3840000, "len": 192000}` / `{"kind": "esc50", "file": "1-100038-A-14.wav"}` / `{"kind": "synth", "fn": "coffee_machine", "seed": 123}` |
| `license` | 许可简称（`CC-BY-4.0` · `CC-BY-NC-3.0` · `own` …），用途恒为「只评测」|

## 三、整批（`manifest.json`）

`build_id` · 生成时间 · 生成脚本与参数文件（`scenes.json` · `sounds.json` · `device_chain.json`）的 sha256 · 素材索引 sha256 · 段数（按地点 × 档 × 设备）· 种子基 · 交集检查结果（对 exp006 `44df`：说话人 / DEMAND 时段 / 合成函数名，三项都应为 0）· 许可汇总 · 效度级（本批 D0）。

## 四、要和 scorecard 商量的三件事

1. **字段的正本放哪**：建议 Rust 那边定类型、导出 schema，本线校验；如果你们不打算在 Rust 里读逐段字段，那就以本线 `manifest.py` 为正本
2. **`vdn-eval` 怎么读**：现在 `real` 只认 exp005 / exp003。建议加 `{"kind": "scene", "clips": "<clips.jsonl 仓内路径>", "scene": "<地点键>", "tier": "common", "device": "mba-builtin"}`，一条任务展开成多段；干净参考用 `audio.owner`
3. **按声音拆分**：成绩单要按 `sounds[].cls` 拆，需要逐段分数长表（段 × 系统 × verifier），再和本清单按 `id` 连接。需要我在清单里加别的字段（比如每种声音的「说话时 / 停顿时」区间分开给）就说
