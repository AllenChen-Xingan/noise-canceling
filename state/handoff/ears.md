# 现状页 · ears（批次 20261102-b7，续 20261031-b6）

- **状态**：进行中
- **手上**：找一切办法把「更多人耳」接进评测，做成可验证的东西（issue #42）
- **交代**：主目录 `_tmp/20261102-b7/ears.md`（续 `_tmp/20261031-b6/ears.md`）
- **做到哪**（2026-11-02）：b7 收口。第 3 步完成：decision 第 8 节按 D58/D60/D62/D64 重看，E1/E3/E4/E5/E7 划掉写依据，剩 E2（盲听夹大众已评题）· E6（向上游报错）按人能感知的后果改写；第 0/1/2/3/4 节与 D62 冲突处已改
- **下一步**：等名额（排 benchsynth 后）跑 regen_exp003 + score_author（要建 _tmp/exp009-venv）→ 回填第 6 节 → 对抗自审 → 报收尾

## 改了哪些文件
| 文件 | 改了什么 |
|---|---|
| `state/handoff/ears.md` | 新建 |
| `docs/research/017_more-ears/notes/A–H, Z` | 9 份调研笔记 |
| `docs/research/017_more-ears/decision.md` | 方案 |
| `docs/experiments/exp009-more-ears/` | README · fetch_labels · crowd_ears · regen_exp003 · score_author · data-manifest · results/ |

## 发现
- 上游 bug：vmc2026-track1-meta 的 CHiME-7 MOS 把 SIG/BAK/OVRL 三表混算（resources/urgent2026_challenge_track2/scripts/data/chime-7-udase-eval/data_prep.py:42-58）；BENCHMARK.md:19 与 :52 NISQA 行抄错。exp009 已绕开
- exp003 原听测音频已不在本机（abtest worktree 的 _tmp 清掉了）→ regen_exp003.py 用 exp004 留的 src 重新生成并核增益
- exp007 calib：顶层 screens 12、指标级 screens 9（3 屏全答「分不出」），建议实现线 md 同时写 12 / 9
- URGENT 2024 distill_mos.scp 有一个系统整片缺分（300 条 nan）

## 待人拍板
见 `docs/research/017_more-ears/decision.md` 第 8 节：只剩 E2（盲听每轮多听 4–6 道大众已评题）· E6（用你的账号向上游报两处数据错误）；E1 并入已有待拍板 4-1

## 转交（范围外，留给大总管）
- 

## 流水
- 读交代、common、协议、intent、待拍板、D44/D48、exp007 calib → 作者一致率基线：DNSMOS OVRL 0.48 [0.26,0.70]（48 对）
- 报到大总管（noise-canceling-c4）→ 给 exp009；名额在 benchsynth 手里
- 起 workflow wf_ad512269-bb9（7 调研 + 1 批评，笔记写 docs/research/017_more-ears/notes/）
- 摸本机：DNSMOS onnx（_tmp/exp002-venv）、NISQA 权重（resources/NISQA/weights）、torch venv（_tmp/exp008-torch-venv，benchsynth 的，只读借用或自建 exp009-venv）；exp007 定标法在 src/eval/src/calib.rs（同屏配对、按屏聚类 bootstrap、种子 20261030）——exp009 照此口径
- 大总管追加 H 线（CPC/DNS5 WAcc/SSI/vmc2026）→ 派子代理写 notes/H-extra-sources.md
- workflow 完成；H 线完成（CPC 听众是听障者，不能外推）
- fetch_labels.py：标签约 27 MB + CHiME-7 CSV 1.7 MB（Range），哈希与批评者一致 → data-manifest.json
- crowd_ears.py 第一段出数（3 分钟，无推理）；修 distill_mos nan
- 申请名额（regen + venv + 打分），大总管批准下载 ~400 MB、排队中
- 写 exp009 README（预注册 V-AUTHOR-VS-CROWD 判据：作者 SCOREQ 一致率 < 0.516 才算偏离）、decision.md 初稿（含 D51 要的评委起点一节）
- 回 judgeloop（018）：起点 SCOREQ + 各数据集数字、本机现成评分模型路径（DNSMOS · NISQA；SCOREQ 待名额）
- 2026-11-02 b7：读交代/common/D56–D65 → 报到 noise-canceling-e3，名额排第二 → 重看第 8 节并改 decision；核第二段输入齐（exp003 build f8c3 答卷、exp004 src 在；exp009-venv 未建）
