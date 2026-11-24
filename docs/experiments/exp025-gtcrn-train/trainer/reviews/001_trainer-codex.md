# 评审 001 · exp025 trainer 代码（codex · gpt-6-astra · read-only）

> 对象：`main...HEAD` 的 diff（`docs/experiments/exp025-gtcrn-train/trainer/**` + `state/handoff/trainer.md`）。**最新一轮：第 1 轮（2026-11-24）· 总评「修改」· 待第 2 轮**。
> 累计处理：P1×5、P2×2，逐条回源码核实：属实 6、驳回 1。
> 记录放在本目录：trainer 的写入范围只有 `trainer/**`，不写 `docs/reviews/`；收口时由大总管决定要不要并入总索引。

## 第 1 轮（2026-11-24）· 总评「修改」

原文在 scratchpad（不入库）。effort medium，耗时 67 秒。行号指评审时的版本。
本轮 P0/P1：5 条 · 停止线：未满足 · 留作 issue 的 P2：无（两条都改了）

- **P1｜losses.py:259 训练跳过 a = 0 的字头，对拍不跳过；恰好 3 个字头且首个在 0 点时，Rust 判有效、训练整段跳过**（第 1 轮 #1）
  - 核实：现象属实，但这是有意的
  - 驳回理由：
    - 训练段是随机裁切的，a = 0 说明裁切点落在说话中间，之前的停顿被裁掉了，这不是真字头；再叠上 istft 首帧的边缘效应，罚它就是罚裁切
    - 评测片段以停顿开头，正本规则在那里本来就碰不到 a = 0，所以「对拍」核的前向（skip_leading=False）与正本逐值一致
    - 训练口径只多一条「a = 0 不算」，剩下不足 3 个时按同一条样本不足规则跳过
    - 差异有测试钉住：`test_skip_leading_onset_only_in_training`
  - 处理：不改代码；理由写在 `losses.onset_plan` 的说明里
- **P1｜train.py:286 损失有限但梯度非有限时照样更新**（第 1 轮 #2）
  - 核实：属实（`clip_grad_norm_` 默认不报错）
  - 处理：更新前查梯度范数，不是有限数就按训崩停下、不存坏权重；测试 `test_nonfinite_gradient_does_not_update`
- **P1｜train.py:185 上限没覆盖准备与收尾；到上限或收到信号后先评估再存盘**（第 1 轮 #3）
  - 核实：前半部分是旧读法。计费已经从函数开头算起，第一次检查就含准备时间；不过标定期间确实不检查。后半部分属实
  - 处理：停下时先存检查点；只有正常跑完才补评估，到上限或收到信号时不评估；测试 `test_signal_saves_without_extra_eval`。标定期间不检查：标定默认 200 批前向，量级几分钟（推断），留作已知局限
- **P1｜train.py:212 心跳只在记日志时写，被杀前那段漏记**（第 1 轮 #4）
  - 核实：属实
  - 处理：改成后台线程每 30 s 写一次心跳（取数卡住时也写），漏记的上界是 30 s；机器的真实账单归 cloudrun。测试 `test_heartbeat_thread_writes_while_data_stalls`
- **P1｜train.py:217 检查点全坏时被当成新运行**（第 1 轮 #5）
  - 核实：属实
  - 处理：已有 run.json 或检查点文件、却一份都读不出时报错，不覆盖原记录；测试 `test_all_checkpoints_corrupt_is_an_error_not_a_fresh_run`
- **P2｜runrec.py:24 续训改单价会追溯改写已花的钱**（第 1 轮 #6）
  - 核实：属实
  - 处理：美元按段累计并写进检查点和心跳，每段只按本段单价算；续训时在检查点、心跳、上次停下记录三处取最大值；测试 `test_usd_accumulates_per_segment_price`
- **P2｜smoke.py:126 测速没传设备，云上走 CPU 混音，不代表正式管线**（第 1 轮 #7）
  - 核实：属实
  - 处理：传入设备，模型模式与训练一致

### 引用抽查

- codex 列的「核实属实」：5 条
- 没有「不符」

### 是否再审

有 P1，改完开第 2 轮，附本记录；#1 的驳回理由交 codex 再看。
