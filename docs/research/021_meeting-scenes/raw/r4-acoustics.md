# 原始报告 · r4-acoustics（workflow wf_c790c606-7d6 · 2026-11-03，未经合成，供追溯）

**要点（5 行内）**
1. 有实测原文的场景：飞机客舱、高铁或火车站候车厅、食堂、开放办公、家里电视声。下文直接读了 Ozcan 2006、Wu & Kang 2020、Wang 等（天津食堂）、Hongisto & Keränen 2021、Pearsons 1977（美国环保署报告）的原文表格，数字可以直接用。
2. 家里电视声：在听者耳边平均 61 dB，人离电视平均 3 m（Pearsons 1977）。按距离推算，在笔记本麦处它和用户本人的声音大约一样响，SIR 约 0 dB【推断】。宠物和小孩出现在通话里，商业问卷给的比例是 29–69%。
3. 开放办公：全球已发表数据里，4 m 处语音声级在 40–60 dB，干扰距离 rD 在 2.1–21 m。由此推出邻座 4 m 时的 SIR 约 +1 ~ +21 dB【推断】。
4. 已有数字的核对：仓库 F3 引的「餐厅 59–80 dBA（Lebo）」原文仍未核到，只是二手转述。F3 里的「香港 73.9 dBA」在本轮查到的两篇香港原文里都没出现，判为未核实。Wu 2018 的拐点（59.3 dBA）和斜率（0.54）核对一致。
5. 普遍缺口：每个场景突发声音的频度（次/分）、笔记本麦处的 SIR 实测、机场大厅 T60 的同行评审实测，基本都是「未查到」。

---

# 第 4 路：14 个开会地点的声学事实

**通用说明**
- 背景声级大多是在**听者处**或**房间里**测的，不是在笔记本麦处测的，也不是针对用户自己的声音。
- 推 SIR 时，用户本人的声音统一按 Pearsons 1977 的数取：普通对话在 1 m 处 55 dB。人离麦 0.5 m 时，只算直达声约 61 dB【推断】。人离麦约 0.5 m 的依据见下面「跨场景」一节。

## 跨场景：人离笔记本麦的距离
- 【论文】Brascamp 2021，*J. Vision* 21(8):19，图 3A。这是线上研究，有效样本 247 人。自然坐姿下，眼睛到**笔记本**屏幕平均 **51.7 cm**，用外接屏时平均 61.5 cm。https://pmc.ncbi.nlm.nih.gov/articles/PMC8383908/
- 【论文】Rempel 等：用台式显示器，24 人，三档距离平均 52.4 / 73.0 / 85.3 cm。这不是笔记本的数据。https://pubmed.ncbi.nlm.nih.gov/17915601
- 【推断】MacBook 麦在屏幕和键盘附近，所以嘴到麦约 0.4–0.6 m。仓库 F7 的基点 0.5 m 有了间接依据。**嘴到麦本身的实测：未查到。**

## 跨场景：说话声随噪声抬高（Lombard 效应）
- 【论文·原文】Pearsons 1977（美国环保署报告 EPA-600/1-77-025）摘要：背景低于 48 dB 时，面对面说话在 1 m 处平均 55 dB；背景在 48–70 dB 时，噪声每升 1 dB，说话升约 0.6 dB，最高到 67 dB。https://www.nonoise.org/epa/Roll2/roll2doc16.pdf （PDF 第 5 页）
- 【论文·原文】Wu 2018，*Ear Hear* 39(2)。两段折线拟合：拐点在噪声 59.3 dBA（此时说话 66.0 dBA）；拐点以下斜率 0.34 dB/dB，以上 0.54 dB/dB。共 894 个听音场景，录音机挂在胸前。**和仓库 F3 一致。** https://pmc.ncbi.nlm.nih.gov/articles/PMC5824438

---

## home-quiet 书房 / 卧室
- **背景声级**
  - 【论文·原文】Wu 2018 图 5：在家时噪声中位数 53.6 dBA，SNR 中位数 9.9 dB。胸前录音，听者处测量。
  - 【论文·原文】Pearsons 1977 §VII-B（第 23–25 页）：郊区住宅室内外平均环境声 45 dB，市区 55 dB。在家里说话时，人们保持约 9–14 dB 的语音噪声比。
  - 【二手，原文未核】Smeds 2015：安静场景噪声中位数 41 dBA。https://pubmed.ncbi.nlm.nih.gov/25690777/
  - 【论文，搜索摘要】伦敦 49 户：室内 LAeq,16h 为 38.4 dB。https://www.euronoise2018.eu/docs/papers/480_Euronoise2018.pdf
- **T60（混响时间）**
  - 【论文·摘要】Díaz & Pedrero 2005，*Appl. Acoust.* 第 945–956 页：西班牙 11,687 个房间，其中 8,246 间卧室、3,211 间客厅。同样大小的卧室和客厅 T60 相近，频率越高 T60 越短。**具体秒数未取到。** https://manuscript.isc.ac/Inventory/10/1170777.htm
  - 【论文，二手】Burgess（英国声学学会会议论文）：英国客厅 0.5 s 的说法来自 1960 年代，现在实测更短。
  - 【官方文档，二手】住宅隔声测量把 T0 = 0.5 s 作为参考值。
- **旁人说话**：基本没有。
- **突发声音**：门外声音、自己敲键盘。**频度未查到。**
- **人离麦**：见「跨场景」，约 0.5 m【推断】。

## home-family 家里客厅
- **背景声级**：同 home-quiet，在家中位数 53.6 dBA（Wu 2018）。市区住宅 55 dB（Pearsons）。
- **T60**：同 home-quiet，具体秒数未取到。
- **电视 / 视频外放的人声**
  - 声级：【论文·原文】Pearsons 1977 第 25 页、图 7、图 8。
    - 麦克风放在观看者耳边，观看者自己调到喜欢的音量。
    - 电视人声平均 **61 dB**，分布约 49–70 dB，样本约 28 个（数图 7 的直方图）。
    - 观看者离电视平均 **3 m**。
    - 背景噪声每升 1 dB，人会把电视调大 0.7 dB（r = 0.58，背景范围 38–52 dB）。
  - 声级：【论文，搜索摘要】Brooks & Attenborough（英国声学学会）：英国电视收听声级范围 35–76 dBA。年份因扫描件识别不清未核。
  - 常见度：【官方文档】美国时间使用调查 2024：美国人平均每天看电视 2.6 小时。https://www.bls.gov/news.release/atus.nr0.htm
  - 常见度：【论文，二手，原文未核】Lapierre 等 2012，*Pediatrics*：1,454 户有 8 个月到 8 岁孩子的家庭，孩子每天暴露在背景电视里接近 4 小时。
  - **SIR【推断】**：用户在 0.5 m 外说话约 61 dB，电视在用户处约 61 dB，所以 SIR ≈ 0 dB。范围约 −9 ~ +12 dB。电视人声和用户本人声音一样响，是这个场景最难的地方。
- **小孩 / 宠物**
  - 宠物比例：【官方文档，行业协会】美国宠物用品协会 2024：44% 的美国家庭养狗，30% 养猫。【论坛/博客】中国城镇家庭养宠比例约 22–23.5%（统计网站、中国日报）。
  - 出现在通话里的比例：【论坛/博客，商业问卷】
    - CraftJack 2021（1,520 人）：69% 的人遇到过伴侣、孩子或宠物突然出现在视频通话里，其中宠物 43%，孩子 37%。
    - Citrix/OnePoll 2020（2,000 人）：29%。
    - IWG 2019（英国）：49% 的人说工作电话被孩子、家人或宠物打断过。
    - 这些问卷只问有没有出现，没有问声音。
  - 狗叫声级：【实测，咨询报告】单声狗叫 LA5 = 75 dB(A)，在 1 m 处；另一份报告 79 dB(A) LAeq，在 1 m 处。https://cdn.centralcoast.nsw.gov.au/sites/default/files/2023-07/addendum_to_noise_report_-_renl417-03f02_technical_memo_r0_1.pdf
  - 狗叫声级：【论文·摘要】Sales 1997：犬舍里 Leq 65–100 dB，峰值常到 125 dB。这是犬舍整体，不是家里。
  - 猫叫声级：**可信实测未查到。** 宠物网站说约 45 dB，没有出处，不建议用。
  - 小孩说话或哭闹的声级：**未查到。**
- **突发频度**：**未查到。**

## open-office 开放工位
- **背景声级**
  - 【论文，二手】英国声学学会 Leonard & Chilton 的综述表：20 个开放办公室 41–53 dBA，平均 47。分来源：Warnock & Chu 9 间，平均 47；Moreland 7 间，平均 45；Nilsson & Hallström 4 间，平均 50。同表呼叫中心 21 处平均 58 dBA。
  - 【论文，搜索摘要，原文未核】*Appl. Acoust.* 2021：43 间有人上班的办公室，LAeq 约 53.6 dB。
  - 【论文，二手】Chilton：背景高于约 45 dBA 后，背景每升 1 dB，说话升 0.55–0.65 dB。
- **ISO 3382-3 标准量**
  - 【论文·原文】Hongisto & Keränen，Euronoise 2021。https://www.sea-acustica.es/archivo/congresos/Madeira21/ID107.pdf
  - 表 3（全球文献里报过的范围）：
    - 语音空间衰减率 D2,S：1.5–12 dB
    - 4 m 处语音声级 Lp,A,S,4m：40–60 dB
    - 背景声级 Lp,A,B：26–54 dB
    - 干扰距离 rD：2.1–21 m
    - 舒适距离 rC：2.1–43 m
  - 表 1（芬兰图尔库一间 24 × 6 m、12 个工位的办公室，7–8 个测量团队）：D2,S = 3.7 dB，Lp,A,S,4m = 52.4–52.5 dB，Lp,A,B = 37.6–37.9 dB，rD = 14.7–15.3 m。
  - 【官方文档，二手】ISO 3382-3 附录 A 给的「良好」示例值：D2,S ≥ 7 dB，Lp,A,S,4m ≤ 48 dB，rD ≤ 5 m。https://www.association-of-noise-consultants.co.uk/wp-content/uploads/2016/06/B3-Workplace-wellbeing-wondorous-A-Parkin-Offices.pdf
  - 【论文·摘要】Haapakangas 等，21 间办公室：rD 每长 1 m，「高度受扰」的几率升 10–15%。
- **SIR【推断】**：邻座在 4 m 时，邻座声级约 40–60 dB，用户自己约 61 dB，所以 SIR ≈ +1 ~ +21 dB；图尔库那间约 +9 dB。邻座在 1.5–2 m 时，按 D2,S 再加 3–6 dB 的邻座声级，SIR 可能 ≤ 0 dB。
- **邻座典型距离**：**统计数据未查到。** 图尔库那间是两排面对面的工位，屏幕隔板高 117 / 147 cm，图里没标工位间距。
- **T60**：**普查数据未查到。** ISO 3382-3 不用 T60 评价开放办公。
- **突发频度**：**未查到。**

## phone-booth 电话亭 / 专注舱
- **隔声**
  - 【论文·摘要】Hongisto 等：按 ISO 23351-1 测了 11 款电话亭，语音声级降低量 DS,A 分级 A+ 到 D。实验室间比对时同一款亭子测出 27.2–30.3 dB。https://www.turkuamk.fi/julkaisu/acoustic-performance-of-eleven-commercial-phone-booths-according-to-iso-23351-1/
  - 【官方文档，厂商证书】BuzziNest：DS,A = 27.1 dB，B 级。【论坛/博客，厂商说法】Framery O：30.3 dB。
- **亭内背景**【推断】：亭外按 47–54 dBA 算，减 27–30 dB 后，外面的人声进到亭内约 20–27 dBA。亭内通风扇的声级：**未查到。**
- **亭内 T60**：**未查到。**
- **旁人说话**：隐约能听到，SIR 很高（> 25 dB）【推断】。

## meeting-room 会议室（多人共用一台 Mac）
- **背景声级**
  - 【官方文档，摘录】美国暖通空调学会手册第 49 章：会议室暖通背景目标 NC/RC 30，高要求的会议室 NC/RC 25。https://handbook.ashrae.org/Handbooks/A23/IP/A23_Ch49/a23_ch49_ip.aspx
  - **有人开会时的实测：未查到。**
- **T60**
  - 仓库 F6 已写会议室目标 0.5–0.7 s，本轮没有重复查。
  - 【论文，二手】格拉茨工大的混合会议研究：一间 150.8 m³ 的研讨室，按奥地利标准 ÖNORM B 8115-3 目标 0.5 s ± 20%。
  - **实测普查：未查到。**
- **旁人说话**：近处有几个人都要保留。几个人离笔记本 0.5–2 m 的实测：**未查到。**【推断】直达声电平差在 0 ~ −12 dB。

## cowork 共享办公
- **本身的实测：未查到。**
- 【推断】可以按「开放办公（41–54 dBA）+ 咖啡馆（咖啡机、走动）」混着取，没有出处。

## cafe 咖啡馆
- **背景声级**
  - 【二手，原文未核】Lebo 等 1994，*West J Med* 161:45–49。PubMed 摘要只确认测了旧金山湾区 27 家餐厅，测的项目有最小 / 峰值 / 平均声级、录音、现场言语识别。59–80 dBA、峰值 87 dBA 来自 2015 年《职业与环境卫生杂志》的转述。全文 PDF 被浏览器验证页挡住，没读到。**F3 仍是「未核原文」。** https://pmc.ncbi.nlm.nih.gov/articles/PMC1011367/
  - 【论文·摘要】*Int J Audiol* 2016;55(3):184–8：波特兰 6 家独立咖啡馆，8 名咖啡师各戴个人剂量计 3 个班次，Leq 71–83 dBA。这是在咖啡师身上测的，不是在座位上。https://informahealthcare.com/doi/abs/10.3109/14992027.2015.1124295
  - 【论坛/博客，众包 App】SoundPrint：星巴克平均 72 dBA，Dunkin' 69 dBA，约 37% 的店高于 74 dBA。用户自己上传，测法不统一。
- **SNR**：仓库 F2 已有，按 Wu 的关系外推约 +7 ~ −3 dB【推断】。
- **T60、邻桌 SIR、磨豆机频度：未查到。**

## canteen 餐厅 / 食堂
- **天津两所高校食堂**
  - 【论文·原文】Wang, Kong, Yao, Kang, Yuan，《中国大型高校食堂的人群噪声与发声功率级》，UCL 存档，*Appl. Acoust.* 2021。https://discovery.ucl.ac.uk/10132069/1/Crowd%20Noise%20and%20Vocal%20power%20level%20in%20Large%20College%20Canteens%20in%20China.pdf
  - 表 1：食堂 H 为 49.8 × 18.6 × 3.4 m、3,149 m³、680 座，T60 = 1.08 s；食堂 T 为 8,418 m³、716 座，T60 = 1.04 s。T60 是 500 / 1k / 2k Hz 三个倍频程的平均，用气球爆破测，测时人很少。
  - 用餐时段噪声 61–73 dBA。每 5 分钟一次、每点测 1 分钟，连测 3 小时。
  - 引言引前人：中国高校食堂常见 70–80 dBA，每餐持续约 2 小时。
- **丹麦大学食堂**
  - 【论文，二手：会议幻灯片摘录】Odeon，BNAM 2012：3 个大厅晚餐时 LAeq 82–87 dB(A)。各厅 T60：A 厅 2.5 s、87 dB，B 厅 0.8 s、82 dB，C 厅 1.0 s、83 dB。https://odeon.dk/pdf/BNAM%202012%20-%20Noise%20in%20eating%20establishments.pdf
  - 【论文·摘要，二手】*Acta Acustica* 2023：学生食堂人多时 LA95 约 69 dB，超过约 150 人后不再上升。
- **香港餐厅**
  - 【论文·摘要】To & Chung 2014，*Noise & Health* 16(73)。Leq,1h：中餐厅 67.6–79.3 dBA，快餐 69.1–79.1 dBA，西餐 66.7–82.6 dBA。https://research.mpu.edu.mo/en/publications/noise-in-restaurants-levels-and-mathematical-model/
  - **仓库 F3 的「香港 73.9 dBA」在这篇摘要和 PMC3744581（香港 23 家餐厅，服务区 75.9 ± 5.6 dB，表 2）里都没出现，判为未核实。**

## transit-hub 机场 / 车站候车厅
- **哈尔滨大型火车站**
  - 【论文·原文】Wu, Kang 等 2020，*Appl. Acoust.* 160（DOI 10.1016/j.apacoust.2019.107137）。https://discovery.ucl.ac.uk/10090746/1/Wu%2C%20Kang%20Acoustic_comfort_in_large_railway_station_final.pdf
  - 车站 310 × 190 m，容积 1.2 × 10⁶ m³。候车区（表 2）容积 11,100 m³、162 × 68 m。
  - 背景声级：繁忙时段 8:00–18:00 测，每点 5 分钟。T60：夜里停运后测，按 T20 外推。
  - 表 4 各区实测：

    | 区域 | Leq | T60 |
    |---|---|---|
    | 候车座位区 | 60.8–66.4 dBA | 8.63–9.98 s |
    | 售票厅 | 67.7–68.4 dBA | 4.98–5.39 s |
    | 安检 | 69.8–70.8 dBA | 2.97–3.15 s |
    | 检票口 | 68.9–73.8 dBA | 3.28–4.16 s |
    | 站内餐厅 | 75.1 dBA | 1.58 s |
    | 商店 | 58.7 dBA | 4.62 s |

  - T60 超过 4.5 s 时，旅客自己能感到回声。
- 【论文·摘要，二手】华南理工学报 2014：某高架高铁站超大候车厅，中频 T60 = 9.21 s。【论文·摘要】那不勒斯 4 个地铁站中频 T60 > 3 s。
- **机场**
  - 【论坛/博客，厂商案例】卡尔加里机场装吸声顶棚后 T60 约 6 s → 约 2 s；Tweed-New Haven 机场处理后 1.8 s。
  - 【官方文档】美国机场合作研究计划报告 175：广播区建议 T60 ≤ 1.5 s，关键区 < 1.1 s。https://www.nap.edu/read/24839/chapter/24
  - 【论文·摘要】海口美兰机场指廊候机区（*Buildings* 2025, 15(3):426）：建议广播 SNR 10 dB(A)、STI ≥ 0.45。**T60 秒数没取到（网站返回 403）。**
  - **机场大厅 T60 和背景 Leq 的同行评审实测：未查到。**
- **广播频度：未查到。**

## car-passenger 车内（乘客）
- 【论文·原文】Kendra 等 2015，WASET 国际环境工程期刊第 9 卷。3 辆车各测 2 次，满载 5 人。表 I 结果：
  - 匀速 50 km/h：65.0–66.7 dB
  - 50 → 90 km/h 加速：67.2–67.8 dB
  - 城市工况：64.2–65.0 dB
  - 瞬时最大 76.1 dB
  - 文中只写「dB」，没写是否 A 计权。https://zenodo.org/records/1109657/files/10002764.pdf
- 【实测，媒体评测】汽车之家：120 km/h、驾驶员耳边、关空调和音响。比亚迪秦 Pro DM 68.5 / 69.0 dB，奥迪 A8L 60.2 dB。各车平台不完全一致。https://www.autohome.com.cn/advice/202001/963614.html
- 【论文·摘要】某中国研究：车速每升 10 km/h，车内约升 4 dB(A)。
- 【论文·原文】Wu 2018 图 5：「交通」场景（主要在车里）SNR 中位数 5.6 dB。
- **T60**：仓库 F6 取 0.08 s；**本轮没有查到新的实测。**
- **司机或导航说话的 SIR、转向灯频度：未查到。**

## rail-air-cabin 高铁 / 飞机客舱
- **飞机**
  - 【论文·原文】Ozcan & Nemlioglu 2006，*Canadian Acoustics* 34(4)。2 架 A321、1,000 km 航线、2 类声级计。https://jcaa.caa-aca.ca/index.php/jcaa/article/view/1854
  - 表 2，巡航持续噪声：两个航班分别测 10 / 16 次，平均 79.4 / 78.1 dB(A)，最小 74 / 75，最大 82。
  - 舱内位置：前舱 76、中舱 75、后舱 78.5 dB(A)。靠窗 82 dB(A)，比中间和靠过道（78）高约 4 dB。
  - 表 3，突发声音：机组广播、提示音 82–88 dB(A)。航班 2 巡航段记了 2 次突发（88、82）【实测，样本极小】。
- **火车 / 飞机说话**
  - 【论文·原文】Pearsons 1977 §VII-E（第 29 页）：旧金山湾区捷运和普通客机里，环境声平均 77 dB，说话平均 75 dB（SNR 约 −2 dB），说话人和听者平均相距 **0.4 m**。
- **高铁**
  - 【论文，二手，原文 PDF 404】汉阳大学，Euronoise 2015：中间车厢座位，有砟轨道，100 / 200 / 300 km/h 时 60 / 63 / 66 dBA。
  - 【论文·摘要，二手】国内某 CRH 动车组 VIP 车厢，350 km/h，加阻尼器前后 75.1 → 71.3 dBA。
  - 【论文·摘要】浙江大学：CRH 车内噪声以中低频为主，300–350 km/h 之间音质稳定。
- **T60、邻座 SIR**
  - 【推断】Pearsons 给的说话距离 0.4 m 和 SNR −2 dB，说明邻座说话在用户处和舱噪差不多响。
  - **T60 未查到。**

## street-outdoor 街边 / 户外
- 【论文·摘要】德黑兰市中心 2006：路边 78.5、人行道 73.3、店门口 68.7 dBA，总平均 74.7 dBA，86.6% 的点超过 65 dBA。https://ijehse.tums.ac.ir/index.php/jehse/article/view/99
- 【官方文档】辛辛那提有轨电车环评附录：城市社区声级一般 45–75 dBA，75 dBA 约是繁忙道路边人行道的水平。
- 【论文·原文】Pearsons 1977 第 25 页：在家门外说话时，人们只保持 5–8 dB 的语音噪声比。Wu 2018：户外 SNR 中位数 9.7 dB。
- **T60**：户外基本没有混响【推断】。
- **风噪、喇叭频度：未查到。**

## hotel-room 酒店房间
- 【论坛/博客，行业杂志】客房环境声典型约 28–32 dBA。https://www.constructioncanada.net/the-challenge-of-guest-room-acoustics-in-hotels/print
- 【论文，学生论文】某机场希尔顿：客房设计目标 RC 25–35。【论坛/博客，厂商】夜间 ≤ 30 dB(A)。
- **背景主要是空调或风机盘管：实测未查到。T60 未查到**；可借住宅约 0.5 s 的参考值【推断】。
- **走廊声频度：未查到。**

## library-classroom 教室 / 图书馆
- **教室**
  - 【论文，二手】Shield & Dockrell 2004：英国 30 间空教室平均 47 dBA；上课时按 6 类活动分，56–77 dB LAeq。
  - 【官方文档，二手】美国教室声学标准 ANSI S12.60：空教室背景 ≤ 35 dBA，核心教学空间 T60 ≤ 0.6 s。
  - 【论文·原文】Pearsons 1977 摘要：教师讲课在 1 m 处 67–78 dB。
- **图书馆**
  - 【论文，搜索摘要】葡萄牙大学阅览室：500–2000 Hz 平均 T60 1.3 s；空场、关暖通时 32 dB(A)；平时比背景高约 11 dB(A)。https://web.fe.up.pt/%7Ecarvalho/icsv17.pdf
  - 【论文·摘要】印尼某校园图书馆：7 个点测 3 天，一层平均 54.57 dB，开放式的二层平均 61.98 dB。
  - 【论文·摘要】马来西亚某大学图书馆：32.74 dBA。
  - 【论文·摘要】都灵中央图书馆：约 160,000 m³，中频 T60 约 5 s。
- **旁人轻声交谈的声级和 SIR：未查到。**

---

## 和仓库已有数字的核对
- **F2 / F3，Wu 2018：一致。** 59.3 dBA 拐点、0.54 斜率已核原文。还补到：拐点以下斜率 0.34；在家时噪声 53.6 dBA、SNR 9.9 dB；交通场景 SNR 5.6 dB。
- **F3，Lebo 59–80 dBA：仍是二手。** PMC 摘要只能确认测了 27 家餐厅。
- **F3，香港 73.9 dBA：未核实。** To & Chung 2014 只给范围 66.7–82.6；PMC3744581 里没有这个数。
- **F7，人离麦距离：新增一条间接依据。** Brascamp 2021：笔记本眼屏距离 51.7 cm。
- **F6，T60：新增几档实测。** 车站候车区 8.6–10 s；食堂 1.04–1.08 s；阅览室 1.3 s（二手）。仓库网格最高只到 1.3 s，transit-hub 场景远超这个上限。
- **F4，SIR：新增两条推断依据。** 家里电视场景约 0 dB；开放办公邻座 4 m 时 +1 ~ +21 dB。

## 主要来源
- Ozcan & Nemlioglu 2006：https://jcaa.caa-aca.ca/index.php/jcaa/article/view/1854
- Wu, Kang 等 2020：https://discovery.ucl.ac.uk/10090746/1/Wu%2C%20Kang%20Acoustic_comfort_in_large_railway_station_final.pdf
- Wang 等（天津食堂）：https://discovery.ucl.ac.uk/10132069/1/Crowd%20Noise%20and%20Vocal%20power%20level%20in%20Large%20College%20Canteens%20in%20China.pdf
- Hongisto & Keränen 2021：https://www.sea-acustica.es/archivo/congresos/Madeira21/ID107.pdf
- Pearsons 等 1977：https://www.nonoise.org/epa/Roll2/roll2doc16.pdf
- Wu 2018：https://pmc.ncbi.nlm.nih.gov/articles/PMC5824438
- Brascamp 2021：https://pmc.ncbi.nlm.nih.gov/articles/PMC8383908/
- Kendra 等 2015：https://zenodo.org/records/1109657/files/10002764.pdf
- Lebo 1994：https://pmc.ncbi.nlm.nih.gov/articles/PMC1011367/
- To & Chung 2014：https://research.mpu.edu.mo/en/publications/noise-in-restaurants-levels-and-mathematical-model/
- 香港餐厅工人研究：https://pmc.ncbi.nlm.nih.gov/articles/PMC3744581/
- 美国时间使用调查：https://www.bls.gov/news.release/atus.nr0.htm
- 美国机场合作研究计划报告 175：https://www.nap.edu/read/24839/chapter/24

本轮只做了网络检索和阅读；PDF 用 `Read` 或 `pdftotext` 读到标准输出，没有写任何文件。