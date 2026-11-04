# 023 第④路笔记：中文会议 / 对话真实录音

> 2026-11-03 · 手下 mtgdata 的子代理 r4-zh。只读网页与论文，未下载数据本体（只取过 5 篇 arXiv 论文 PDF，各 < 2 MB，放会话临时目录，不入库）。
> 新卡在 `../cards/`；014 已有的 6 集只补字段，见 `r4-zh-addendum.md`。

## 一句话结论（事实，非建议）
中文公开集里**没有找到任何笔记本 / Mac 内置麦录的段**。最接近「消费设备 + 中文 + 会议」的是 MISP-Meeting（讯飞智能办公本内置 8 麦 + 相机麦 + 头戴近讲）【论文】和 WenetSpeech 的 TEST_MEETING（真实会议远场，设备未写）【论文】。

## 新卡一览（14 张）
下表用于横向比较设备与许可；接近度未评，留给汇总。

| 卡 | 真实 / 回放 | 场景键 | 设备键 | 近讲参考 | 转写 | 许可结论 | 获取 |
|---|---|---|---|---|---|---|---|
| magicdata-ramc | 真实 | home | phone | 没有 | 有，带时间戳 | 只评测 | 直接下载 |
| wenetspeech | 真实（TEST_MEETING） | meeting-room, other | unknown | 没有 | 有 | 只评测 | 注册 |
| kespeech | 真实 | other | phone | 没有 | 有 | 只评测 | 直接下载 |
| aishell-1 | 真实 | other | studio | 没有 | 有 | 训练可（有歧义） | 直接下载 |
| aishell-2 | 真实 | other | phone, studio | 部分 | 有 | 未查到 | 签协议 |
| aishell-dmash-ffsvc2020 | 真实 | home | phone, tablet, array, close-talk, studio | 有 | 部分 | 只评测 | 签协议 |
| magicdata-read-slr68 | 真实 | other | phone | 没有 | 有 | 只评测 | 直接下载 |
| primewords-set1 | 真实 | other | phone | 没有 | 有 | 只评测 | 直接下载 |
| st-cmds | 真实 | other | phone | 没有 | 有 | 只评测 | 直接下载 |
| aidatatang-200zh | 真实 | other | phone | 没有 | 有 | 只评测 | 已下架 |
| cn-celeb | 真实（媒体） | other | unknown | 没有 | 没有 | 只评测 | 直接下载 |
| hkust-mandarin-telephone | 真实（电话 8 kHz） | other | phone | 没有 | 有（另购） | 不可 | 付费 |
| wenetspeech-yue | 真实（媒体） | other | unknown | 没有 | 有 | 只评测 | 直接下载元数据，音频需联系 |
| mdcc | 真实（有声书） | other | studio | 没有 | 有 | 只评测 | 签协议 |

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
- AISHELL-2 官网许可页 WebFetch 只拿到标题，许可「未查到」，需换浏览器复核。
- aidatatang_200zh 原页 404，许可与数字都来自转述。
- WenetSpeech TEST_MEETING 设备、单独体积均未查到。
- MagicData-RAMC 是否分通道、有无设备型号标签未查到。
- KeSpeech 总 GB 未查到。
