# CHiME-Home（DCASE 2016 Task 4 家庭音频标注）

- **名称**：CHiME-Home domestic audio dataset（CHiME-Home）
- **链接**：https://archive.org/details/chime-home
- **许可**：archive.org 许可字段「Attribution-Noncommercial-Share Alike 3.0」（CC BY-NC-SA 3.0）→ 结论：只评测【官方文档】
- **大小**：409.2 min ≈ 6.8 h（DCASE 数据目录；DCASE 2016 任务页引论文「based on 6.8 hours of domestic environment audio recordings」）；chime_home.tar.gz 3.9 GB；4 s 片段共 6137 段（数据目录写 6138 个文件），其中高一致性标注段开发 1946 + 评测 816 = 2762 段【官方文档】
- **场景键**：home-family
- **设备键**：non-laptop-unknown
- **笔记本段**：未查到（DCASE 目录只写「Uncontrolled」，设备未写；DCASE 2016 任务页引的 CHiME 语料摘要提到人头躯干模拟器在家中录背景，是否即本集来源未写明）；无逐条设备标签
- **干净参考**：没有
- **转写**：没有（只有 7 类事件标签：儿童语音、成年男声、成年女声、电视 / 游戏、敲击、宽带噪声、其他）
- **语种**：英语（英国家庭）【推断】
- **语体**：家庭日常自发说话（含儿童）+ 电视声；有无笑声、附和未单独标注
- **获取方式**：直接下载（archive.org tar.gz）
- **与 D62 的关系**：真实录音（家里，无目标说话人参考）。若开发阶段拿来评测：按标签挑「电视 / 游戏」「儿童语音」段当「家里背景」干扰噪声，约 1181 段电视 + 1214 段儿童（各 4 s，即各约 1.3 h）；全包 3.9 GB【论坛 / 博客，个人说法：分类段数来自 DagsHub README，未在官方页核到】【推断：时长】
- **出处**：https://archive.org/details/chime-home【官方文档】; https://dcase-repo.github.io/dcase_datalist/datasets/sounds/chime_home.html【官方文档】; https://dcase.community/challenge2016/task-audio-tagging【官方文档】

## 备注
- 录音来源是 Sheffield CHiME 项目的家庭录音，标注于 2015 年 QMUL 完成。
- 采样率 DCASE 目录同时写了 16 kHz 和 48 kHz，未核实。
- 怀疑派复核：删去「archive.org 宣称来自约 10 h」——archive.org 页面没有时长，官方口径是 6.8 h；段数 6137 / 2762 改由 DCASE 2016 任务页（官方）支撑；笔记本段补一句来源不明。许可 CC BY-NC-SA 3.0 已在 archive.org 核实。
