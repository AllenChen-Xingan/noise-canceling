# exp003 · 离线 AB 听测：DPDFNet-2 · GTCRN · FastEnhancer-M（附 VoxLock）

> 意图：人拍板 D01「端到端附加延迟上限先听再定」——同一批咖啡馆类录音过几个候选，人盲听后在 ≤ 45 ms（GTCRN、FastEnhancer-M）与 ≤ 75 ms（DPDFNet-2）之间选。
> 候选变迁：先是 DPDFNet-2 · GTCRN；大总管追加 VoxLock（只留注册说话人），计时后因单线程跟不上实时出局；人/大总管再定加 FastEnhancer-M。
> 真相源权重里人耳 AB 排第 2，客观分只作旁证（`.42cog/intent.md`「真相源与权重」）。**下面的客观分不替人定**。

## 给人：先看这里
- **打开**：`_tmp/20261030-exp003/builds/20261030110755-f8c3/index.html`（worktree 根下；页面用 `../../audio/` 读音频，复制时保持 `builds/<id>/` 与 `audio/` 同在一个父目录）。全程本机，不联网、不上传。戴耳机
- **每段一屏**：A / B / C 是三个候选（随机），`原声` 是带噪录音；`1`–`3` 切候选、`0` 原声、空格播放。每个候选累计听满 3 秒才能答
- **答三问**：最清楚（或「分不出」）· 最自然 · 每个候选「拿去开会：能用 / 勉强 / 不能用」（假如你是会议对方）并勾出吃字的。听不出哪个字被吞，点「显示原文」
- **14 屏**（12 段 + 2 道检验题），约 25–30 分钟；进度按版本存在浏览器里，可以中途歇
- **值得重点听的段**：c06（邻桌 3 人）、c07（邻桌 6 人 −5 dB）、c11（中文）——ASR 显示这几段降噪后丢词；c09（杯碟）、c10（钢琴）——纯噪音段，看谁压得干净
- 做完点「导出结果 JSON」交大总管，放进 `results/`

## 一句话结论（客观旁证，待人耳验证）
- **DNSMOS**：FastEnhancer-M 与 DPDFNet-2 并列领先（OVRL 2.81 / 2.75，GTCRN 2.45，原声 1.79）；逐段 FastEnhancer 胜 GTCRN 11:1
- **可懂度**：DPDFNet-2 最好（ESTOI 0.63，SI-SDR 6.6 dB）；FastEnhancer 与 GTCRN 同为 ESTOI 0.59
- **吃字（ASR）**：三者删除率都高于原声；FastEnhancer 删除、替换最多（6.1% / 30.1%），GTCRN 最少（4.3% / 20.1%）。去掉三段赌场（c01–c03，三家都烂）后：GTCRN 0.9% · DPDFNet-2 3.1% · FastEnhancer 4.8%（原声 0.4%）。**降噪后 ASR 的替换和插入也都比原声多**——降噪让「字」更像别的字。差别只有几个到十几个词，且 ASR 对 0.03 dB 级变化都会改转写，要人耳定
- **延迟**：FastEnhancer-M 与 GTCRN 同档（端到端约 38–42 ms），DPDFNet-2 约 66–70 ms
- **一句话**：多 30 ms 换来的 DPDFNet-2，客观上可懂度更好、DNSMOS 与同档的 FastEnhancer-M 打平；FastEnhancer-M 背景最干净但 ASR 吃字最多。人耳要回答的就是「DPDFNet-2 比 FastEnhancer-M / GTCRN 听着清楚多少」以及「FastEnhancer 的吃字听不听得出」

## 素材（12 段，16 kHz 单声道，每段 9.6–12 s）

| 段 | 内容 | 类别 | 有无第二个人声 |
|---|---|---|---|
| c01–c03 | 赌场环境（人声 + 机器声），SNR 5 / 0 / −5 dB；开头就说话 | babble-多人 | 有 |
| c04 / c05 | 邻座一人说话 + 咖啡馆露台底噪，5 / 0 dB | 单个干扰者 | 有 |
| c06 / c07 / c08 | 邻桌 3 / 6 / 4 人 + 食堂或露台底噪，0 / −5 / 10 dB | babble-多人 | 有 |
| **c09** | 厨房杯碟水声，0 dB | 非人声 | **无（纯噪音）** |
| **c10** | 背景钢琴曲（拉格泰姆），5 dB | 非人声 | **无（纯噪音）** |
| c11 / c12 | 中文目标 + 英文邻桌 3 人 / 邻座 1 人，0 / 5 dB | babble | 有 |

合成段（c04–c12）：目标语音前留 1.0 s、后留 0.5 s 纯噪音；SNR = 目标有声区间 RMS 对全部干扰 RMS；种子 20261030（`make_clips.py`）。逐段来源、说话人、起点见 `clips.json`。

| 来源 | 用在 | 许可 | 位置 · 哈希 |
|---|---|---|---|
| urgent2026_challenge_track2 `assets/sp0{3,4,5}*.wav`（github.com/urgent-challenge/urgent2026_challenge_track2） | c01–c03（带干净参考） | 仓库 MIT（`pyproject.toml:12`）；音频原始语料出处仓里未写（推断随仓库许可） | `clips.json` 的 `noisy_sha256` / `clean_sha256` |
| LibriSpeech test-clean（openslr.org/resources/12） | c04–c10 目标与干扰 | CC BY 4.0 | `~/vdn-data/librispeech/test-clean.tar.gz` · sha256 `39fde525…6c23` |
| DEMAND PCAFETER / SCAFE / DKITCHEN（zenodo.org/records/1227121） | 底噪 | Zenodo 标 CC BY 4.0；按 D26 只评测 | `SCAFE_48k.zip` md5 `1a4ee387f1900f4b15ba219481c76556` · `DKITCHEN_16k.zip` md5 `7ffbf52d7f4699f96927846103dc8788` · PCAFETER md5 已核；ch01 的 sha256 见 `clips.json.source_sha256` |
| Wikimedia Commons「Scott Joplin – Maple Leaf Rag」 | c10 | Public domain | `~/vdn-data/music/…ogg` · `9348ee28…8a39` |
| Commons「Zh-婆婆冰.ogg」（Landxia） | c11 | CC BY-SA 4.0，只评测（D26） | `~/vdn-data/zh/zh_popobing_landxia.ogg` · `ba5d93c3…c7ba` |
| Commons「Zh-盲文.ogg」（Commusic） | c12 | CC BY-SA 4.0，只评测 | `~/vdn-data/zh/zh_braille_commusic.ogg` · `e8e99205…4f66` |

弃用：Commons TimShi 那份中文录音全程约 −35 dB 底噪，不能当干净参考。缺口：**没有作者本人的真实咖啡馆录音**（本批不录；页面完成页有「加自录片段」入口）；中文只 2 段，c12 是「盲文点位」朗读，内容单调。

## 处理方法（`enhance.py`）
- **逐帧流式**，和实时一致：每次只喂一帧 / 一个 hop，状态回灌；每段重建状态
  - DPDFNet-2（`resources/weights/dpdfnet/dpdfnet2.onnx`）：vorbis 窗 320/160，state 按 ONNX 元数据初始化（同 exp001）；输出落后 4 帧（640 样本），按此平移
  - GTCRN simple（`resources/gtcrn/stream/onnx_models/gtcrn_simple.onnx`）：周期 sqrt-Hann 512/256，cache 全零
  - **FastEnhancer-M**：官方 release `onnx-dns-v1.0.0` 的 **wav2wav** 版（STFT/iSTFT/OLA 都在图里，没自写分帧，避免 Σw² 没归一的 16 ms 嗡声），每次喂 160 样本、缓存回灌，同 `resources/fastenhancer/scripts/test_onnx.py`；输出落后 n_fft − hop = 352 样本（`resources/fastenhancer/docs/docs/onnx.md`），按此平移
  - VoxLock 1.2.1（只为留数据，不进页面）：给定 3 s 样本 / 自动锁定两种，每次推 10 ms
- **对齐核对**：与干净参考做互相关，**12 段 × 5 个候选 = 60 个滞后都是 0**（`align.json`）；怀疑派把输出人为平移 160 样本，互相关能测出 160，说明核对方法对错位敏感
- **响度**：BS.1770（pyloudnorm），**只在主讲人说话区间测**（按干净参考的能量 VAD），同段所有版本对齐到 −26 LUFS；有版本峰值超 −1 dBFS 时同段一起再降（c09 降到 −26.56）。不用整段响度，因为停顿里压得越狠的模型整段越安静、被抬得越多，说话声会响 1–3 dB，听起来「更清楚」（自审怀疑派 P1；旧 build 68ab / cd87 用的是整段响度）
- 对比页用 16-bit 文件，文件名是随机串；同段各版本字节数、样本数相同

## 对比页设计（`ab_page.html` → `build_page.py`）
- **每个 build 一个子目录** `_tmp/20261030-exp003/builds/<build>/`，不覆盖、不删旧文件；揭盲表 `unblind-key.json`、`manifest.js`、`files.json` 同时复制进本目录 `builds/<build>/` 提交（人定）。浏览器进度按 build 分开存
- 两层随机：生成时每段候选随机排进槽位；页面打开时按**拉丁方**轮换到 A/B/C——每个候选在每个字母位置正好出现 4 次（冒烟实测）；题目顺序洗牌；「最清楚 / 最自然」上下顺序一半一半。相邻顺序（残留效应）没做平衡
- 检验题两道（借 P.808）：所有槽位同一文件（应答「分不出」）；一份低通 1 kHz + 5 dB 白噪的劣化版（最清楚 / 最自然都不该选它；答「分不出」不扣）。检验题沿用原段的描述与原声，细心的人能认出来
- 每个候选累计播放满 3 秒才能答；切换时约 5 ms 淡出→淡入、不交叉淡变（ITU-R BS.1534-3 §5.3；JS 1 ms 定时器调音量，近似，不是采样精确）
- 导出字段（与 rubric 对齐）：每题 `slots`（字母→模型）· `qorder` · `answer{clearer, natural, eaten[], usable{}, tags{}}`（原始字母）· `by_model{…}`（揭盲后）· `plays` · `listen_s` · `switches` · `started_at` / `answered_at` · `note`；汇总里有 `head_to_head`（两两逐段：先比「开会」评级，平级再看谁被选最清楚）
- 揭盲表 base64 存在 `manifest.js` 的 `slotmap` 字段，只防一眼看到（字段不叫 key：gitleaks 会当成密钥拦提交）
- 冒烟：Playwright + 系统 Chrome 无头、`--mute-audio`、`play()` 打桩成只发 `timeupdate`、虚拟时钟快进；走完 14 题并导出，无报错

## 客观分数（`score.py`，结果 `scores.json`）

均值（12 段；VoxLock 两列见后）：

| 版本 | SIG | BAK | OVRL | ESTOI | SI-SDR dB | 删除率 | 替换率 | 脑补率 |
|---|---|---|---|---|---|---|---|---|
| 原声 | 2.35 | 1.88 | 1.79 | 0.55 | 1.26 | 3.2% | 15.4% | 6.0% |
| DPDFNet-2 | **3.21** | 3.58 | 2.75 | **0.63** | **6.64** | 4.6% | 22.7% | 10.5% |
| GTCRN | 2.96 | 3.29 | 2.45 | 0.59 | 3.68 | **4.3%** | **20.1%** | 10.9% |
| FastEnhancer-M | 3.12 | **3.84** | **2.81** | 0.59 | 4.90 | 6.1% | 30.1% | **8.2%** |

非语音段压低（输出非语音段功率 ÷ 原声非语音段功率）：均值被 c09 / c10 两段接近静音（−60 ~ −72 dB）拉偏，改报**中位数**（只算非语音段 ≥ 1 s 的 9 段，c01–c03 只有 0.08–0.6 s 剔除）：DPDFNet-2 −6.2 dB · FastEnhancer-M −6.3 dB · GTCRN −4.8 dB。

逐段（OVRL / ESTOI / 删除率）：

| 段 | 原声 | DPDFNet-2 | GTCRN | FastEnhancer-M |
|---|---|---|---|---|
| c01 | 1.90 / 0.65 / 0% | 3.32 / 0.85 / 0% | 2.76 / 0.78 / 0% | 3.26 / 0.83 / 0% |
| c02 | 1.12 / 0.48 / 2% | 2.67 / 0.70 / 2% | 2.01 / 0.57 / 5% | 2.56 / 0.67 / 8% |
| c03 | 1.08 / 0.27 / 33% | 2.35 / 0.42 / 30% | 1.94 / 0.34 / 39% | 2.39 / 0.40 / 21% |
| c04 | 2.24 / 0.68 / 0% | 2.81 / 0.68 / 0% | 2.51 / 0.69 / 5% | 2.83 / 0.57 / 5% |
| c05 | 2.31 / 0.43 / 0% | 3.16 / 0.45 / 0% | 2.79 / 0.42 / 0% | 2.86 / 0.39 / 0% |
| c06 | 1.33 / 0.40 / 0% | 2.37 / 0.59 / 12% | 2.50 / 0.48 / 0% | 2.59 / 0.55 / 17% |
| c07 | 1.10 / 0.21 / 0% | 2.07 / 0.29 / 0% | 1.55 / 0.18 / 0% | 2.22 / 0.24 / 16% |
| c08 | 2.74 / 0.75 / 0% | 3.07 / 0.88 / 0% | 2.75 / 0.78 / 0% | 3.11 / 0.87 / 0% |
| c09 | 2.71 / 0.94 / 0% | 3.37 / 0.97 / 0% | 3.10 / 0.93 / 0% | 3.31 / 0.95 / 0% |
| c10 | 1.08 / 0.57 / 0% | 2.92 / 0.87 / 0% | 2.18 / 0.74 / 0% | 2.92 / 0.87 / 0% |
| c11 | 1.11 / 0.53 / 2% | 2.00 / 0.45 / 10% | 2.14 / 0.51 / 2% | 2.48 / 0.46 / 7% |
| c12 | 2.71 / 0.64 / 0% | 2.93 / 0.43 / 0% | 3.14 / 0.62 / 0% | 3.13 / 0.31 / 0% |

两两逐段胜负（`scores.json.pairwise_wins`，胜:负:平）：

| 对比 | OVRL | ESTOI | SI-SDR | 删除率 |
|---|---|---|---|---|
| DPDFNet-2 对 GTCRN | 9:3:0 | 9:3:0 | 8:4:0 | 3:2:7 |
| DPDFNet-2 对 FastEnhancer-M | 4:7:1 | 11:1:0 | 10:2:0 | 4:2:6 |
| GTCRN 对 FastEnhancer-M | 1:11:0 | 4:8:0 | 3:9:0 | 4:1:7 |

不是每项都 DPDFNet-2 领先：c11、c12 上它的 ESTOI 低于 GTCRN 和原声；c06、c11 上 GTCRN 的 OVRL 更高。

指标口径：DNSMOS P.835 用人实际听到的归一后文件，**非个性化版**（`is_personalized_MOS=False`，不惩罚旁人说话）；ESTOI、SI-SDR 用对齐后未归一文件对干净参考；非语音段按干净参考能量 VAD（20 ms 帧、低于最响帧 35 dB、前后各扩 100 ms）。

### 纯噪音、无旁人场景（c09 杯碟、c10 钢琴）
| 段 | 指标 | 原声 | DPDFNet-2 | GTCRN | FastEnhancer-M | VoxLock 给定 |
|---|---|---|---|---|---|---|
| c09 | OVRL / BAK | 2.71 / 2.93 | 3.37 / 4.19 | 3.10 / 3.81 | 3.31 / 4.14 | 3.04 / 3.75 |
| c09 | 非语音压低 dB | 0 | −72.4 | −6.6 | −6.7 | **−1.1** |
| c10 | OVRL / BAK | 1.08 / 1.18 | 2.92 / 4.18 | 2.18 / 3.79 | 2.92 / 4.18 | 2.43 / 3.34 |
| c10 | 非语音压低 dB | 0 | −61.8 | −28.8 | −63.4 | −26.5 |

杯碟这种瞬态噪音，只有 DPDFNet-2 在停顿里压干净；钢琴 DPDFNet-2 与 FastEnhancer-M 相当。VoxLock 给定在两段上压环境噪音都不如通用模型（pse 关心的「能否当主模型」证据，方向一致但只有两段）。

## 吃字（Qwen3-ASR，人定 D28）
- 本机 42model 服务 `qwen3-asr-plus:1.7b`（OpenAI 兼容 `/v1/audio/transcriptions`，temperature 0，语言 en / zh）。参照 = 干净参考的转写；英文按词，中文按字（夹在中文里的英文按词）。删除率 = 删除 ÷ 参照长度；替换率同理；**插入 ÷ 参照长度记为「脑补率」**
- **重复性**：84 个文件各转两次，**84/84 逐字一致**，没有不一致的文件
- **逐词明细**：`eaten_words.md`（`eaten_words.py` 生成）列每段「原声转写里在、降噪后被删」的词与两份转写原文。合计新删：DPDFNet-2 7 · GTCRN 7 · FastEnhancer-M 14（VoxLock 给定 25、自动 7）
- **是真吞字还是 ASR 波动？** 同一文件重复转写不波动。但 ASR 对微小声学差异很敏感：一次重跑中 c03 只因同段一起降峰值多降了 **0.03 dB**，GTCRN 那份的转写后半句就整句变了。所以**几个词的差别不能当真吞字的证据**，要人耳答。值得重点听：c06 DPDFNet-2 与 FastEnhancer-M 都丢 "bannister in the"、c07 FastEnhancer-M 丢 "its … small, a"、c11 DPDFNet-2「红色的」变「这个」
- 「新删」用位置集合相减，等代价对齐的平局可能造成少量假阳性（推断）
- c12（中文目标、英文邻座）里，原声与三个通用模型的转写都带出了英文干扰者的话（插入）；VoxLock 给定把旁人压掉，脑补率最低（0.4%）

## FastEnhancer 计时（`bench_fastenhancer.py`，结果 `fastenhancer_bench.json`）
Apple M5，macOS 27.0.1，onnxruntime 单线程，输入同 exp001（`resources/gtcrn/test_wavs/mix.wav`），热身 200 步后计 6000 步；官方 wav2wav 图，计的是整步（含 STFT/iSTFT/OLA）。

| 档 | hop / 周期 | p50 ms | p99.9 ms | max ms | max 占周期 | ≤ 周期一半 | ≤ 2 ms 线 |
|---|---|---|---|---|---|---|---|
| B | 256 / 16 ms | 0.34 | 0.56 | 0.78 | 4.9% | ✅ | ✅ |
| S | 256 / 16 ms | 0.55 | 0.90 | 1.04 | 6.5% | ✅ | ✅ |
| **M** | 160 / 10 ms | 0.69 | 1.06 | **1.15** | **11.5%** | ✅ | ✅ |

三档都远在半周期内，按规则（大总管定：「最坏一帧 ≤ 周期一半」里最大的一档）选 **M**。对照 exp001：DPDFNet-2 最坏 0.89 ms、GTCRN 0.62 ms。端到端附加延迟按 002 口径（直通 6–10 ms + 窗 32 ms + 前瞻 0）约 **38–42 ms**，与 GTCRN 同档（前瞻 0 是 sota 的说法与窗长推算，未对照论文核）。输出与输入能量比 −2.3 dB，没有非有限值。L 档没测（交代只要 B/S/M）。

## VoxLock（已出局，留档）
- 计时（`bench_voxlock.py` → `voxlock_bench.json`，同口径，计 `Stream.push()` 整次）：int8 单帧（8 ms）p50 8.36 / p99.9 17.8 / max 23.7 ms；10 ms 块 p99.9 16.6 ms；40 ms 块 p99.9 31.0 ms（77%）、max 40.9 ms（102%）。fp32 单帧 p50 6.81 ms，低于 8 ms 周期，但 p99.9 12.8 ms 仍超；只记了分位数，没记均值、没做持续运行测试。**结论只对「int8 单线程」成立：跟不上实时**；fp32 加缓冲、多线程、Core ML 都没测。20 ms 块 p50 反而高于 40 ms 块折算值，可能是每次 2 / 3 帧交替、输入形状不固定（推断）
- 延迟推算：窗 32 ms、前瞻 0 → 理论 38–42 ms；单线程要攒 40 ms 一块才勉强跟上 → 约 70–74 ms 起
- 分数（均值）：给定样本 OVRL 2.49、ESTOI 0.64（最高）、删除率 9.1%（c03 79%）、脑补率 0.4%；自动锁定 OVRL 1.96，接近原声
- 坑：注册样本取自同一说话人同一章节 / 同一录音（c01–c03 甚至和测试内容重叠），比真实用麦克风注册乐观；自动锁定在 11/12 段 3.4–8.7 s 才锁上，c03 整段没锁、输出逐样本等于原声；锁定那一刻会把之前最多 30 s 一次性重算（`core.py:224-234`），离线看不出这个算力尖峰
- 页面不再放 VoxLock；`files.json` 里两列音频还在

## 事故与教训（2026-10-30）
重建 VoxLock 版页面时同路径覆盖了旧 build `20261030102301-68ab` 的 `manifest.js` / `unblind-key.json`（人已在上面答了 6 题），又在清理「孤儿音频」时删了旧 build 的 37 个音频。只从 git 里的旧 `align.json`（文件名→模型）加打印过的 manifest 片段恢复出 c01–c03 的对照（`builds/20261030102301-68ab/`，大总管已揭盲其中 2 题）。改法：每个 build 独立子目录、浏览器存储键带 build、对照表进 git、不删任何旧文件。

## 已知局限
- 素材全是合成或公开录音，没有作者本人的真实咖啡馆录音；中文只 2 段
- 客观分的参照是「干净参考的 ASR 转写」，不是人工转写；LibriSpeech 段附了人工转写（`scores.json.per_clip.*.truth`）
- c01–c03 的干净参考本身有底噪，非语音段又短，非语音段指标在这三段不可信
- 说话区间响度对齐依赖干净参考的 VAD；真实录音没有干净参考，这一招只适用于离线评测
- 「听满 3 秒」只能保证听过，不能保证认真比；检验题沿用原段描述，可能被认出

## 文件
| 文件 | 作用 |
|---|---|
| `_paths.py` | 找到放 `resources/` 的主目录（worktree 里没有 resources/ 实物） |
| `_common.py` | 共用的 VAD 与切词 |
| `make_clips.py` | 素材 + VoxLock 注册样本 → `_tmp/20261030-exp003/src/`、`clips.json` |
| `enhance.py` | 各候选逐帧处理、对齐、说话区间响度 → `raw/`、`audio/`、`files.json`、`align.json` |
| `build_page.py` + `ab_page.html` | 对比页 → `_tmp/20261030-exp003/builds/<build>/`，对照表 → `builds/<build>/` |
| `score.py` | 客观分 + Qwen3-ASR 吃字 → `scores.json`（转写缓存 `_tmp/20261030-exp003/asr_cache.json`） |
| `eaten_words.py` | 逐词吃字明细 → `eaten_words.md` |
| `bench_fastenhancer.py` / `bench_voxlock.py` | 计时 → `fastenhancer_bench.json` / `voxlock_bench.json` |
| `builds/<build>/` | 每个 build 的揭盲表、manifest、files.json（进版本库） |
| `results/` | 人交回的答卷 |

复现（worktree 根下，`PY=<主目录>/_tmp/exp002-venv/bin/python`，推理前先申请名额）：`$PY …/make_clips.py` → `$PY …/enhance.py` → `$PY …/build_page.py [--cands a,b,c]` → `$PY …/score.py --asr` → `python3 …/eaten_words.py`；计时脚本单独跑。

仓外权重：

| 文件 | 来源 | 字节 | sha256 |
|---|---|---|---|
| `~/vdn-data/models/fastenhancer-dns-v1.0.0/fastenhancer_m.onnx` | github.com/aask1357/fastenhancer release `onnx-dns-v1.0.0`（MIT） | 2033752 | `29da81b46ad42758d9f586fb1706902862ec3f0337d6bd49670c5565e42382fb` |
| `…/fastenhancer_s.onnx` | 同上 | 832813 | `d7c937d6b475dbe89a6e6a69853b685fe62d629f4c96077559c556ac6f8aaa2d` |
| `…/fastenhancer_b.onnx` | 同上 | 413220 | `e2d8758a91a5c7d59334fdd4b871900cbd311d6694d99d9e0a77662063e60434` |
| `~/vdn-data/models/voxlock/enroll.onnx` | HF `arbazsiddiqui/voxlock` rev `2413b292…`，`onnx/v2/`（Apache-2.0） | 21023799 | `81f305c9b7a32f13d1401f53bb4c6301826a607c9e807e60912f36ef1b9fae29` |
| `…/voxlock/chunk_int8.onnx` | 同上 | 56560589 | `c50ffdc8d723fd4f91cb08889f3d0143491f95967b705f35c444a0b4afe88172` |
| `…/voxlock/chunk.onnx`（fp32） | 同上 | 77788950 | `11093eda6001794ffa03cd018dc343b12848fc739a2f151e26530e0a66f9659a` |
| `…/voxlock/config.yaml` | 同上 | 2431 | `786230c34b7f40f4a23eb1dd791d917010e74443a7e8125eee7e9f14fae0224b` |

whisper ggml large-v3-turbo（1624555275 B，sha1 `4af2b29d7ec73d781377bfd1758ca957a807e941`）下载核对过，人定改用 Qwen3-ASR 后已删（D28）。

## 待人拍板
1. **延迟档（D01）**：听完后在 ≤ 45 ms（GTCRN 或 FastEnhancer-M）与 ≤ 75 ms（DPDFNet-2）之间选。页面完成页的「DPDFNet-2 对 GTCRN 逐段」一行和导出里的 `head_to_head` 是主要证据。默认：听完再定
2. **45 ms 档里选谁**：FastEnhancer-M（DNSMOS 高、背景干净、ASR 吃字多）还是 GTCRN（吃字少、背景压得浅）。默认：按你耳朵里「吃字」与「背景」哪个更难忍
