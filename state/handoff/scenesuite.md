# 现状页 · scenesuite（批次 20261103-b11）

- **状态**：进行中
- **手上**：exp013 场景测试集：15 个地点的混合声音片段 + Mac 内置麦模拟 + 逐段声音标注
- **交代**：主目录 `_tmp/20261103-b11/scenesuite.md`
- **做到哪**（2026-11-03）：人试听第一版后指出人声与底噪混响不一致；修法方案 `docs/experiments/exp013-scene-suite/reverb-plan.md` 已报大总管，等「可以」与下载档。codex 评审按大总管要求推迟到第三版
- **下一步**：拿到许可 → 下载实测 RIR → 改成「一段 = 一个房间」→ 第三版 build → 自审 + codex

## 改了哪些文件
| 文件 | 改了什么 |
|---|---|
| `docs/experiments/exp013-scene-suite/README.md` | 方案 + 第九节实现偏差 |
| `docs/experiments/exp013-scene-suite/clip-manifest.md` | 片段清单格式 v1.1（与 scorecard 的 schema 一致）|
| `docs/experiments/exp013-scene-suite/device_chain.{py,json}` | 022 九步设备链与参数正本 |
| `docs/experiments/exp013-scene-suite/{scenes,sounds}.json` | 地点组法、声音类别正本 |
| `docs/experiments/exp013-scene-suite/*.py` | 生成器（dsp · rooms · pools · synth_sounds · tracks · compose · labels · manifest · leak · build · tasks · fetch）|
| `docs/experiments/exp013-scene-suite/tests/` | 39 条测试 |
| `docs/experiments/exp013-scene-suite/materials.json` | 下载素材登记（URL · 哈希 · 许可）|

## 发现
- 本机可用的英文说话人只有 20 人（LibriSpeech test-clean 奇数位），另一半被 exp006 训练用过；中文只有 2 人有许可记录
- 两种电平算法（gated / P.56）对稀疏人声本来就差最多 1.5 dB；「≤ 1 dB」改成两层核对，见 README 第九节第 2 条
- 自己写的 P.56 曾取错门限，一段 50 ms 响声会把电平抬高约 14 dB；已修并加回归测试
- 合成舱噪曾有亚音频漂移，峰值到 5 倍满幅；已修并加峰值因数测试

## 待人拍板
1. **会议室怎么判**（近处 2–3 人都要留，和其他场景「只留我」相反）
   - A. 照样出，干净参考 = 近处所有人 —— 代价：现有模型大概率成绩差 ← 建议（大总管已按 A 先做）
   - B. 本批不出会议室 —— 代价：缺一个地点
   - 不回默认 A
2. **哪些声音合成得可能不像**：舱噪全合成、风噪、咖啡机蒸汽、只有一首背景音乐、电视外放带轻微失真（详见 README 第六节第 2 题）
   - A. 成绩单上先标「合成，待真实录音校」，这些声音的结论只作方向参考 ← 建议
   - B. 不标
   - 不回默认 A；人在桌面试听第一版 b22d 的 11 段，反馈来了按反馈改

3. **实测 RIR 下载档**（混响修法，见 reverb-plan.md 第四节）
   - A. MIT + ACE + McVAMPIRE，约 685 MB，超 500 MB 线 —— 6 个地点人声与底噪同房间，车内也是 ← 建议
   - B. MIT + ACE，约 429 MB —— 车内不成对
   - C. 只下 MIT 12 MB —— 全部非成对，只靠 T60 区间约束
   - 不回默认 B
4. **混响一致性检查的阈值**：RIR 的 T60 落在底噪名义区间 ×[0.8, 1.25] 内算一致（文献里 T60 人耳可辨差 3.3%–26%，名义区间本身是推断）
   - 不回默认用它；代价是差 20% 以内的组合会被放过

## 转交（范围外，留给大总管）
- `src/eval/suite/scenes.json` 进来时，vdn-schema 的 DATA_FILES 要加一行（大总管已交给 scorecard）
- 第一版 build b22d 的音频（3.2 GB）留不留，等人试听完由大总管定
- exp006 场景分类器不能在本集上评（同池），已告诉 scorecard 与大总管

## 流水
- 读交代、共同规矩、手下协议 → 报到
- 读 021 / 022 / 016、exp006 / exp008、vdn-eval → 写方案草稿与清单格式
- scorecard 回复清单 v0 → 改到 v1
- 大总管批方案与两批下载 → fetch.py 下载 ESC-50 168 段（71 MB）+ DEMAND TCAR / TMETRO / PSTATION（376 MB），登记 materials.json
- 写设备链与 022 文档交叉校验（11 条测试）
- 写生成器全套；冒烟发现并修：P.56 门限、T60 量法、广播提示音算成人声、合成噪声亚音频、峰值超满幅
- 草稿目录里跑过一次完整规模冒烟（17 s，未申请名额，已向大总管补报）
- scorecard 导出 schema → 校验器改为按 schema 递归查 + 4 条额外检查；请 scorecard 加 4 个字段，已加
- 申请名额跑第一版 build
- 第一版 build 20261103214702-b22d（480 段，18 s）→ 39 条测试全过；名额归还
- 三路自审（信号 · 防泄漏与契约 · 忠于 021/022）→ 修：owner_dry 对齐与电平、补 7 种声音、风噪绕过方位衰减、有规律声按整段功率、方位按实际位置、对照 exp006 test/seq 报告、校验器支持 anyOf、manifest 记 schema 哈希与库版本、缺素材默认拒跑、加强负对照
- 大总管转来：任务加 external-mic 对照 → 60 条
- 第二版 build 20261103215813-c025（480 段，17 s）→ 40 条测试全过；名额归还
- scorecard 定：任务极性照写 suppress / preserve，由它映射
- codex 第 1 轮开跑
- 人试听第一版 b22d 反馈混响不一致 → 停 codex（无结果）→ 两路调研（实测 RIR 库 · 盲混响估计）→ 写 reverb-plan.md 报大总管
