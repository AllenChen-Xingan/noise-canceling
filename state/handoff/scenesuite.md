# 现状页 · scenesuite（批次 20261103-b11）

- **状态**：进行中
- **手上**：exp013 场景测试集：15 个地点的混合声音片段 + Mac 内置麦模拟 + 逐段声音标注
- **交代**：主目录 `_tmp/20261103-b11/scenesuite.md`
- **做到哪**（2026-11-03）：方案草稿与片段清单格式已写，等大总管说「可以」；格式已发 scorecard 对齐
- **下一步**：收到「可以」后先写设备链模块与参数文件（带测试），再写生成器

## 改了哪些文件
| 文件 | 改了什么 |
|---|---|
| `docs/experiments/exp013-scene-suite/README.md` | 方案草稿 |
| `docs/experiments/exp013-scene-suite/clip-manifest.md` | 片段清单格式草稿 v0 |

## 发现
- 本机可用的英文说话人只有 20 人（LibriSpeech test-clean 奇数位），另一半被 exp006 训练用过
- `vdn-eval` 的 `real` 只认 exp005 / exp003，要读本集得由 scorecard 加一种来源

## 待人拍板
1. **会议室怎么判**（近处 2–3 人都要留，和其他场景「只留我」相反）
   - A. 照样出，干净参考 = 近处所有人 —— 代价：现有模型大概率成绩差 ← 建议
   - B. 本批不出会议室 —— 代价：缺一个地点
   - 不回默认 A

## 转交（范围外，留给大总管）
- 第二批下载（DEMAND TCAR / TMETRO / PSTATION，约 350 MB）要大总管批

## 流水
- 读交代、共同规矩、手下协议 → 报到
- 读 021 catalog / coverage、022 simulation / devices、016 synth-standard、exp006 sim / recipes、exp008 cells、vdn-eval suite → 写方案草稿与清单格式
