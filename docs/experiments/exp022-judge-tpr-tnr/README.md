# exp022 · 评估器对人耳的真阳性率 / 真阴性率

属调研 031（`docs/research/031_eval-judge-audit/`），issue #142。手下 evalaudit，2026-11-13。

## 问什么
成绩单的过关线（听感 Δ、吃字）判「过 / 不过」时，和人耳差多少？分开报：
- 真阳性率：人说不过的里面，评估器也判不过（抓到坏的）
- 真阴性率：人说过的里面，评估器也判过（不冤枉好的）

## 数据（全部已在仓里，不重跑模型，不要名额）
- 人耳：exp003 build f8c3，12 屏 × 3 个模型 = 36 条，1 名听者，会话打折
- 听感分：`exp009-more-ears/results/exp003_predictors.json`（作者实际听到的那份音频）
- 识别：`exp003-offline-ab/scores.json`
- 过关线：运行时从正本 `src/eval/src/thresholds/table.rs` 读；读不到就报错

## 怎么跑
```
python3 docs/experiments/exp022-judge-tpr-tnr/tpr_tnr.py
python3 -m unittest discover -s docs/experiments/exp022-judge-tpr-tnr -p 'test_*.py'
```
只用 Python 标准库，几秒跑完。输出 `results/tpr_tnr.md`（表）、`results/tpr_tnr.json`（逐条判定）。

## 结论
见 `results/tpr_tnr.md`，解读见 `docs/research/031_eval-judge-audit/audit.md` 第 1、2 条。样本小、只有一名听者，只能说明方向。
