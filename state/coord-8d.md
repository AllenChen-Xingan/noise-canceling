# 大总管看板 · noise-canceling-8d

> 本大总管的账本（e3 的在 `state/coord.md`，99 的在 `state/coord-99.md`，不改它们）。git 写操作在 `../noise_canceling-hygiene` 做。

## 当前事 · 代码卫生（2026-11-02 起，旁支，人点名）

人原话：「另外这几个，也让你来管理，然后修复吧 / 新开一个pr 和issue，来修复」。来源：三个通用技能（single-source-truth · complexity-budget · verify-first）扫描，报告 `docs/reviews/three-skills-scan-20261102/README.md`。

| 手下 | 线 | 管的实体 | 状态 |
|---|---|---|---|
| hygiene | 600 行上限进 tidy · cli 同名类型 · 拆 cli 大文件；第二段（等 99 的 #57 合并）采样率只写一处 · 阈值比对 · #62 · `feat/code-hygiene` · `../noise_canceling-hygiene` | xtask · vdn-cli · vdn-types（采样率）· vdn-eval（阈值） | 已派（交代 `_tmp/20261102-b8/hygiene.md`） |
| split | 拆 denoised / capture / speaker-lab 到 ≤600 行 · 同 #62 同 worktree | vdn-denoised · vdn-capture · vdn-speaker-lab | 完成，窗口已关；43 测试拆前拆后同；交接 `state/handoff/split.md` |

## 进度板

- 2026-11-02 22:0x 开 #62、worktree、派 hygiene。第一段避开 #57 在改的 control · denoised · eval · shm · schema
- 2026-11-02 更正：手下首条消息误写报到给 e3，已改为 8d 并发消息更正；已撤回误加在 e3 `state/coord.md` 的两行
- 2026-11-02 99 回复：采样率收口、阈值比对都留在 #62；提醒第一段别改 xtask/ci.rs、第二段改 ci.json 后跑 vdn-schema --check；#57 合并时它会通知
- 2026-11-02 hygiene 已改向 8d 报到；第一段第 1 件（R-SIZE · file_size.rs · 基线 14 条 · 8 条单测）代码写完未编译。e3 确认 hygiene 归 8d，board/backlog 说明行保留
- 2026-11-02 e3 基准结束、锁已删；通知 hygiene 可跑单 crate 测试（每次编译前看锁）
- 2026-11-02 hygiene 报第一段完成；8d 抽查：未碰禁区、Verdict 改名 ChainVerdict/LiveVerdict、stop.rs 带 2 测、基线 11 条；8d 亲手跑 xtask ci 中。风险：基线含 #57 在改的文件，第二段先合 main 再核基线
- 2026-11-02 e3 计时约 7 min（fastpath v4short），8d 验收编译已停；hygiene codex 第 1 轮在跑，基线风险已确认收到
- 2026-11-02 99 报 #57 已合并（PR #63）；8d 把 origin/main 合进 hygiene（601202e，无冲突）；基线 3 处待改（denoised 878 · synth 删 · schema/tests/crosscheck.rs 870 新进）；放行第二段
- 2026-11-02 人定新规矩「所有代码文件 ≤600 行」，不设基线（写进 hygiene worktree 的 CLAUDE.md 第五节第 7 条 + 记忆）。分工：99 的 #64 拆 pipeline.rs · stages.rs，#65 拆 desc.rs · policy.rs · schema/tests/crosscheck.rs；e3 的 fastpath 在 #58 拆 bench 3 个文件，#58 合并后 #62 扫描扩到 docs/；hygiene 拆 verifiers.rs · runner.rs · single_source.rs；新派 split 拆 denoised · capture · speaker-lab。#62 硬上限等 #64/#65 合并后再合
- 2026-11-02 派后台分析：AST 做了之后的好处、复杂度对比、扩展方向 → docs/reviews/ast-complexity-20261102/README.md
- 2026-11-02 99 #65 要改 cli ModelArg（#62 已搬到 args.rs）与 xtask 新文件 model_names.rs：同意；#65 先合，8d 合 main 进 #62 时把改法搬进 args.rs
- 2026-11-02 split 报到（noise-canceling-hygiene-ad）；single_source.rs 也归 99 iface；hygiene 只拆 eval/verifiers.rs · eval/runner.rs
- 2026-11-02 R-RATE 分工：hygiene 改其余 crate（跳过 99 名下 6 个文件）；split 改 capture/denoised/speaker-lab 15 处；99 名下文件的字面量等 #64/#65 合并后，由 hygiene 在 #62 合 main 时改（rate.rs 在 #62 才有）
- 2026-11-02 e3 基准结束、锁删；两个手下改动进行中，全量验收等两人都报完成再跑
- 2026-11-02 split 报：三文件已拆完、R-RATE 15 处已改；拆前测试数用导出旧提交对比法补跑
- 2026-11-02 人定：single_source.rs 拆分时按规则改名（no_mirror.rs · param_range.rs），已转 99 iface
- 2026-11-02 split 完成：三文件拆到 ≤600（最大 487）· 43 过拆前拆后同 · clippy 净 · R-RATE 15 处；8d 抽查行数与交接后关窗；两件转 hygiene（删 3 条基线 · tidy 认 cfg(test) 模块）
- 2026-11-02 hygiene 第 5 件完成：rate.rs · R-RATE · ~115 处改引用（纠正 eval/system.rs 1 处语义错）· 基线剩 8 · test_modules.rs（认 cfg(test) 模块，带负例）；11 crate 274 过 / 4 忽略，tidy_passes 只剩 99 名下 6 处字面量。第 6 件发现 V-TIMBRE 分档 代码 2/4/6 dB vs rubric N4 1.5/3/6 dB → 让它查来历、量后果、报我，不自行改
- 2026-11-02 hygiene 第 6 件：7 处阈值不一致或无出处（state/handoff/hygiene-thresholds.md），全是建议值，分档只影响报告等级、复算变化 0 条。8d 定：不新出题，挂 012 拍板 #8；V-TIMBRE/V-SPLVL 保留代码量法；建正本表 + 两条比对测试，待定清单显式、新不一致与已修好都要红
- 2026-11-02 hygiene 第 6 件完成：thresholds.rs 正本（16 评测器，每数带出处）· 比对测试 5 条 · 待定 7 处（更正：V-INTF 有出处，V-SPLVL 过线 −3 无出处）· eval ci 与改前一致 13/17 · 1/10 · 5/5。下一步拆 verifiers.rs / runner.rs
- 2026-11-02 hygiene 拆完 verifiers.rs 1168→399（physical/perceptual/sim）· runner.rs 629→424（gate/real）；基线剩 6 条全是 99 的文件；硬上限放最后；docs/experiments 扫描等 #58 合并后打开。8d 开跑全量 xtask ci 验收
- 2026-11-02 99 知会 #65 带来：denoised/lib.rs +16 行（FE-M 名从 ModelId 取）· policy 拆出 policy_tests.rs（SR 在 :90）· tidy 新规则文件 no_mirror/param_range/scan/model_name · cli 删 ModelArg 加 ModelOpts::kind()。合并 #62 时要搬：denoised 16 行进拆后文件、ModelArg 改动进 args.rs、改名引用、policy_tests SR
- 2026-11-02 8d 亲手验收 a94623aa：xtask ci fmt/clippy/check-deps/cargo-deny 过；test 431 过 / 1 败 / 5 忽略（--no-fail-fast），唯一失败 xtask tidy_passes；tidy 6 处全是 99 名下 stages.rs ×5 · policy.rs ×1 的采样率字面量（预期，等 #64/#65 合并后补）。基线 6 条不报错
- 2026-11-02 hygiene codex 第 2 轮 P1×3 P2×3 属实已改（待定清单精确到值 · tidy 测试模块认法剥注释 · 阈值比对保号与方向 · R-RATE 放行要理由）；V-SPLVL 量法实为同 rubric C3 → 8d 定改为 4（铁律 5），移出待定
- 2026-11-02 hygiene：V-SPLVL 改 −1/−2/−4（rubric C3），待定剩 6 处；vdn-eval 全过（比对 7 条）· xtask 45/1（tidy_passes 预期）· eval ci 等级变化 0、门禁不变；codex 第 3 轮在跑
- 2026-11-02 e3 #66 合并；8d 合 origin/main 进 #62（ebc246a7，冲突仅评审索引两行都留）；docs/experiments 全 ≤600 → 让 hygiene 打开 docs 扫描。顺手修 codex-review 技能 privacy-scan.sh：以 P1/P2 开头的评审被 file 误判为 Netpbm 图片 → 改为字节复查（无 NUL 且合法 UTF-8 才放行），并按扩展名拦 pbm/pgm/ppm/pnm；4 个样例验证
