# 现状页 · 手下 evalcmp（批次 20261116-b19，issue #155）

状态：进行中

## 手上
管 3 个实体：评测器（`src/eval`）· 评测参考轨（exp013 生成器的参考输出）· 成绩单（规约 `specs/rubric/scorecard.md`）。
1. 配对比较子命令 `vdn-eval compare`
2. 早期回声参考轨 `owner_early.wav` + 成绩单旁列
3. 自审 + codex，连续两轮没有 P0/P1

## 做到哪
- 配对比较、早期参考轨（生成端 + 成绩单旁列）、规约第十 / 十一节都写完了，单测与集成测试全过（实测，见流水）
- 下一步：申请名额跑新加的两个测试和 clippy；在已有的真实 clips.jsonl 上冒烟 compare；出新 build（≤ 5 段冒烟，全量要不要本机出待大总管定）；自审 + codex

## 正本与交叉校验（CLAUDE.md 五.5）
| 东西 | 正本 | 第二处表示 · 校验 |
|---|---|---|
| 早期混响毫秒数 100 | `docs/experiments/exp013-scene-suite/rooms.py` `EARLY_REVERB_MS` | traindata 混音配方 import 它（对拍测试待 traindata 给路径） |
| 有早期参考旁列的项 | `standard::EARLY_REF` | 按行为核对（`early_tests.rs`：换参考后读数变了的项必须正好是它）；规约第十一节表（`tests/scorecard_compare_doc.rs`） |
| 片段清单 `owner_early` 字段 | `scene.rs` `Tracks` | `specs/schema/eval-scene-clip.schema.json` 由 vdn-schema 导出，`--check` 比对 |
| compare 的三态名 · SC-AGG 参数 · 否决逐段判法 | `compare.rs` · `thresholds/table.rs` | 规约第十节三张表（`tests/scorecard_compare_doc.rs`） |

## 流水
- 11-16 报到；100 ms 正本与 trainer、traindata 商定放 `rooms.py`（大总管同意，要求旧 build 行为不变）
- 11-16 `rooms.py` 加 `EARLY_REVERB_MS` 与 `early_rir`；`compose.py` 加 `point_early`（不抽随机数）与 `owner_early` 轨；`manifest.AUDIO_KEYS` 加一项
- 11-16 实测：新代码重生成 build 17f5 的 3 段（cafe-c00-mba · meeting-room-s01-mba · street-outdoor-c00-ext），各轨样本哈希与旧清单逐个相同
- 11-16 `tests/test_early.py` 9 条过；旧 build 仓内 40 个文件哈希钉在 `tests/old_builds.sha256`
- 11-16 Rust：`scene.rs` 加可选 `owner_early`；`scorecard/early.rs` 旁列；`compare.rs` · `compare_out.rs` · `main.rs` 接 `compare`
- 11-16 名额内：scorecard 单测 65/65；vdn-schema 重导 schema（+7 −0，`--check` 一致）；external_e2e 12 · scorecard_e2e 11 · scorecard_standard 5 · scorecard_compare_doc 2 全过；exp013 Python 自检 77 条全过
- 11-16 按行为测出有早期参考旁列的项 10 个：V-ONSET V-WEAK V-PUMPV V-CLICK V-ESTOI V-TIMBRE V-SPLVL V-TSOS V-SWALLOW V-PSE-OVL
- 11-16 加 e2e `early_reference_side_column` 与 compare 真实打分压低 6 dB 测试（待名额跑）

## 待人拍板
（暂无）

## 转交（范围外的发现）
- 吃字与两个评分模型（QA 层）的早期参考旁列本批没算：要换参考另跑模型，适合上云时一起做
