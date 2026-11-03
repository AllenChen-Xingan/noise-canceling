# 人耳偏好数据集 · 字段说明

生成器：`build_dataset.py`（只读仓里的答卷与对照表，确定性输出）。改字段 = 改生成器 + 本文，走提交链。
三个文件：`trials.jsonl`（每屏一行）· `pairs.jsonl`（每个两两比较一行）· `summary.json`（计数与会话有效性）。

## trials.jsonl · 每屏一条

| 字段 | 类型 | 说明 |
|---|---|---|
| `rec_id` | 字符串 | `<实验>/<build>/<题序两位>-<片段>`，全库唯一；其他文件都用它指回来 |
| `exp` · `build` | 字符串 | 实验号；页面 build id（对照表在 `docs/experiments/<实验>/builds/<build>/`） |
| `trial_order` | 整数 | 人看到的题序。exp005 导出里没有题序，这里是答卷 JSON 的键序（见 `trial_order_note`） |
| `kind` | `real` · `trap` | 正式题 / 检验题 |
| `trap` | 对象或 null | `{type, pass, expect}`：检验题类型、是否通过、期望答法 |
| `clip` | 字符串 | 片段 id（各实验 `clips.json` 或 exp006 `files.json`） |
| `scene` | 对象 | `fine`（细场景，与 exp006 `recipes.json` 7 大类同名：quiet · hvac · babble · neighbor · clatter · traffic · music）· `user`（用户层：quiet · cafe · commute · music）· `desc`（一句话）· `lang`；exp006 另有 `device` · `snr_db`。**标签按素材生成参数定，不是人的感知**（exp005 c05 按参数是食堂底噪，人说「本来周围就是比较安静」） |
| `systems` | 列表 | 本屏的候选（版本名照各实验对照表：exp003 `dpdfnet2` · `gtcrn` · `fastenhancer_m`；exp005 `raw` · `personal` · `aggressive`；exp006 `base` · `route` · `polish` · `both`；检验题另有 `degraded`；同一文件出现在多个槽位时按位置改名：全同题 `identical#1/#2/…`、exp005 检验题 `personal#1/#2`——**避免与「分不出」的 `same` 撞名**） |
| `positions` | 对象 | 版本 → 字母位置（A / B / C / D），给位置偏好分析 |
| `qorder` | 列表或 null | 「最清楚 / 最自然」的上下顺序（exp003 / exp006） |
| `questions` | 列表 | 这一页问了哪几问（不同实验问法不同，见下「维度」） |
| `answers` | 对象 | 揭盲后的答案：`clearer` · `natural` · `quietest` · `pick`（best-of，值 = 版本名 / `same` 分不出 / `reference` 选了原声 / null 没问）· `eaten`（被勾吃字的版本列表）· `eaten_none`（勾了「都没有」）· `usable`（版本 → `ok` / `marginal` / `no`）· `tags`（版本 → 缺陷标签列表，词表见 `specs/rubric/denoise.md` 附录） |
| `note` | 字符串 | **人原话备注照录**，不改一个字 |
| `listen_s` · `plays` · `switches` | | 每个版本累计听了几秒、播放几次、切换次数（`ref` = 原声）；exp005 导出没有 |
| `started_at` · `answered_at` | ISO 时间 | 本屏开始与最后一次改答案的时间；exp005 导出没有 |
| `session` | 对象 | `id` · `validity`（`valid` 有效 · `discounted` 打折 · `unverified` 检验题没答到 · `void` 作废，按 `specs/rubric/denoise.md` 4.2）· `why` · `started` · `finished` |
| `excerpt_s` · `lock_t` | | exp005：人听的节选起止秒、认人锁定时刻 |
| `audio` | 对象 | `noisy_sha256` · `clean_sha256` · `out_sha256`（版本 → 被听那份文件的 sha256；**null = 没记**）· `gen`（生成脚本、种子、参数）· `note`（如「哈希是 25 s 节选」）· `out_sha256_missing`（为什么没有哈希）。音频不进仓 |
| `unblindable` | bool | 旧 build 68ab 揭盲表丢失的屏：只留原始字母答案 `raw_answer_letters`，不进配对 |
| `raw_answer_letters` | 对象 | 揭盲前的原始字母答案：旧 build 无法揭盲的屏，以及 exp003 检验题 |
| `qid` | 字符串 | exp005 页面的随机题号（对照表 `key` 的键） |
| `data_build` | 字符串 | exp006：数据 build（页面 build 在 `build`） |
| `trial_order_note` | 字符串 | exp005：题序的来历说明 |
| `source` | 对象 | 答卷文件与对照表文件的仓内路径 |

## summary.json

`n_trials` · `n_pairs` · `sessions`（每个会话的有效性、屏数、备注数、best-of 选择落在哪个字母位置）· `pairs_by_dim`（每个维度的配对数、分出胜负 / 平、`identical_audio`、`invalid`、`consistency` 计数）· `dims`（维度中文名）。

## pairs.jsonl · 每个两两比较一行

| 字段 | 说明 |
|---|---|
| `rec_id` · `exp` · `build` · `clip` · `scene_fine` · `scene_user` | 指回 trials |
| `dim` | 维度（下表） |
| `a` · `b` | 两个版本（字母序） |
| `winner` | `a` 或 `b` 的版本名，或 `tie` |
| `rule` | 怎么推出来的：`best_of`（被选「最 X」者对同屏其余各 1 胜，**没被选的两个之间不出配对**，rubric 4.3）· `same`（答「分不出」→ 全部两两平）· `usable_grade`（开会评级比较，同级记平）· `eaten_mark`（被勾 vs 没勾；`_conflict` = 同屏又勾了「都没有」）· `tag_presence`（标了缺陷 vs 没标，只在有差别时出）· 后缀 `+identical_audio` 见下 |
| `strength` | `normal` · `weak`（分不出、标签可选、答案自相矛盾）· **`invalid`**（两份相同文件被分出胜负）· **`consistency`**（两份相同文件判平：只作一致性检验） |
| `pos_a` · `pos_b` | 两者的字母位置 |
| `identical_audio` | 两份音频 sha256 相同（逐字节同一文件） |
| `session_validity` | 所属会话的有效性 |

### 维度

| `dim` | 中文 | 来自哪一问 | 哪些实验问了 |
|---|---|---|---|
| `clearer` | 清楚 | 「哪个最清楚」 | exp003 · exp006 |
| `natural` | 自然 | 「哪个最自然」 | exp003 · exp005 · exp006 |
| `meeting` | 开会可用 | exp003 / exp006：每个候选「能用 / 勉强 / 不能用」；exp005：「真开会选哪一份」 | 全部 |
| `eaten` | 吃字（没被勾的胜） | 「哪几个吃字」（exp005 问的是「吞字、变轻或句首被切」） | 全部 |
| `quietest` | 最安静（旁人声音最小） | 「旁边的人说话时哪一份他的声音最小」 | exp005 |
| `defects` | 缺陷标签 | 可选标签 | exp003 · exp006 |

## 硬规则（用数据的人必读）

1. **同文件配对 = 无效票，不进定标、不进虚拟听众训练**：`identical_audio = true` 的对 `rule` 带后缀 `+identical_audio`；分出胜负的 `strength = invalid`（现有 1 对：exp005 c08 开会选「只有 GTCRN」而不选逐字节相同的「激进」）；判平的 `strength = consistency`（现有 4 对），是**免费的一致性检验**，可用来估人的随机性，但也不当偏好用
2. **`weak` 只作旁证**；定标（012 §2.1 L1）只用 `normal`，且会话 `validity ∈ {valid, discounted}`；`discounted` 会话的 best-of 配对按 rubric 4.2「只认显著」处理
3. **`unverified` / `void` 会话不进定标**（旧 build 68ab 的 2 屏只作记录）
4. **检验题不出配对**
5. 配对不独立：同一屏出的几对要**按屏聚类**做 bootstrap（012 §2.1）
6. **不同实验的问法不同**（exp005 的「吃字」包括「变轻、句首被切」；exp005「开会」是单选，exp003 是逐个评级），合并统计时按 `exp` 分层

## 每条记录喂给谁

| 维度 / 字段 | 定标哪个 verifier（id 照 `src/eval/src/verifiers.rs`） | 虚拟听众（D48）怎么用 |
|---|---|---|
| `clearer` | V-EAT-ASR · V-SPLVL · V-ONSET（片段内排序，012 L1 / L2） | 成对比较模型的「清楚」头 |
| `natural` | V-MUSIC · V-VACUUM · V-TIMBRE · V-BGSTAB（真实录音判「不知道」）· V-DNSMOS | 「自然」头；主训练信号（exp005 里方向最一致：保守版 7/8） |
| `meeting` | 综合项，**不定标单个 verifier**；用来检验 rubric 总分与人的整体选择是否同向 | 总偏好头（RLHF 式奖励模型的主标签） |
| `eaten` | V-EAT-ASR（exp003 已知人勾 ≫ ASR 删：rubric 第 6 节「ASR 漏报」）· V-ONSET · V-SIM-GATE（字头漏开） | 吃字惩罚头（重罚，意向书「从不吃字」） |
| `quietest` | V-INTF · V-NR | 「压旁人」头 |
| `defects` / `tags` | `metallic` → V-MUSIC（V-ROBOT 未实现）· `vacuum` → V-VACUUM · `muffled` → V-TIMBRE · `pumping_bg` → V-BGSTAB · `clipped_onsets` → V-ONSET · `pumping_voice` → 无（V-PUMPV 未实现） | 缺陷检测器的弱标签 |
| `note` | 不进统计；进 `specs/rubric/preferences.md` 的偏好条目与下一轮要问的问题 | 不用（文本备注只给人读） |
| `scene` | 按场景分层定标（同一 verifier 在不同场景可能置信度不同） | 场景条件化（D40：场景内学偏好） |
| `positions` | 位置偏好校正（exp003 f8c3 C 位 6/9） | 训练时加位置特征或做位置去偏 |

**接入 vdn-eval**：evalimpl 的 `vdn-eval calib --answers` 读 exp003 格式答卷 + `scores.json`（`per_clip[clip][model]`）。本数据集可以直接按 `rec_id` 指回原答卷；要喂 calib 的话，exp005 / exp006 需先转成 exp003 格式（`trials[].kind/clip/slots/by_model`）——转换器未写，列为缺口。
