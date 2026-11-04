# 原始报告 · r5-usage（workflow wf_c790c606-7d6 · 2026-11-03，未经合成，供追溯）

## 要点（5 行内）

1. 没有一份调查直接问过「你在哪里开视频会议」，查到的都是「在哪里工作」，只能拿来近似。工作地点占比不等于开会地点占比（推断）。
2. 硬数据最强的一份是 SWAA 论文（美国，N=27,364，2021-11 至 2022-06）。按工时算：雇主场所 44.4%、家 29.1%、客户处 10.6%、亲友家 5.3%、共享办公 5.3%、公共场所 5.3%。也就是说，远程工时里约 1/3 不在自己家【论文】。
3. 在家时用哪个房间：独立书房/办公室 32%–55%，卧室 21%–31%，客厅 20%–23%，厨房 5%–9%。样本是 Buffer 2023（N=3,000，全球）、意大利 Frontiers 2021（N=1,934）、CraftJack（样本量不详）。
4. 干扰噪声排名：在家时，「人声」是首位（意大利研究 25% 认为是主源）。被打断的来源是宠物 47%、伴侣 32%、孩子 21%（FinanceBuzz，N=500，美国，2020）。开放办公室里首位是「同事打电话大声说话」，占 29%（Plantronics，N=2,184，7 国，2017）。
5. 以下 4 个场景键没有可靠比例：`phone-booth`、`canteen`、`transit-hub`、`library-classroom`。腾讯会议、飞书、钉钉也没有公开的「开会地点」数据，未查到。

---

## 一、硬数据明细（按来源）

### A. SWAA：按工时和出行次数拆的工作地点【论文】
- 出处：Caros, Guo, Zheng, Zhao,《The impacts of remote work on travel: insights from nearly three years of monthly surveys》，arXiv 2303.06186。见 §3（第 5 页）、§4.1.1、**Figure 1**（第 7 页）。链接：https://arxiv.org/pdf/2303.06186
- 样本：SWAA 月度在线面板，只含美国居民，已按 CPS 的年龄、性别、学历、收入重新加权。
- **Figure 1(a)，按工时**（2021-11 至 2022-06，N=27,364）：
  - EBP（雇主场所）44.4%
  - Home（家）29.1%
  - Client（客户处）10.6%
  - FFH（亲友家）5.3%
  - Cowork（共享办公）5.3%
  - Public（公共场所）5.3%
  - 论文对「公共场所」的定义是「咖啡馆、图书馆、社区中心」
- **Figure 1(b)，按工作出行次数**（2022-11 至 2023-01，N=13,091）：
  - EBP 51.3%、Client 14.0%、Public 13.7%、FFH 12.9%、Cowork 8.1%
- 其他发现：
  - 2021-11 那一波里，超过 43% 的受访者上周在第三场所工作过（§4.1.1）
  - 30 岁以下的人用第三场所的比例约是 50 岁以上的 3 倍（Figure 3）
  - 城市居民、与室友同住的人用第三场所更多（Figure 4）
- MIT News 访谈给出的性别差异：男性约 40% 的远程工时在第三场所，女性约 30%。见 https://news.mit.edu/2022/jinhua-zhao-work-remote-0128【官方文档，MIT 新闻稿】

### B. Buffer State of Remote Work【官方文档，厂商调查】
- **2023 版**：https://buffer.com/state-of-remote-work/2023 ，小节「82% of remote workers are working from home」
  - 样本：N=3,000 远程工作者，全球（美国 19%、尼日利亚 7%、印度 7% 等），2022-10-10 至 2022-11-28
  - 问「What is your primary work location?」：家 82%、办公室 9%、共享办公 5%、咖啡馆 2%、其他 2%
  - 在家用哪个房间：独立办公空间 37%、卧室 21%、客厅 20%、在房间之间换来换去 14%
  - 45% 的人在一间多用途房间里工作（卧室、厨房或客厅）
- **2019 版**：https://buffer.com/state-of-remote-work-2019
  - 样本：N=2,471，美国占 48%，调查日期未写
  - 问「What location do you primarily work from?」：家 84%、共享办公 8%、咖啡馆 4%、图书馆 0.5%、其他 3%（含房车、酒店）
  - 问「second most common location」：咖啡馆 37%

### C. 在家时用哪个房间、什么噪声
- **Puglisi et al. 2021**，Frontiers in Built Environment，DOI 10.3389/fbuil.2021.688484【论文】。链接：https://www.frontiersin.org/journals/built-environment/articles/10.3389/fbuil.2021.688484/full
  - 样本：意大利（81% 在北部），N=1,934，2020-05 在线问卷，正文 Methodology 部分与 Table 1
  - 房间：独立房间 54.6%、共用房间（厨房、客厅等）43.1%、户外（阳台、花园）0.8%。摘要里写的户外是 2%，与正文不一致，原文未解释
  - 问 Q13「What is the main source of noise present during your remote working activity?」：25% 选「人发出的噪声」（说话、走动、打电话、听音乐）
  - 分房间看「成年人噪声」（Figure 2）：独立房间 22%，共用房间 30%
  - 交通、警笛各自是主源的比例都不到 15%（Discussion）
  - 孩子、宠物、家电、邻居的分项数值只在 Figure 2 图里，正文没写，我未转录
- **CraftJack**，经 ThinkRemote 转述，【论坛/博客，二手，原文未核】：https://thinkremote.com/remote-workers-work-from-their-bed-survey/
  - 家庭办公室 32%、卧室 31%、客厅 23%、厨房 9%、地下室 5%
  - 样本量、国家、问法都不详；文章 2021-08 发布
- **Call Center Studio（2020）**，【论坛/博客，二手，原文未核】：https://callcenterstudio.com/blog/home-office-blogs/
  - 客厅 45.9%、卧室 32.4%、车库或独立办公空间 16.6%、厨房 5%
  - 样本约 300 人

### D. 会议中被打断、听到什么噪声
- **FinanceBuzz**【官方文档，媒体自办调查】：https://financebuzz.com/survey-remote-work-lifestyle
  - 样本：N=500 名全职远程或部分远程工作者，2020-03 至 2020-04，国家未写（推断为美国）
  - 视频会议被谁打断过：宠物 47%、配偶或伴侣 32%、孩子 21%
  - 同一页「过去一年在哪里工作过」：
    - 酒店 69%、机场 61%、飞机 46%、火车 14%
    - 行驶中的车 42%、停着的车 39%
    - 咖啡馆 58%、餐厅 30%、酒吧 16%
    - 共享办公 29%
    - 海滩或泳池 21%、公园或户外 20%
- **Dimensional Research for Cisco Webex**，《The Rise of the Hybrid Workplace》，2020-10【官方文档，厂商委托】：https://www.cisco.com/c/dam/en/us/products/collateral/collaboration-endpoints/global-workforce-survey.pdf
  - 样本：N=1,569，全球
  - 第 7 页，问「When working from home, what are the biggest frustrations during video meetings?」：
    - 音频差 50%
    - 多人同时说话 47%
    - **其他与会者环境嘈杂 46%**
    - **自己这边有背景噪声 32%**
  - 第 6 页，在家开会用什么设备收音：笔记本 42%、头戴耳机 23%、手机 18%、耳塞 8%
  - 这一页支持「内置麦是主流」【官方文档】
- **Plantronics《Managing Noise》**，2017-10【官方文档，厂商】：https://m.softchoice.com/web/newsite/documents/amplify/Plantronics-Hardware-Managing-Noise.pdf
  - 样本：N=2,184，澳、中、法、德、瑞典、英、美 7 国；受访者都在 500 人以上的开放办公室工作，且每周至少去一次
  - 第 12 页，问「What are the most annoying noises you hear?」：
    - 通话时大声说话 29%
    - 同事在工位前交谈 23%
    - 手机提示音 13%
    - 私人电话 11%
    - 吸鼻子或咳嗽 9%
    - 敲键盘 8%
    - 吃东西 5%
  - 第 13–14 页：39% 说客户抱怨过听到自己这边的噪声。客户听到了什么：
    - 同事说话但听不清内容 27%
    - 同事说话且听得清 26%
    - 一般噪声 25%
    - 音乐 8%
    - **公共场所噪声（咖啡馆、机场休息室）7%**
    - 打字 7%
  - 第 7 页：29% 嫌吵时会离开工位，去公司内更安静的地方（如专注室）
- **IWG 2019**，经 FE News 转述【官方文档，厂商，二手，原文未核】：https://www.fenews.co.uk/skills/school-holidays-signal-stress-for-home-workers/
  - 英国 N>2,000，属于全球 15,000 人调查的一部分，2019-01
  - 家人或孩子要人关注 65%
  - 孩子、家人、宠物打扰工作电话 49%
  - 开着电视当「电子保姆」34%
- **Microsoft**（Teams 博客）只把「大声打字、狗叫」列为降噪功能的目标，不是调查数据【官方文档】：https://www.microsoft.com/en-us/microsoft-365/blog/?p=234182
- **Work Trend Index**：未查到噪声来源的数据。
- **Krisp**：未查到公开的噪声排名报告，只有营销文章列举「键盘、狗叫、哭闹」【论坛/博客】。

### E. 车里、在路上
- **Office Depot**，经 HR Review / ThinkRemote 转述【二手，原文未核】：https://www.hrreview.co.uk/hr-news/strategy-news/remote-working-means-some-employees-work-from-their-car/127554
  - 美国 N=2,000，2023-03-20 至 2023-03-23：约 1/3 在车里工作过
- **heycar / Censuswide**，英国 N=2,004，2020-09【二手，原文未核】，链接同上：
  - 10% 经常在车里工作
  - 23% 会进车里给客户打电话
- **比利时 Vias（2022）**【二手，原文未核】：https://www.vrt.be/vrtnws/en/2022/05/12/one-in-ten-taking-part-in-video-calls-behind-the-wheel
  - 过去一个月 10% 的比利时人边开车边开视频会议；样本量未写
- 以下【二手，原文未核】：
  - RAC（英国）：17–24 岁司机中 30% 承认开车时视频通话
  - PEUGEOT / YouGov（阿联酋、沙特）：超过 70% 开车时用过视频会议，厂商委托
  - Root Insurance（美国，2021，N=1,819）：54% 开完视频会后开车难以集中注意力
  - 这几项都是开车的人，不是乘客；`car-passenger` 没有乘客的数据（推断）
- **华盛顿特区 State of the Commute 2019**【官方文档，二手摘录】：火车通勤者中 58% 在路上处理工作，没拆出通话

### F. 咖啡馆、酒店
- **YouGov 英国（2024-05，N=5,195）**，经 startups.co.uk 转述【二手，原文未核】：https://startups.co.uk/news/coffee-shop-etiquette-video-calls/
  - 只有 8% 认为在咖啡馆开视频会可以接受
  - 38% 认为完全不可接受
  - 48% 认为戴耳机就可以
  - 三个数字之间口径不清，原文未核
- **Marriott Vacations / OnePoll（2023，N=2,000）**【二手】：度假时在哪里工作，酒店房间 25%、度假租房 23%
- **Mews / OnePoll（2024，N=2,000，美国）**【二手】：度假时想在酒店房间工作的占 69%
- **Owl Labs 2024**（美国 N=2,000，2024-07）【官方文档】：58% 在家庭办公室和共享办公之外的地方工作过。链接：https://owllabs.com/state-of-hybrid-work/2024

### G. 会议室、电话亭
- **Owl Labs 2025**（美国 N=2,000，2025-07）【官方文档】：https://owllabs.com/state-of-hybrid-work/2025
  - 67% 放弃过调试会议室视频设备，改用自己的笔记本
  - 75% 遇到过「听不清所有人」
  - 76% 遇到过回声或失真
  - 这几条支持「会议室里多人共用一台笔记本」场景存在，但不是地点占比（推断）
- **Owl Labs 2024**：23% 说办公室隐私更好就愿意回去，14% 说要更多会议室
- **PointGrab** 称电话亭在高峰日占用率 80%–95%【论坛/博客，厂商营销】
- 工位、会议室、电话亭三者各占多少：**未查到**

### H. 中国
- 腾讯会议、飞书、钉钉：**未查到**公开的开会地点或房间分布。
- 北大国发院与智联招聘《中国远程居家办公发展报告》（2022-04）只定性提到「居住环境嘈杂、空间狭小」，没给比例【二手，原文未核】：https://news.qq.com/rain/a/20220519A0BHRY00
- Plantronics 2017 的样本含中国，但没有单列中国的数据。

---

## 二、对应 14 个场景键

| 场景键 | 可用的最佳数字 | 来源类型 |
|---|---|---|
| `home-quiet` | 独立房间 54.6%（意大利）；独立办公空间 37% + 卧室 21%（Buffer 2023） | 论文 / 厂商 |
| `home-family` | 共用房间 43.1%（意大利）；客厅 20%–23%；被宠物打断 47%、伴侣 32%、孩子 21%；IWG 英国 49% 的电话被家人或宠物打扰 | 论文 / 厂商 / 二手 |
| `open-office` | SWAA 雇主场所工时 44.4%（未拆工位和会议室）；Plantronics 开放办公室最烦的噪声是通话大声说话 29% | 论文 / 厂商 |
| `phone-booth` | 未查到占比；29% 嫌吵时去公司内的安静处（Plantronics） | 厂商 |
| `meeting-room` | 未查到占比；Owl Labs 2025 中 67% 改用自己笔记本 | 厂商 |
| `cowork` | SWAA 工时 5.3%、出行 8.1%；Buffer 2023 主要地点 5% | 论文 / 厂商 |
| `cafe` | SWAA 公共场所（含咖啡馆）工时 5.3%、出行 13.7%；Buffer 主要地点 2%、次要地点 37%（2019）；FinanceBuzz 过去一年去过 58% | 论文 / 厂商 |
| `canteen` | 未查到；只有 FinanceBuzz「餐厅工作过」30% | 媒体调查 |
| `transit-hub` | 未查到占比；FinanceBuzz 机场 61%（过去一年去过）；Plantronics 客户听到咖啡馆或机场噪声 7% | 媒体调查 / 厂商 |
| `car-passenger` | 未查到乘客数据；在车里工作 1/3（Office Depot）、10%（heycar）；进车打客户电话 23% | 二手 |
| `rail-air-cabin` | 未查到占比；FinanceBuzz 飞机 46%、火车 14%；华盛顿特区火车通勤者 58% 在路上办公 | 媒体调查 / 二手 |
| `street-outdoor` | 未查到；FinanceBuzz 公园或户外 20%；意大利户外 0.8%–2% | 媒体调查 / 论文 |
| `hotel-room` | FinanceBuzz 酒店 69%（过去一年去过）；Marriott 度假时 25% | 媒体调查 / 二手 |
| `library-classroom` | 只有 Buffer 2019 图书馆作为主要地点 0.5%；SWAA 把图书馆并进「公共场所」；教室未查到 | 厂商 / 论文 |

**口径警告（推断）：**
- 「过去一年去过」（FinanceBuzz）和「工时占比」（SWAA）不能放在一起比；前者会把偶发地点放大很多倍。
- 所有数据都是「工作地点」，不是「开会地点」。
- 样本几乎都是欧美，大多在疫情期间采集。

## 三、没打开的原文
以下几处只看到二手转述，有二手结论的已在上文标注：
- Webex PDF（MDPI 与 ProQuest 返回 403）
- benefitspro 上的 OnePoll / Citrix 数据（孩子 24%、宠物 13%）
- Jabra Hybrid Ways of Working 2022 全文（第三场所用的是倍数比较，没给绝对数）
- 意大利研究 Figure 2 的分项数值

我用 `Read` 读了三份下载到本机的 PDF（Webex、Plantronics、arXiv），存在 `/Users/allenchen/.claude/projects/-Users-allenchen-xingan-chen-B-MyCreate-5-App-noise-canceling-scenes/d30acd32-697d-4346-9811-adea3e1dbdee/tool-results/`。本次没有写或改仓库里的任何文件。