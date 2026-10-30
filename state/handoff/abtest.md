# 现状页 · abtest（批次 20261030-b2）

状态：**已停（应大总管要求，搬到 worktree 分支 exp/003-offline-ab 由新会话接手）**

## 手上
exp003：DPDFNet-2 对 GTCRN 的离线 AB 听测（交代 `_tmp/20261030-b2/abtest.md`）

## 做到哪
四个脚本 + 页面模板已写完（都还**没跑过**）；素材下载大半完成。还没申请过名额、没跑过任何推理。

## 下一步（接手者照做）
1. **改 `make_clips.py` 的 c11**：把 `DATA / "zh/zh_bubbling_well_road_timshi.ogg", 120.0, 9.5` 换成 `DATA / "zh/zh_popobing_landxia.ogg", 120.0, 9.5`；`main()` 里 `srcs` 列表同样替换。原因：TimShi 那份全程有约 −35 dB 底噪（语音约 −22 dB），不能当干净参考；婆婆冰（Landxia）静音段 −90 dB，120 s 起 9.5 s 有声占 89%
2. 等下载完：`SCAFE_48k.zip`（md5 1a4ee387f1900f4b15ba219481c76556）、`DKITCHEN_16k.zip`（md5 见 Zenodo API `records/1227121`）、`ggml-large-v3-turbo.bin`（sha1 应为 `4af2b29d7ec73d781377bfd1758ca957a807e941`，大总管要求核对并在 README 记 URL/哈希/大小）。核 md5 后在 `~/vdn-data/demand/` 解压（得到 `SCAFE/ch01.wav`、`DKITCHEN/ch01.wav`）。若后台下载因会话关闭中断，重下：`curl -sSL -o ~/vdn-data/demand/<名>.zip https://zenodo.org/api/records/1227121/files/<名>.zip/content`；whisper：`https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin`
3. 申请名额后依次跑（项目根下，venv 复用 `_tmp/exp002-venv`，已含 onnxruntime/librosa/scipy/pystoi/pyloudnorm/soundfile）：
   `make_clips.py` → `enhance.py` → `build_page.py` → `score.py --whisper`
   - 跑完核 `align.json`：互相关滞后应为 0（DPDFNet 已按 4 帧=640 样本平移，GTCRN 0 帧）；不为 0 就查
   - 核响度：三版 `lufs_after` 应相同（目标 −26 LUFS，峰值超 −1 dBFS 时三版一起降）
4. 浏览器打开 `_tmp/20261030-exp003/index.html` 冒烟（能播、切换、答题、导出）；**不要放声音**（红线），只看界面与控制台
5. 写 `docs/experiments/exp003-offline-ab/README.md`（素材清单：来源 URL · 许可 · sha256，`clips.json` 里有 `source_sha256`；处理方法；分数表；页面用法；局限与缺口）
6. 自审三视角（怀疑派 · 实现派 · 产品派），修完报大总管「abtest 可以收尾」+ 页面路径 + 一句客观分数谁领先

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
