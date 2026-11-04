# 建议先下的一小批（只列不下）

> 2026-11-03 · 手下 mtgdata · issue #82。用途：要拿真实录音做评测锚时，先下哪几份。**本文不下载任何东西**；下不下、何时下，见 `decision.md` 待人拍板。
> **措辞口径**：本文说「未找到 / 🟥」一律指「本轮检索范围内，没有确认到能筛出的匹配子集」，不等于世上没有；「自录 / 回放重录」是补法建议，不是结论。
> 挑选顺序：笔记本 / Mac 录的优先 → 带转写的优先 → 许可干净的优先 → 体积小的优先。每个开会地点至少一份；找不到的写明。
> 大小标【实测】的，是读远端 HTTP 头或 zip 目录得到的；标【官方文档】的，出自数据集页面。

## 一、D65 里已有的（约 40 GB，人已同意「要训练 / 合成时再下」）

下面这几份 D65 已列，它们本身就补了几个地点，**不用再问一次**。

| 数据集 | 补哪个格子 | 为什么 | 大小 |
|---|---|---|---|
| DR-VCTK | `open-office` × `mba-builtin` | 本轮找到的、能按地点对上的 MacBook Air 录音（回放重录），有干净参考；End-user Licence 未读，按 D65 只评测 | 1.67 GB【官方文档】 |
| ACE Chromebook 档 | 不补真实锚格子（合成原料） | 笔记本内置 2 麦录的实测 RIR + 现场噪声，语音要自己卷积相加，适合做「真实设备链」合成；许可两处冲突，按只评测 | 1.1 GB【官方文档】 |
| DNS4 盲测 | `home-family` · `open-office` × `non-laptop-desktop` / `non-laptop-phone` | 859 条逐条带设备字段（Mobile 638 / desktop 221） | 1.66 GB【实测】 |
| DNS5 盲测 | `cafe` · `transit-hub` | 标了咖啡馆 4 条、机场 25 条；`ap` 段分 desktop / mobile | 3.89 GB【实测】 |
| CHiME-6 dev + eval | `home-family` × 麦阵 + 近讲 | 真人晚餐聚会，近讲参考 + 时间戳转写 | 约 23 GB【官方文档】 |
| AliMeeting Eval + Test | `meeting-room`（中文） | 中文真人会议，近讲参考 + 转写 | 约 12.3 GB【014】 |

## 二、本次新建议的（不在 D65 里）

| # | 数据集 | 补哪个格子 | 为什么选它 | 许可 | 大小 |
|---|---|---|---|---|---|
| 1 | DNS 2021 两届盲测合集（V2 + V3 + 情绪 + 鼠标点击，一个 zip） | `home-family` · `open-office` × `mac-unknown`（31 条）/ `laptop-unknown`（约 330 条） | 不用签协议就能拿到的 Mac 真人录音只在这里（众包者自填 mac / macbook，没核实是不是内置麦；另一处 MOBIO 要签 EULA）；还带 600 条情绪语音，补「语体」缺口 | 未查到 → 按只评测 | 778 MB【实测】 |
| 2 | NISQA_TEST_LIVETALK | `cafe` · `car-passenger` · `transit-hub` · `street-outdoor` × `non-laptop-phone` | 4 个公共地点的真人真机通话，每条有人评分（MOS），能用来校准无参考打分；德语 | CC BY 4.0（仅研究）→ 只评测 | 232 条 × 6–12 s ≈ 35 min【推断：232 × 9 s】；但只随 NISQA 全集整包发布，`NISQA_Corpus.zip` 约 15.9 GB【官方文档】，能否单取未查到 |
| 3 | SIG 挑战 2023 真实测试集 | 无地点标签 × PC / 手机 | 2023 test 500 条真实发送端录音（PC / 手机混，无逐条设备标签），500 条都有转写，可做字准确率（吃字）回归；逐条人评分未公开，**不能**复现论文里的相关数字 | 未查到 → 按只评测 | 约 414 MB【官方文档：GitHub 文件大小合计】 |
| 4 | EasyCom | `canteen` | 真人对话 + 回放餐厅噪声，有头戴参考 + 转写；直接下载，不用签协议（ECHI 要签） | CC BY-NC 4.0 → 只评测 | 5 h 18 min；GB 未查到 |
| 5 | AISHELL-5 Eval1 | `car-passenger`（中文） | 中文车内真人对话，有转写 | CC BY-SA 4.0 → 训练可（D64） | 3.3 h；约 1.7 GB【补充卡】 |
| 6 | AMI 测试集（阵列混合 + 头戴混合两路） | `meeting-room`（英文） | 会议室里许可最干净（CC BY，可训练），有头戴参考 + 时间戳转写 | CC BY 4.0 → 训练可 | 约 9 h【推断：AMI 标准测试集 16 场】；GB 未查到 |

**合计**：已知大小的 1 + 3 号约 1.2 GB；5 号 AISHELL-5 Eval1 包约 1.7 GB【补充卡】；2 号若不能单取要按整包 15.9 GB 算；4、6 号大小未查到。不含 2 号整包时估计 < 10 GB【推断：按 16 kHz 单声道、约 18 h 折算约 2 GB，多通道再乘通道数】，含整包则约 20–25 GB。下载前要先逐份核大小。
**规矩**：单份 > 50 MB 先问大总管；累计 > 500 MB 要经大总管问人（common.md 第 3 条）。本批新增肯定超过 500 MB。

## 三、各地点有没有着落

| 场景键 | 第一、二节里的 | 没着落时怎么办 |
|---|---|---|
| `meeting-room` | AliMeeting · AMI | — |
| `open-office` | DR-VCTK · DNS 2021 | DR-VCTK 是回放，DNS 2021 众包段地点与家混、无逐条地点标签；建议：自录（`matrix.md` 第三节） |
| `home-family` | CHiME-6 · DNS 2021 · DNS4 | 本轮未确认「一人开会、家人在旁」的子集；建议：合成或自录 |
| `home-quiet` | 🟥 | MagicData-RAMC 只有整包 15 GB，不进小批；先用 DNS 2021 里的安静段代替（要人工挑） |
| `cafe` | DNS5 盲测 · LIVETALK | 没有笔记本录的 → 合成 |
| `canteen` | EasyCom | — |
| `transit-hub` | DNS5 盲测 · LIVETALK | 找到的真人来源是手机通话；笔记本的建议自录 |
| `bus-metro-cabin` | 🟥 | 本轮找到的真人集只有 CHiME-3 BUS（LDC 付费）；建议：合成（TAU / DEMAND 背景） |
| `car-passenger` | LIVETALK · AISHELL-5 | — |
| `street-outdoor` | LIVETALK | 找到的真人来源是手机通话与 VoxCeleb 户外采访（要人工挑）；笔记本的建议自录 |
| `library-classroom` | 🟥 | ASCEND（安静教室，领夹）可选，CC BY-SA → 训练可（D64） |
| `phone-booth` · `cowork` · `rail-air-cabin` · `hotel-room` | 🟥 | 本轮没找到也没专门查；先合成 |

## 四、刻意没选的
- **3D-Speaker**：笔记本录音最多（训练集 9.6 万条，按设备可筛，普通话），但音频许可读不到原文，先不选；核清后是中文笔记本锚的首选
- **MOBIO**：2008 款 MacBook 真人说话，要签 EULA 且签字人须在编，见 `decision.md` 第三节
- **MISP2021**：家里电视房真人对话正合适，但许可原文和下载入口都没找到
- **NOTSOFAR-1**：许可干净，但 HF 仓库要 token，且单场按设备拆得很碎，挑子集要先读目录；作为 AMI 的备选
- **CHiME-9 ECHI、MOBIO、ConferencingSpeech 2021**：要签协议 / 注册，见 `decision.md` 第三节
