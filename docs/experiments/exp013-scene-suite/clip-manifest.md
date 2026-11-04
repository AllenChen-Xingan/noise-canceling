# exp013 · 片段清单格式（v1.2 · 2026-11-03 · 已与 scorecard 对齐）

> v0 → v1（scorecard 回复）：正本改为 Rust 类型导出的 schema；`audio` 加 `noise` / `interf` 两条汇总轨；`hash.wav_sha256` 改为逐轨对象。
> v1.1 → v1.2（第三版 build，实测 RIR）：`targets[].src` 与 `sounds[].src` 里多了 `rir`（`{kind: "rir", dataset, room, file, ch, t60_s, drr_meas_db, direct_gain_db}`）；`chain.room`（`room_id` · `dataset` · `paired` · `room_t60_s` · `bed_cls` 等）；`chain.room_dims_m` 去掉。都在自由对象里，不改 schema。
> v1 → v1.1：`hash` 定为 `{samples, wav_sha256}` 两个逐轨对象（同 scorecard schema）；另加 4 个字段，已请 scorecard 加进 Rust 类型：`targets[].src` · `measured.snr_all_db` · `measured.owner_level_dbfs` · `sounds[].direction_db`。

> 用途：场景测试集每段一条记录，`vdn-eval` 与成绩单按它读音频、按场景和声音分组出分。
> 落点：每个 build 一份 `docs/experiments/exp013-scene-suite/builds/<build-id>/clips.jsonl`（每行一段）+ 一份 `manifest.json`（整批）。
> 正本：`vdn-eval` 里的 Rust 类型（`SceneClip` 等，scorecard 写），由 `vdn-schema` 导出 `specs/schema/eval-scene-clip.schema.json`。本线 `manifest.py` 读这份 schema 校验每行，不另立正本；本页只是说明（CLAUDE.md 第五节 5 条）。
> 例外：声音类别键 `cls` 的正本是本线 `sounds.json`，Rust 只当字符串读；场景键、设备键 Rust 有枚举，由 scorecard 的测试直接比对 021 / 022 原表。
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
| `audio` | 对象 | 各轨相对主目录的路径：`mix` · `owner`（混响 + 设备后，**评测用干净参考**）· `owner_dry`（主说话人干轨：无房间、无设备链，按直达声到达时刻后移、与 `owner` 对齐，电平与主说话人链后相同；Air 麦版本另有最小相位 FIR 的几个样本延迟未补）· `stems`（键 = 声音实例 id）· `floor`（设备底噪）· `noise`（全部非语音分轨 + `floor` 之和）· `interf`（全部旁人语音分轨之和）。`noise` / `interf` 都取削波前，`additive` 为假时也给 |
| `has_clean_ref` | 布尔 | 有无干净参考（本集恒为真）|
| `additive` | 布尔 | 分轨相加是否等于 mix（削过波的段为假，此时 `stems` 是削波前的）|
| `targets` | 数组 | 要留的说话人：`[{"spk": "<说话人 id>", "dist_m": 0.5, "azimuth_deg": 0, "level_dbfs": -27.3, "iv": [[起, 止), …]}]`。外加 `src`（素材出处，同 `sounds[].src`，防泄漏检查用）。`suppress` 段恰好 1 个；`preserve_all` 段 2–3 个，`owner` = 全部之和 |
| `sounds` | 数组 | 每个干扰声音实例一条，见第二节 |
| `nominal` | 对象 | 名义值：`snr_db`（主说话人对**稳态干扰**：底噪 + 有规律 + babble + 设备底噪；突发声不算，按各自 `rel_db` 单设）· `sir_db`（对听得清的旁人语音之和；无旁人时 `null`）· `t60_s` · `owner_dist_m` · `owner_level_dbfs`。生成时用 gated 电平（20 ms 帧 > 最大帧 −35 dB）|
| `measured` | 对象 | 实测值（设备链之后，主说话人电平改用 ITU-T P.56，与生成时不同算法）：`snr_db` · `snr_all_db`（突发声也算进去）· `sir_db` · `owner_level_dbfs` · `t60_s`（Schroeder 法，在主说话人 RIR 去掉直达窗后量）· `drr_db` · `overlap_ratio`（主说话人活动区间里有旁人语音的比例）。与名义值 95% 的段差 ≤ 1 dB、全部 ≤ 1.5 dB；按条件分档请用 `nominal` |
| `owner_iv` · `interf_iv` | 区间数组 | 016 §4 的样本级区间：从**混响后**轨算，帧能量 > 全段最大帧 −35 dB，两端各外扩 2 帧（40 ms）。`interf_iv` = 所有旁人语音的并集 |
| `chain` | 对象 | 设备链各步实际抽到的参数：`flat` · `tilt_db_per_oct` · `peak_*` · `floor_ratio_db` · `floor_lowmid_excess_db` · `strength` · `reverb_stress` · `vpio_db` · `clip_drive_db`；另有 `floor_clamped`（底噪比名义 SNR 允许的还响、被压低）· `headroom_db`（整段峰值超 −1 dBFS 时整段调低的量，0 = 没调）· `room_dims_m` · `owner_dry_delay_samples`（owner_dry 后移了多少样本）|
| `seed` | 整数 | 本段种子；`build.py --only <id>` 用它单段重现 |
| `hash` | 对象 | `{"samples": {…}, "wav_sha256": {…}}`，两个都是逐轨对象，键同 `audio`（`stems` 下按实例 id）。`samples` = 样本内容 sha256（float32 小端字节，016 S1）；`wav_sha256` = 文件哈希，`vdn-eval` 逐条核对（wav 不带时间戳块，可复现）|

## 二、声音实例（`sounds[]` 每项）

| 字段 | 含义 |
|---|---|
| `inst` | 实例 id，段内唯一，如 `bed0`、`talker1`、`dog0`；同时是 `audio.stems` 的键 |
| `cls` | 声音类别键（正本 `sounds.json`），如 `bed-cafe` · `coffee-machine` · `talker` · `tv-voice` · `dog-bark` |
| `group` | `bed`（底噪）/ `pattern`（有规律）/ `event`（突发）/ `speech`（旁人语音、电视、广播）|
| `is_speech` | 布尔：听得清的单个人声（旁人、电视、导航、广播人声）为真，算进 SIR、`interf_iv`、`audio.interf`；其余（含 babble、广播提示音）为假，算进 `audio.noise` |
| `iv` | 活动区间（样本，规则同 `owner_iv`）|
| `level_dbfs` | 活动段电平（设备链之后）|
| `rel_db` | 相对主说话人的电平差（活动段对活动段）|
| `dist_m` · `azimuth_deg` | 声源距离、相对正前方的方位角；扩散声（底噪、babble）填 `null` |
| `src` | 素材出处：`{"kind": "librispeech", "spk": "1089", "utts": [...]}` / `{"kind": "demand", "env": "SCAFE", "ch": 3, "start": 3840000, "len": 192000}` / `{"kind": "esc50", "file": "1-100038-A-14.wav"}` / `{"kind": "synth", "fn": "coffee_machine", "seed": 123}` |
| `license` | 许可简称（`CC-BY-4.0` · `CC-BY-NC-3.0` · `own` …），用途恒为「只评测」|
| `direction_db` | 可选：设备链第 2 步对该声源直达部分的三频段方位衰减 `{low, mid, high}`（dB）；扩散声与 `external-mic` 没有 |

## 三、整批（`manifest.json`）

`build_id` · 生成时间 · 生成脚本与参数文件（`scenes.json` · `sounds.json` · `device_chain.json`）的 sha256 · 素材索引 sha256 · 段数（按地点 × 档 × 设备）· 种子基 · 交集检查结果（对 exp006 `44df`：说话人 / DEMAND 时段 / 合成函数名，三项都应为 0）· 许可汇总 · 效度级（本批 D0）。

## 四、与 scorecard 已定（2026-11-03）

1. 正本在 Rust，导出 schema 后 scorecard 告诉路径，本线照它校验
2. `RealSource` 加 `{"kind": "scene", "clips", "scene", "tier", "device"}`，一条任务展开成多段，干净参考用 `audio.owner`
3. 任务文件 `src/eval/suite/scenes.json`，30 条，graders 由 scorecard 填（本线先留空数组）
4. 「说话时 / 停顿时」不分开给，scorecard 用 `owner_iv` 自己切
