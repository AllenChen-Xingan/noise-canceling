# AISHELL-1

- **名称**：AISHELL-1 Open Source Mandarin Speech Corpus（AISHELL-1，OpenSLR SLR33）
- **链接**：https://www.openslr.org/33/
- **许可**：Apache License v.2.0（同页另写「The data is free for academic use」；论文 §1 也写「under Apache 2.0 license」）→ 结论：训练可【官方文档 · 论文】
- **大小**：170+ h · 400 人；data_aishell.tgz 15 GB【官方文档 · 论文 §1】
- **场景键**：other（录音环境 OpenSLR 页与论文均未写，未查到）
- **设备键**：non-laptop-studio
- **笔记本段**：没有。原始 AISHELL-ASR0009（500 h）三路并行录：AT2035 高保真麦、安卓（三星 NOTE4/S6、OPPO A33/A95s、荣耀 6X）、iPhone（5c/6c/6s）；但 AISHELL-1 只发布高保真麦一路，手机路未公开发布【论文 §2】
- **干净参考**：没有（发布的就是高保真麦近讲本身）
- **转写**：有（句级，无时间戳）
- **语种**：普通话
- **语体**：朗读；无笑声、附和、情绪
- **获取方式**：直接下载（OpenSLR 及镜像）
- **与 D62 的关系**：真实录音（真人朗读，高保真麦）。开发阶段若用，只能作干净中文朗读底料（合成时叠噪声）或吃字检查的基线；test 子集大小未单列，需整包 15 GB
- **出处**：https://www.openslr.org/33/【官方文档】; https://arxiv.org/abs/1709.05522 §1–2【论文】

## 备注
- 任务点名找「设备子集」：AISHELL-1 的安卓 / iPhone 并行录音存在于 AISHELL-ASR0009，论文 §5.3.1 用过，但未公开发布；要拿需联系希尔贝壳（商业数据），本路未联系。
- 许可歧义：Apache 2.0 是宽松许可，但同页「free for academic use」一句可能被理解为仅学术；按 Apache 原文判训练可，怀疑派请复核。
- 怀疑派复核：许可在 OpenSLR 页与论文 §1 两处都写 Apache 2.0，维持「训练可」；「free for academic use」是描述句，不是附加条款。手机型号、§5.3.1 手机解码实验核对一致。改动：场景键括注「安静室内录音间」无出处（论文没有任何环境描述），改为未查到。
