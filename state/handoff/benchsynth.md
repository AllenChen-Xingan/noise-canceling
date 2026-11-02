# 现状页 · benchsynth（research/016-bench-synth · issue #41）

大总管：noise-canceling-e3（旧名 c4 / 69）· 批次 20261102-b7

- 2026-10-31 报到；读交代与 common.md
- 2026-10-31 读 D44–D50；开始核实四条诊断
- 2026-10-31 诊断 1–4 核实成立；新发现：全部带人耳答卷的片段（exp003 12 段、exp005 8 段）都是加法混合，真实锚为零
- 2026-10-31 起调研 workflow wf_4b3f7b2d-afd（6 路 + 6 怀疑派），笔记写 docs/research/016_bench-synth/notes/
- 2026-10-31 大总管给编号 exp008；人新指示「可以多弄几个来对比」→ 系统阶梯 ≥ 12（独立模型为主）；派子代理盘点可跑模型 → notes/G-system-ladder.md
- 2026-10-31 exp008 数据侧检查完成（build 20261031-102406-0c67，348 段，不占名额）：S1/S2/S3 全过；S6 合成杯碟、合成音乐「不过」，合成 babble 临界/过，稳态无参照 D0。中途修两处：float wav 文件头含时间戳 → 改样本哈希；电平在设备链前定 → 实测 SNR 偏 6.3 dB → 改到设备链后
- 2026-10-31 调研 workflow 完成（12 代理，6 篇笔记各带怀疑派复核）；名额已拿到（第一轮 12 系统 + 理想掩码），rank.py 全量在跑（后台）；② cargo fetch ③ 小下载 已批，① Hush dylib ④ torch 大总管在问人
- 2026-10-31 benchmark.md、synth-standard.md 正文写完（D0–D4 分级、S1–S12）；待 rank 结果写 mvp.md、decision.md
- 2026-10-31 第一轮阶梯跑完（348 段 × 13 系统，6287 s，0 错误），名额已报结果：S7 自身稳（OVRL τ_AB 0.85）· S8 与人耳反向（0.25–0.38，3 系统 48 对，功效低）。大总管再给名额做第二轮：torch 已装（主目录 _tmp/exp008-torch-venv）；Hush ×2 + DFN3 被本会话权限检查拦下（预编译 dylib），不绕，记缺口；其余 7 个三个子代理在接。mvp.md、decision.md 初稿写完（待第二轮回填）
- 2026-10-31 三路对抗自审（统计怀疑派 / 红队 / 实现者）回来并已改：S7 单格 τ 无区分力（错配格也 0.85）→ 改用交互一致性 S7b（SNR/噪声/SIR 轴过、混响轴不过）；S8「反向」降为「判不了」（屏聚类区间含 0.5）；S10 隔离不过（exp008 素材与 exp006 训练段重叠）；S2 空检补上后仍过；rank.py 加带护栏压噪指标；synth-standard 改定级单位为（格, verifier）、族定义、真实锚定义、D4-add、预注册、清单样本级区间；加第六节 D51「已知好坏的合成配对」。第二轮：RNNoise / WebRTC ×2 出音频在打分，torch 五个在跑（机器负载 ~35，慢）
- 2026-11-02 新会话接续（vdn 会话 10-31 晚中断）。盘点第二轮：RNNoise · WebRTC ×2 · MossFormer2-48k 出全 348 段、未打分；FRCRN 179 · MossFormerGAN 44 · URGENT-BSRNN 40 · G-MaP-SE 11 段中断；Hush ×2 + DFN3 记缺口。上轮留的两个未提交改动（mossformer2 记录跑完 348 段、现状页一行）核对无误，已由 hook 提交
- 2026-11-02 名额①（4 个系统打分 `rank.py --tag r2`）已拿到并在跑；名额②（续跑 4 个 torch 系统）排在 ears 后
- 2026-11-02 按 D62–D65 改正文：真实锚「为零」改述为「开发阶段不追（D62），数据侧目标级 D2」；S4 / S10 / S11 加公开集测试份 / 训练份切分与 SA 可训练；待拍板 1、2、8 标已定。S10 红队 P0 处理定为「标限制（0c67 只评第三方模型）+ 下一 build 按 D63 换池」，写进 synth-standard S10 处理段、mvp 2.2、exp008 README。rankstat.py 加第二轮变体 / 族代表、输出加 tag 不覆盖第一轮
