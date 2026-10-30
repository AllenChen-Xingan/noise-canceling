# 现状页 · abtest（批次 20261030-b2）

状态：**进行中（接替会话，worktree `noise_canceling-exp003`，分支 exp/003-offline-ab）**

人新定 D28：吃字率改用本机 Qwen3-ASR-Plus 1.7B（42model 服务），不用 whisper。范围追加：VoxLock 作第三候选（M5 单线程逐帧计时 + 加进 AB 页），接入说明在主目录 `docs/research/005_pse/gap.md` 末节。

## 手上
exp003：DPDFNet-2 对 GTCRN 的离线 AB 听测（交代 `_tmp/20261030-b2/abtest.md`）

## 做到哪（2026-10-30 更新）
- **两模型轮已完成**：12 段素材 · 逐帧增强（对齐滞后 12/12 为 0、三版 −26 LUFS）· 页面 `_tmp/20261030-exp003/index.html`（静音无头冒烟通过）· 客观分数 `docs/experiments/exp003-offline-ab/scores.json`。DNSMOS OVRL DPDFNet-2 2.75 对 GTCRN 2.44（9/12 段赢）；删除率 4.6% 对 4.3%（原声 3.2%）；Qwen3-ASR 两次转写 48/48 逐字一致。吃字逐词明细 `eaten_words.md`：两者各 7 个「原声没删、降噪后删了」的词
- **VoxLock 轮进行中**（名额已批）：正在跑 `bench_voxlock.py`（int8/fp32 × 块长 8/10/20/40 ms 单线程计时）
- 页面已按 rubric 加：开会能用/勉强/不能用、缺陷标签、播放/切换计数、切换 5 ms 淡出淡入、两问顺序随机

## 下一步
1. 计时跑完 → make_clips（加注册样本，核各段音频哈希不变）→ enhance（加 voxlock_given / voxlock_live）→ build_page → score --asr（新两列）→ eaten_words.py。预计计时后约 10 分钟
2. 归还名额，报大总管：页面路径 + VoxLock 每块 p99.9/max 占周期 + 端到端延迟推算
3. 写 README（素材许可哈希 · 方法 · 分数表 · 纯噪音段 c09/c10 上 VoxLock 对比 · 吃字一节 · 局限）
4. 三视角自审，修完报「abtest 可以收尾」

## 已写文件（都在 `docs/experiments/exp003-offline-ab/`，已被 hook 自动提交）
- `make_clips.py`：12 段素材（c01–c03 urgent 赌场 5/0/−5 dB 截 12 s；c04–c10 LibriSpeech 合成：单干扰者 5/0 dB + SCAFE、babble 3 人 0 dB/6 人 −5 dB/4 人 10 dB、DKITCHEN 杯碟 0 dB、钢琴 5 dB；c11–c12 中文）。目标语音前留 1 s、后留 0.5 s 纯噪音。种子 20261030。产出 `_tmp/20261030-exp003/src/` + `clips.json`
- `enhance.py`：逐帧流式 DPDFNet-2（vorbis 320/160，元数据初始化 state）与 GTCRN simple（周期 sqrt-Hann 512/256，cache 全零）；开头补 n_fft 零、尾部冲刷；按内部延迟平移并互相关核对；BS.1770 统一 −26 LUFS；随机文件名。产出 `raw/`、`audio/`、`files.json`、`align.json`
- `score.py`：DNSMOS P.835（对归一后文件）· ESTOI · SI-SDR · 非语音段 noise_att_db / resid_rel_db（按干净参考能量 VAD）· `--whisper` 时删除/替换/插入（以 whisper 转写干净参考为参照，英文按词、中文按字，自写编辑距离）。产出 `scores.json`
- `ab_page.html`：盲听页模板（A/B/原声同步切换、快捷键、两个都听过才能答、三问 + 备注、localStorage 存进度、揭盲汇总、导出 JSON、自录片段入口、深浅色）
- `build_page.py`：两层随机（生成时 x/y 槽位 + 页面运行时 A/B 平衡洗牌）；两道陷阱题（同一文件 → 应答「差不多」；低通 1 kHz + 5 dB 白噪劣化版 → 应选原样）；产出 `_tmp/20261030-exp003/index.html`、`manifest.js`、`unblind-key.json`

## 仓外数据（`~/vdn-data/`，D22）
- `librispeech/test-clean.tar.gz`（346663984 B，已解压到 `librispeech/LibriSpeech/`）· openslr.org/resources/12 · CC BY 4.0
- `demand/PCAFETER_16k.zip`（md5 已核对一致，已解压 `PCAFETER/`）· `DKITCHEN_16k.zip`、`SCAFE_48k.zip` 下载中 · Zenodo 1227121 标 **CC BY 4.0**（交代写 CC BY-SA；按 D26 仍只评测）
- `music/Scott_Joplin_-_Maple_Leaf_Rag.ogg`（3673778 B）· Commons · Public domain
- `zh/zh_braille_commusic.ogg`（Commons「Zh-盲文.ogg」· Commusic · CC BY-SA 4.0）· `zh/zh_popobing_landxia.ogg`（「Zh-婆婆冰.ogg」· Landxia · CC BY-SA 4.0）· `zh/zh_bubbling_well_road_timshi.ogg`（弃用，底噪）· `zh/cand/`（试过的候选：桂林市 GFDL、深圳地铁、欧洲，均未用）。中文两段只评测不进训练（D26，大总管确认）
- `models/ggml-large-v3-turbo.bin` 下载中（大总管批准 A）

## 待人拍板
（暂无）

## 流水
- 2026-10-30 读交代、common、002 decision、exp001、exp002、004 AB 行；向大总管报到
- 2026-10-30 盘点：现成带噪 6 段；whisper-cli 已装但无正式模型；`_tmp/exp002-venv` 可复用
- 2026-10-30 后台下载 LibriSpeech + DEMAND 三套；Commons 下载钢琴曲（PD）与中文两段
- 2026-10-30 问大总管 whisper 模型 → 批 A（下载 large-v3-turbo，核 sha1，跑前申请名额）
- 2026-10-30 写 enhance.py / make_clips.py / score.py / ab_page.html / build_page.py（未运行）
- 2026-10-30 PCAFETER md5 核对一致；发现 TimShi 中文录音有底噪，换婆婆冰（Landxia），make_clips.py 尚未改
- 2026-10-30 应大总管要求停在安全点，写交接
- 2026-10-30 【接替会话】报到；核 md5：SCAFE_48k 1a4ee387…、DKITCHEN_16k 7ffbf52d… 均一致，已解压到 `~/vdn-data/demand/{SCAFE,DKITCHEN}/`
- 2026-10-30 新增 `_paths.py`（worktree 里按 git 公共目录找回主目录 resources/）；四个脚本改用它；c11 换婆婆冰（Landxia）
- 2026-10-30 score.py：`--whisper` → `--asr`（Qwen3-ASR，同一文件转两次核逐字相同，插入另记 ins_rate=脑补率）；whisper 模型人定已删，不再用
- 2026-10-30 申请名额获批 → make_clips（12 段）· enhance（滞后 12/12 为 0，三版 −26 LUFS）· build_page · score --asr（Qwen3-ASR 1.7B，48/48 两次逐字相同）；DNSMOS OVRL DPDFNet-2 2.75 对 GTCRN 2.44（9/12 段），删除率 4.6% 对 4.3%（原声 3.2%）；名额已归还
- 2026-10-30 页面改为通用 N 候选（A/B/C… + 原声，拉丁方轮换），按 rubric 加「开会能用/勉强/不能用」、缺陷标签、plays/switches/started_at；静音无头冒烟（Playwright + 系统 Chrome，play 打桩 + --mute-audio）通过
- 2026-10-30 修 score.py：numpy 整数写 JSON 报错；加转写缓存 `_tmp/20261030-exp003/asr_cache.json`；中文段英文按词计（c12 ASR 转出了英文干扰者）
- 2026-10-30 VoxLock 权重下到 `~/vdn-data/models/voxlock/`（int8+fp32，sha256 记在 README）；enhance 加 given/live 两模式，make_clips 加注册样本，写 bench_voxlock.py；已申请下一轮名额
- 2026-10-30 VoxLock 轮：计时（M5 单线程 int8 每 8 ms 帧 p50 8.4 ms，跟不上实时；40 ms 块 p99.9 31 ms）· 注册样本 · enhance 四候选（滞后 60/60 为 0）· 页面 4 候选 · 打分（72/72 转写一致）；名额已归还，已报大总管与 rubric（scores.json sha256 e84730fa…）
- 2026-10-30 页面按 rubric：切换 5 ms 淡出淡入、两问顺序平衡随机、吃字并进每行、标签折叠；写 README、eaten_words.py/.md；派三视角自审（怀疑派 · 实现派 · 产品派）进行中
