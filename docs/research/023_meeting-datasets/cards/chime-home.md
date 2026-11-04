# CHiME-Home（DCASE 2016 Task 4 家庭音频标注）

- **名称**：CHiME-Home domestic audio dataset（CHiME-Home）
- **链接**：https://archive.org/details/chime-home
- **许可**：archive.org 许可字段「Attribution-Noncommercial-Share Alike 3.0」（CC BY-NC-SA 3.0）→ 结论：只评测【官方文档】
- **大小**：约 6.8 h（409.2 min，DCASE 数据目录）；archive.org 宣称来自约 10 h 家庭录音；chime_home.tar.gz 约 3.9 GB；4 s 片段，raw 版 6137 段 / refine 版 2762 段【官方文档】【论坛 / 博客，个人说法：DagsHub README 给的段数】
- **场景键**：home
- **设备键**：unknown
- **笔记本段**：未查到（DCASE 目录只写「Uncontrolled」，设备未写）；无逐条设备标签
- **干净参考**：没有
- **转写**：没有（只有 7 类事件标签：儿童语音、成年男声、成年女声、电视 / 游戏、敲击、宽带噪声、其他）
- **语种**：英语（英国家庭）【推断】
- **语体**：家庭日常自发说话（含儿童）+ 电视声；有无笑声、附和未单独标注
- **获取方式**：直接下载（archive.org tar.gz）
- **与 D62 的关系**：真实录音（家里，无目标说话人参考）。若开发阶段拿来评测：按标签挑「电视 / 游戏」「儿童语音」段当「家里背景」干扰噪声，约 1181 段电视 + 1214 段儿童（各 4 s，即各约 1.3 h）；全包 3.9 GB【论坛 / 博客，个人说法：段数】【推断：时长】
- **出处**：https://archive.org/details/chime-home【官方文档】; https://dcase-repo.github.io/dcase_datalist/datasets/sounds/chime_home.html【官方文档】; https://dcase.community/challenge2016/task-audio-tagging【官方文档】

## 备注
- 录音来源是 Sheffield CHiME 项目的家庭录音，标注于 2015 年 QMUL 完成。
- 采样率 DCASE 目录同时写了 16 kHz 和 48 kHz，未核实。
