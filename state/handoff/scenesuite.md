# 现状页 · scenesuite（批次 20261103-b11）

- **状态**：进行中
- **手上**：exp013 场景测试集：15 个地点的混合声音片段 + Mac 内置麦模拟 + 逐段声音标注
- **交代**：主目录 `_tmp/20261103-b11/scenesuite.md`
- **做到哪**（2026-11-03）：生成器、设备链、参数文件、39 条测试已写完并全过；已申请名额跑第一版 build（480 段）
- **下一步**：拿到名额 → 出 build → 生成 `src/eval/suite/scenes.json` → 自审 + codex 评审

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

## 转交（范围外，留给大总管）
- `src/eval/suite/scenes.json` 进来时，vdn-schema 的 DATA_FILES 要加一行（scorecard 说由它或大总管加）

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
