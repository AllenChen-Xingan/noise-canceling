# E-device · 调研与怀疑派记录

## 未找到 / 不符合命门
- MacBook（或任何 Apple 笔记本）内置麦的公开实测 RIR / 频响数据：未查到。只找到 DXOMARK 对 MacBook Pro 14" (2023) 的麦克风频响/指向性图（商业网站图表，无原始数据可下载，https://www.dxomark.com/apple-macbook-pro-14-2023-laptop-test/ 【二手】）。
- 除 ACE（Chromebook Pixel）以外，本路查过的 RIR 库（BUT ReverbDB、AIR、MIRaGe、dEchorate、MIT IR Survey、OpenAIR、VoiceHome-2、DIRHA、EARS-Reverb）都没有笔记本内置麦通道；AIR 只有模拟手机，ACE 的“Mobile”也是用 DPA 4060 搭的模拟阵列。
- DIRHA：公开 RIR 不存在（BUT ReverbDB 论文 §II-B 写“No RIRs are public”），只有 LDC 付费版语音；FBK 原 dirha.fbk.eu 免费子集的现行状态未查到。
- Microsoft AEC Challenge 真实录音（>10,000 台设备，Windows 与 Android）可能含笔记本内置麦，但不是 RIR，也不是已知干净源成对数据；仓库写的是“按 Microsoft 收到时的原始条款”提供，整体许可未查到，有无笔记本也未核实（https://github.com/microsoft/AEC-Challenge 【官方】）。它不在本路范围，留给其他路。
- DAPS 回放用的扬声器型号和距离、有没有转写：未查到。
- DR-VCTK 用了哪些设备、现行版本的许可原文：未查到（README.txt 和 End-user Licence 都没读到内容）。
- Aachen AIR 官方页这次直连被拒（ECONNREFUSED），MIT 许可的说法来自搜索摘录，需要再核一次。
- OpenAIR 站点 TLS 证书过期，没能直连；按条目逐个标注许可的说法来自搜索摘录。
- Univ. of Rochester 的 Smart Speaker Command Dataset（2024，14 个房间的 IR）：由哪台设备录的未查到，没有展开。

## 预判核验
- 预判「ACE 含笔记本麦（Lin8Ch 或 laptop）」：部分成立，需要修正。笔记本那一档的压缩包名是 Chromebook，不是 Lin8Ch：2 通道、Chromebook Pixel 的内置麦、间距 62 mm、用 arecord 直录、没做设备均衡（Eaton et al. WASPAA 2015 §2.2【论文】；Zenodo 6257551 的压缩包列表【官方】）。Lin8Ch 是用 DPA 4060 搭的 8 通道线阵，不是笔记本。“Mobile”3 通道是 DPA 4060 摆成的直角三角形模拟阵列，不是真手机。
- 预判「ACE 含环境噪声」：成立，而且比预期好。每个房间、每个麦位都有 ambient、fan、babble 三种噪声。babble 是 4–7 名真人坐在声源附近持续朗读 TIMIT 句子或自带材料（WASPAA 2015 §2.3；arXiv:1606.03365 表 3【论文】）。Building Lobby 带咖啡厅，有门禁、电梯播报和路人，和 D42 相关。笔记本档的噪声录音也有（Chromebook 压缩包里有 RIR 和噪声）。
- 预判「ACE 是唯一含笔记本麦的公开 RIR」：在本路查过的范围内成立（BUT、AIR、MIRaGe、dEchorate、MIT、OpenAIR、VoiceHome-2、DIRHA、EARS-Reverb 都没有）。全网是否唯一无法证明，只能写“未查到反例”。另外它不是 MacBook，是 2013 年前后的 Chromebook Pixel。
- 附带发现：ACE 的许可有冲突。Zenodo 记录 6257551 写 CC BY 4.0【官方】，Imperial 项目页和 IEEE DataPort 写 CC BY-ND 4.0【官方/二手】。建议交人拍板；拍板前只做校准和评测。
- 预判「DAPS 是 CC BY-NC」：成立，原文是 CC BY-NC 4.0（Zenodo record 4660670，v1.0，16.1 GB【官方】）。设备是 iPad Air 和 iPhone 5S（HF 数据卡【二手】）。
- 预判「BUT ReverbDB 是 CC BY 4.0」：成立（官方页 https://speech.fit.vut.cz/software/but-speech-fit-reverb-database【官方】）。补充：NIST SRE10 和 HUB5 的重录部分只发给有 LDC 授权的单位；麦是驻极体模块加 Sennheiser MKE 2，不含笔记本或手机（JSTSP 2019 附录 A【论文】）。

## 怀疑派更正
- DR-VCTK · 原说法「设备型号与距离未查到、许可未查到、接近度 2」→ 包内 README 明列 8 通道：MacBook Air(2ch)、Apogee MiC、Blue Snowball、iPhone 5S(2ch)、iPad(2ch)；Bose SoundLink III 回放，距麦 2 m；中等办公室开/关窗 16 条件；训练 28 人+测试 2 人；许可 CC BY 4.0 → license_class 改可训练、接近度升到 3（本族唯一 Apple 笔记本内置麦平行数据）· https://datashare.ed.ac.uk/bitstream/handle/10283/3038/README.txt 【官方】
- ACE · 原说法「按 Zenodo 可训练」→ Imperial 项目页原文 “Creative Commons Attribution-NoDerivatives 4.0 International License”，与 Zenodo CC BY 4.0 冲突属实；冲突未决前 license_class 改为只评测（保守）· https://www.imperial.ac.uk/speech-audio-processing/projects/ace-challenge/ 【官方】
- ACE · 原说法「Zenodo v1，2022 上传」→ Zenodo 记录发布日标 2015-03-02，2022-02-24 为建档日，2024-07-17 最后修改 · https://zenodo.org/records/6257551 【官方】
- ACE · 接近度 3.5 → 3：babble 为真人朗读而非自由交谈，目标语音为消声室源需人工卷积相加（属“真实噪声+人工混合”），场所为大堂非咖啡馆
- VOiCES · 原说法「麦型号（是否含领夹）未查到」→ 7×SHURE SM58、4×AKG 417L 领夹、1×SHURE SM11 领夹；领夹麦摆在房间各处并非佩戴在说话人身上，不构成近讲参考；距离约 38–228 inch；babble 由 3 只扬声器回放、TV/音乐 1 只；release 1 为 2 个房间（项目页称 3 个）；源语音 15 h / 3,903 条 · arXiv:1804.05053 §2.1–2.2、表 1 【论文】；https://iqtlabs.github.io/voices/ 【官方】
- DAPS · 原说法「转写 未查到」→ supplementary_files 含朗读脚本（每人 5 篇，篇章级无时间戳）【二手，HF 数据卡】；Zenodo 页只写 tablets and smartphones，iPad Air/iPhone 5S 型号仅二手来源；Zenodo 发布日 2014-05-20 · https://zenodo.org/records/4660670 【官方】
- VoiceHome-2 · 原说法「转写未查到」→ 论文摘要称另提供干净语音与自发语音、全部数据已完整标注；共 12 个房间；干净语音是否同步录制仍未查到 · https://hal.archives-ouvertes.fr/hal-01923108 【论文】
- DIRHA · 补充：主体麦 Shure MX391/O；同步近讲论文未提（close-talk 基线用录音棚另录），LDC 页未列费用（需登录）· arXiv:1710.02560 §3.1–3.4 【论文】；https://catalog.ldc.upenn.edu/LDC2018S01 【官方】
- EARS-Reverb · real_recording「部分」→「没有」：语音为消声室录音、混合为人工卷积，仅 RIR 真实；ACE 贡献 84 条 RIR，上游 ACE 许可冲突会传导；benchmark 已有 v2（2025-04）· arXiv:2406.06185 表 2；ears_benchmark README
- MIRaGe · 补充：官方下载页确实未写任何许可；babble 为 8 只扬声器各放不同多人语音、距墙约 1 m；阵列距网格 1/2/3 m · https://asap.ite.tul.cz/downloads/MIRaGe/ 【官方】
- MIT IR Survey · 核实：官方页不写许可、不写设备，仅写 1.5 m 间距；HF 镜像的 CC BY 4.0 为第三方自标，维持未查到
- AIR · MIT 许可仍只有搜索摘录引用官方页（“see the included MIT license file”），包内文件未亲见，证据等级维持【二手】
- BUT ReverbDB / dEchorate · 核实无误（BUT：CC BY 4.0、8.7 GB+117 GB、rel_19_06、LibriSpeech 重录仅 5 房间；dEchorate：CC BY 4.0、83.9 GB、v1.0 有新版）
- not_found 第 1 条「除 DXOMARK 外无 Apple 笔记本内置麦公开数据」与第 6 条不成立：DR-VCTK 含 MacBook Air 2 通道真实重录

## 怀疑派：漏掉的
- MC-WSJ-AV（REVERB Challenge RealData 来源）· 真实会议室里真人朗读，同步录头戴麦 + 领夹麦 + 阵列，近讲参考齐全；LDC 付费 · https://catalog.ldc.upenn.edu/LDC2014S03
- Libri-Adhoc40 · LibriSpeech 在真实办公室回放，由 40 个分布式麦节点同步重录，有干净源，可作设备与距离的分布参考 · https://github.com/ISmallFish/Libri-adhoc40
- Microsoft DNS Challenge 盲测集 / 真实录音 · 用户在笔记本、耳机等真实设备上录的真实噪声语音，没有干净参考，但最贴近 D42 的设备链；许可要逐包核 · https://github.com/microsoft/DNS-Challenge
- Microsoft AEC Challenge 真实录音 · 超过一万台 Windows / Android 设备的真实近端与远端录音，可能含笔记本内置麦；许可沿用原条款，需要核对 · https://github.com/microsoft/AEC-Challenge
- NISQA Corpus（含 LIVETALK 真人通话录音）· 真人经真实手机和笔记本通话链路、在真实环境里录的语音，带 MOS 分，可用于设备链和听测；子集许可不同 · https://github.com/gabrielmittag/NISQA/wiki/NISQA-Corpus
- ConferencingSpeech 2021 · 在真实会议室用麦克风阵列回放重录，带真实噪声，有干净源 · https://github.com/ConferencingSpeech/ConferencingSpeech2021
- OpenSLR SLR28（RIRS_NOISES）· 汇总了 RWCP、REVERB、AIR 等真实 RIR 和真实点源噪声，Apache 2.0，可作为训练 RIR 池的基线 · https://openslr.org/28
