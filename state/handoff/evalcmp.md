# 现状页 · 手下 evalcmp（批次 20261116-b19，issue #155）

状态：进行中

## 手上
管 3 个实体：评测器（`src/eval`）· 评测参考轨（exp013 生成器的参考输出）· 成绩单（规约 `specs/rubric/scorecard.md`）。
1. 配对比较子命令 `vdn-eval compare`
2. 早期回声参考轨 `owner_early.wav` + 成绩单旁列
3. 自审 + codex，连续两轮没有 P0/P1

## 做到哪
- 配对比较、早期参考轨（生成端 + 成绩单旁列）、规约第十 / 十一节都写完了，单测与集成测试全过（实测，见流水）
- 下一步：自审 + codex 评审；新 build 交给云上跑

## 正本与交叉校验（CLAUDE.md 五.5）
| 东西 | 正本 | 第二处表示 · 校验 |
|---|---|---|
| 早期混响毫秒数 100 | `docs/experiments/exp013-scene-suite/rooms.py` `EARLY_REVERB_MS` | traindata `exp025-gtcrn-train/data/mix.py` import 它与 `early_rir`；对拍测试 `test_mix_loader.py`（traindata 写） |
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
- 11-16 加 e2e `early_reference_side_column` 与 compare 真实打分压低 6 dB 测试
- 11-24 电脑休眠中断后接着做（人指示）。JSON 往返浮点差最后一位 → 改相对容差 1e-12，e2e 过；clippy -D warnings 过；xtask tidy 过；scorecard 单测 68/68
- 11-24 大总管 noise-canceling-e1 已不在会话列表，名额无法申请；按人「接着做」自行跑，记在此
- 11-24 compare 冒烟（只读分数）：a = dfn3-ll、b = dpdfnet2，旧 build 480 段全部配上；涨 10 · 退 8 · 不确定 5。延迟 V-LAT 判退（+46.25 ms）、b 新增否决 480 段。发现 Markdown 里否决段列表太长 → 最多列 10 个，全部在 compare.json
- 11-24 exp013 冒烟 build（4 段，写在会话临时目录，不进仓）：泄漏、电平验收都过，每段有 owner_early.wav；取 1 行做 Rust 解析夹具 `src/eval/tests/fixtures/scene-clip-early-example.jsonl`
- 11-24 traindata 已在 `exp025-gtcrn-train/data/mix.py:30` import `EARLY_REVERB_MS, early_rir`，并有测试和正本逐样本对拍（`test_mix_loader.py:64`）
- 11-24 本窗口的选项题「新 build 在哪出」收到了「上云」的回答。大总管 noise-canceling-33 事后说明：是它在 tmux 里代按的，人当时没看这个窗口。我无法核实是谁按的，按大总管说明记：这是大总管的决定（D97），人未参与
- 11-24 大总管 noise-canceling-33 接替 e1：认可此前自行跑的几次（下列）；批 cloudrun 同一台 CPU 机，和 traindata 合计最多 3 小时、1 美元。本分支合并后由 cloudrun 跑，小文件回到 `_tmp/20261125-cloudrun-exp013build/`，由我核 sha256 后提交到 `builds/<id>/`
- 自行跑的几次（无名额，补记）：11-24 scorecard_e2e 早期参考单条（220 s）· clippy 两次 · xtask tidy · scorecard 单测两次 · compare 冒烟（只读分数）· exp013 冒烟 build（4 段）· exp013 Python 自检
- 11-24 大总管发 1 个名额：codex 修完后整套 e2e 重跑用，跑完回报「名额已还」

## 云上出新 build（大总管 noise-canceling-33 定上云，依据 D97 重活上云，并批了开机费；人未参与）
在云机器的仓库根下（本分支合并后）：
```
PY=<云上 venv>/bin/python
$PY docs/experiments/exp013-scene-suite/build.py --workers 8
```
- 不给 `--build` 时自动取「时间-4 位十六进制」的新 id；同 id 已存在会拒绝，不覆盖旧 build
- 音频约 3.7 GB（估算：480 段 × 约 12 条轨 × 640 KB），留在云上 `_tmp/exp013/<build-id>/`
- 跑完把 `docs/experiments/exp013-scene-suite/builds/<build-id>/` 整个目录（清单 · 哈希 · 排除表 · 参数快照，不含音频）拉回本仓提交（CLAUDE.md 五.1）；再跑 `tasks.py` 指向新 build、提交 `src/eval/suite/scenes.json`
- 验收：`python -m unittest discover -s docs/experiments/exp013-scene-suite/tests` 全过（含旧 build 逐字节不变）；新 build 的 `clips.jsonl` 每行有 `audio.owner_early`

## 待人拍板
（暂无）

## 转交（范围外的发现）
- 吃字与两个评分模型（QA 层）的早期参考旁列本批没算：要换参考另跑模型，适合上云时一起做
