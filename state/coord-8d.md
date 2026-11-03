# 大总管看板 · noise-canceling-8d

> 本大总管的账本（e3 的在 `state/coord.md`，99 的在 `state/coord-99.md`，不改它们）。git 写操作在 `../noise_canceling-hygiene` 做。

## 当前事 · 代码卫生（2026-11-02 起，旁支，人点名）

人原话：「另外这几个，也让你来管理，然后修复吧 / 新开一个pr 和issue，来修复」。来源：三个通用技能（single-source-truth · complexity-budget · verify-first）扫描，报告 `docs/reviews/three-skills-scan-20261102/README.md`。

| 手下 | 线 | 管的实体 | 状态 |
|---|---|---|---|
| hygiene | 600 行上限进 tidy · cli 同名类型 · 拆 cli 大文件；第二段（等 99 的 #57 合并）采样率只写一处 · 阈值比对 · #62 · `feat/code-hygiene` · `../noise_canceling-hygiene` | xtask · vdn-cli · vdn-types（采样率）· vdn-eval（阈值） | 已派（交代 `_tmp/20261102-b8/hygiene.md`） |

## 进度板

- 2026-11-02 22:0x 开 #62、worktree、派 hygiene。第一段避开 #57 在改的 control · denoised · eval · shm · schema
- 2026-11-02 更正：手下首条消息误写报到给 e3，已改为 8d 并发消息更正；已撤回误加在 e3 `state/coord.md` 的两行
- 2026-11-02 99 回复：采样率收口、阈值比对都留在 #62；提醒第一段别改 xtask/ci.rs、第二段改 ci.json 后跑 vdn-schema --check；#57 合并时它会通知
- 2026-11-02 hygiene 已改向 8d 报到；第一段第 1 件（R-SIZE · file_size.rs · 基线 14 条 · 8 条单测）代码写完未编译。e3 确认 hygiene 归 8d，board/backlog 说明行保留
- 2026-11-02 e3 基准结束、锁已删；通知 hygiene 可跑单 crate 测试（每次编译前看锁）
- 2026-11-02 hygiene 报第一段完成；8d 抽查：未碰禁区、Verdict 改名 ChainVerdict/LiveVerdict、stop.rs 带 2 测、基线 11 条；8d 亲手跑 xtask ci 中。风险：基线含 #57 在改的文件，第二段先合 main 再核基线
- 2026-11-02 e3 计时约 7 min（fastpath v4short），8d 验收编译已停；hygiene codex 第 1 轮在跑，基线风险已确认收到
- 2026-11-02 99 报 #57 已合并（PR #63）；8d 把 origin/main 合进 hygiene（601202e，无冲突）；基线 3 处待改（denoised 878 · synth 删 · schema/tests/crosscheck.rs 870 新进）；放行第二段
