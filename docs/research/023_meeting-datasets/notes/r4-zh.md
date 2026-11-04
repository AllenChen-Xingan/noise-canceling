# 023 第④路笔记：中文会议 / 对话真实录音

> 2026-11-03 · 手下 mtgdata 的子代理 r4-zh。只读网页与论文，未下载数据本体（只取过 5 篇 arXiv 论文 PDF，各 < 2 MB，放会话临时目录，不入库）。
> 新卡在 `../cards/`；014 已有的 6 集只补字段，见 `../cards-014-addendum.md`。

## 一句话结论（事实，非建议）
中文公开集里**没有找到带设备标签、能单独筛出的笔记本 / Mac 内置麦段**。Common Voice 中文子集是网页众包录音，必然混有笔记本内置麦，但没有设备字段、筛不出来（怀疑派补，见 addendum）【论文 · 推断】。最接近「消费设备 + 中文 + 会议」的是 MISP-Meeting（讯飞智能办公本内置 8 麦 + 相机麦 + 头戴近讲）【论文】和 WenetSpeech 的 TEST_MEETING（真实会议远场，设备未写）【论文】。

## 新卡一览（14 张）
下表用于横向比较设备与许可；接近度未评，留给汇总。

| 卡 | 真实 / 回放 | 场景键 | 设备键 | 近讲参考 | 转写 | 许可结论 | 获取 |
|---|---|---|---|---|---|---|---|
| magicdata-ramc | 真实 | home | phone | 没有 | 有，带时间戳 | 只评测 | 直接下载 |
| wenetspeech | 真实（TEST_MEETING） | meeting-room, other | unknown | 没有 | 有 | 只评测 | 注册 |
| kespeech | 真实 | other | phone | 没有 | 有 | 只评测 | 直接下载 |
| aishell-1 | 真实 | other | studio | 没有 | 有 | 训练可（有歧义） | 直接下载 |
| aishell-2 | 真实 | home, other | phone, studio | 部分 | 有 | 只评测（怀疑派据 Kaldi README 改） | 签协议 |
| aishell-dmash-ffsvc2020 | 真实 | home | phone, tablet, array, close-talk, unknown | 有 | 部分 | 只评测 | 签协议 |
| magicdata-read-slr68 | 真实 | other | phone | 没有 | 有 | 只评测 | 直接下载 |
| primewords-set1 | 真实 | other | phone | 没有 | 有 | 只评测 | 直接下载 |
| st-cmds | 真实 | other | phone | 没有 | 有 | 只评测 | 直接下载 |
| aidatatang-200zh | 真实 | other | phone | 没有 | 有 | 只评测 | 已下架 |
| cn-celeb | 真实（媒体） | other | unknown | 没有 | 没有 | 只评测 | 直接下载 |
| hkust-mandarin-telephone | 真实（电话 8 kHz） | other | phone | 没有 | 有（另购） | 不可 | 付费 |
| wenetspeech-yue | 真实（媒体） | other | unknown | 没有 | 有 | 只评测 | 直接下载元数据，音频需联系 |
| mdcc | 真实（有声书） | other | studio | 没有 | 有 | 只评测 | 签协议 |
| ascend（怀疑派补） | 真实（领夹近讲） | other | close-talk | 没有 | 有 | 只评测 | 直接下载 |

## 按任务点名逐项回答
1. **MagicData 对话 / 会议集**：RAMC 是唯一公开的 MagicData 对话集（180 h，手机，两人对话，有笑声等非语言事件标注）【论文】。MagicData 会议集未找到公开版。
2. **KeSpeech**：手机朗读 + 次方言；第二期有 clean / noisy 环境标签，是中文「真实手机底噪」少见来源【论文】。
3. **WenetSpeech 会议 / 播客段**：会议只有 TEST_MEETING 15 h（197 场真实会议抽样，远场）；播客 3892 h 强标注在训练集中。全集 Opus 32 kbps 压缩【论文】。
4. **AISHELL-1/2/3 设备子集**：AISHELL-1 只发布高保真麦，手机并行录音未公开；AISHELL-2 训练集只有 iPhone，dev / test 三通道（iOS / 安卓 / 麦）并行公开；AISHELL-3 是录音棚 TTS 语料（HF 卡 apache-2.0、约 85 h），无设备子集，未开卡【官方文档 · 论文】。
5. **CN-Celeb**：B 站等媒体的说话人识别集，无转写、无设备标签。
6. **Fearless Steps 中文**：未找到。Fearless Steps 是 NASA 阿波罗任务英语录音，没有中文版【推断：只知其为英语集，本路未逐页核】。
7. **字节 / 阿里 / 腾讯开源会议集**：阿里只有 AliMeeting（014 已有）；字节、腾讯未找到公开的会议录音集。腾讯会议只作为 WenetSpeech 的下载通道出现，不是数据来源【官方文档】。
8. **线上会议录音（腾讯会议、钉钉）公开集**：未找到。VCSum（239 场真实会议、230+ h）是中文会议**摘要**文本集，未见公开音频【论坛 / 博客，个人说法：搜索摘要】。
9. **HKUST 电话对话**：LDC 付费、8 kHz，不可再分发。
10. **粤语**：WenetSpeech-Yue（网络媒体，CC BY-NC 4.0）、MDCC（有声书，签协议）。另搜到 SpiCE（温哥华粤英双语对话，CC BY 4.0，约 68 段）、HKCanCor（只有转写无音频）、MYCanCor（马来西亚粤语，媒体需申请）、Longmao Cantonese-ASR（HF 门控，约 655 h 客服电话）——均未核原文，未开卡【论坛 / 博客，个人说法：搜索摘要】。

## 需要注册 / 签协议（只记，未操作）
- WenetSpeech：填 Google 表单申请密码；HF 镜像需同意共享联系方式
- AISHELL-2：学术免费，申请流程原文未读到
- AISHELL-DMASH / FFSVC 2020：邮件向组织方申请；supplement 需挑战平台账号 + Zenodo 申请，承诺不修改、不分发、不商用
- MDCC：签 MDCC_LICENSE 发作者邮箱
- MISP2021/2022/2023：页面仅写「available under license」，无入口
- HKUST：LDC 付费

## 证据缺口与怀疑派复核点
- AISHELL-2 官网许可页 WebFetch 只拿到标题；怀疑派已在 Kaldi 官方仓库 egs/aishell2/README.md 读到申请条款，许可改为只评测；申请表要求大学院系信息，个人项目能否申请未查到。
- aidatatang_200zh 原页现为「Resource not found」；怀疑派用 Wayback 存档亲读原页，许可 CC BY-NC-ND 4.0、18G 已改为【官方文档，存档】。
- WenetSpeech TEST_MEETING 设备、单独体积均未查到。
- MagicData-RAMC 是否分通道、有无设备型号标签未查到。
- KeSpeech 总 GB 未查到。

## 怀疑派复核（2026-11-03）
复核 14 张新卡 + addendum 6 节，许可逐条读原文，数字回论文或官方页。改动摘要：
- aishell-2：许可「未查到」→ 自定义申请条款、只评测（Kaldi 官方 README）；场景键补 home
- aidatatang-200zh：许可与大小改为 Wayback 存档原页亲读；删去无出处的「retracted as per the data owner wish」
- aishell-dmash-ffsvc2020：「前 90 句固定唤醒词」与论文不符，改为 30 + 60；设备键 studio → unknown
- aishell-1：场景括注「安静室内录音间」无出处，改未查到
- hkust：设备括注「固话 / 手机」无出处，删
- wenetspeech-yue：评测集表头实为「Speakers」；情绪标签出处只在 GitHub README
- mdcc：许可写成 PDF 原文条款；两个收件邮箱不一致
- addendum：misp-meeting 设备键「other」不在暂定键内，已改；补 common-voice.md 中文子集一节
- 补卡：ascend.md（中英夹杂自发对话，领夹近讲，CC BY-SA 4.0）
- 跨路提示：腾讯相关只有另一路的 tencent-corpus-cs2022.md（会议通话损伤 + 人评分），不是会议录音集，本路「腾讯未找到公开会议录音集」仍成立

