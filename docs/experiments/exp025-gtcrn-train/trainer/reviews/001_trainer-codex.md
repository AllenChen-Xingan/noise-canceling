# 评审 001 · exp025 trainer 代码（codex · gpt-6-astra · read-only）

> 对象：`main...HEAD` 的 diff（`docs/experiments/exp025-gtcrn-train/trainer/**` + `state/handoff/trainer.md`）。**最新一轮：第 7 轮（2026-11-24）· 总评「修改」· 待第 8 轮**。
> 累计处理：P1×12、P2×12，逐条回源码核实：属实 23、驳回 1（第 1 轮 #1，第 2 轮 codex 给出反例后改为采纳）。
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

## 第 2 轮（2026-11-24）· 总评「修改」

原文在 scratchpad（不入库）。effort medium。本轮 P0/P1：3 条 · 停止线：未满足 · 留作 issue 的 P2：无

上一轮处理的验证：codex 核实 #2 #5 #6 #7 已到位。它指出我上一轮记录里有一句不符：「心跳在标定时也写」——实际上线程是在标定之后才启动的。**这句不符属实，是我写错了**，本轮已改成真正从第一行起生效。

- **P1｜train.py:334 准备期和每次评估前不检查上限与信号**（第 2 轮 #1）
  - 核实：属实
  - 处理：信号处理与心跳线程挪到函数第一行；标定逐批检查，要停就在准备期返回（不建 run.json、不存检查点）；每步算完后先查上限与信号，命中就跳过评估、直接存盘停下。测试 `test_signal_mid_training_saves` · `test_signal_saves_without_extra_eval` · `test_limits_hit_during_setup_stop_before_training` · `test_limit_hit_after_resume_saves_checkpoint`
  - 已知局限：一次开发评估（默认 20 批）进行中不再检查；量级是秒级（推断，未量化）
- **P1｜train.py:288 心跳漏记整个准备期**（第 2 轮 #2）
  - 核实：属实（同上一条的根因）
  - 处理：心跳从第一行起写；准备期就被杀、还没有检查点时，下次开跑从心跳接上账（写进 run.json 的 `billed_s_before`）。测试 `test_signal_during_setup_keeps_only_heartbeat`
- **P1｜losses.py:259 a = 0 字头的驳回理由有反例**（第 2 轮 #3，即第 1 轮 #1 第二次提出）
  - 核实：属实。片头只静 10 ms 就开口时，20 ms 分帧也给出 a = 0，那是真字头
  - 处理：改为采纳。默认与 V-ONSET 正本同口径（不跳过）；「跳过 a = 0」降成配置开关 `loss.on_skip_leading`，默认关，作扫参选项。测试 `test_onset_default_matches_rust_rule`
- **P2｜losses.py:279 HybridLoss 均值 ≤ 0 时自动标定给出非正权重，保护项静默失效**（第 2 轮 #4）
  - 核实：属实（SI-SNR 项是负对数）
  - 处理：按绝对值定尺度，接近 0 时取 1；输入不是有限数或结果不是有限正数就报错。测试 `test_calibrate_weight_positive_even_if_hybrid_negative`

### 引用抽查

- codex 列的「不符」1 条（心跳线程启动时机）：属实，已改
- codex 列的「核实属实」5 条

### 是否再审

有 P1，改完开第 3 轮。

## 第 3 轮（2026-11-24）· 总评「修改」

原文在 scratchpad（不入库）。effort medium，耗时 75 秒。本轮 P0/P1：1 条 · 停止线：未满足 · 留作 issue 的 P2：无

上一轮处理的验证：codex 核实 on_skip_leading 默认关、标定按绝对值并拒绝非正权重、心跳线程在准备前启动，都已到位。

- **P1｜train.py:313 最后一步收到信号时仍被判「步数跑满」并补评估；开发批构造与评估中不检查停止**（第 3 轮 #1）
  - 核实：属实
  - 处理：循环顶上上限 / 信号优先于步数；开发批逐批构造、逐批检查；评估逐批检查，中途要停就不记这一行。测试 `test_signal_on_last_step_skips_final_eval`
- **P2｜train.py:287 先写 run.json、后存第 0 步，中间被杀就被永久拒绝**（第 3 轮 #2）
  - 核实：属实
  - 处理：run.json 先记 `phase: init`，第 0 步落盘后改 `training`；重启时遇到「init 且没有任何检查点」，核对配置与数据一致就重建，账从心跳接上。测试 `test_crash_between_record_and_step0_can_rebuild`
- **P2｜train.py:139 开发评估把整批无效的保护项算进分母**（第 3 轮 #3）
  - 核实：属实
  - 处理：lvl / on 按有效段数加权，另报有效段数。测试 `test_evaluate_weights_protection_terms_by_valid_segments`
- **P2｜export.py:86 inf ≤ inf 会判通过**（第 3 轮 #4）
  - 核实：属实
  - 处理：三路输出和阈值都必须是有限数。测试 `test_export_compare_rejects_nonfinite`

另：续训时如果心跳里的账一开始就超过上限，现在会在准备期就停（不再花钱），报告已落盘的步数。

### 引用抽查

- codex 列的「核实属实」6 条，没有「不符」

### 是否再审

有 P1，改完开第 4 轮。

## 第 4 轮（2026-11-24）· 总评「修改」

本轮 P0/P1：1 条 · 停止线：未满足 · 留作 issue 的 P2：无。上一轮 4 条修复 codex 逐条核实到位。

- **P1｜train.py:234 同名运行没有互斥**（第 4 轮 #1）
  - 核实：属实（两个进程会同时写 run.json、心跳、检查点，临时文件同名）
  - 处理：读任何状态前先用 flock 独占 `run.lock`，第二个进程直接报错；进程死了锁自动释放。测试 `test_same_run_cannot_start_twice`
- **P2｜smoke.py:118 开发份复现把 NaN 判通过**（第 4 轮 #2）
  - 核实：属实
  - 处理：实测、已知、容差都必须是有限数；缺键、空基准都不算通过。测试 `test_known_dev_nan_and_missing_fail`
- **P2｜smoke.py:164 正式数据冒烟可以不给开发份基准**（第 4 轮 #3）
  - 核实：属实（014 第 3 步要求开发份读数等于已知成绩）
  - 处理：正式数据模式下没给 `--known-dev` 就记「未完成」，冒烟不通过；合成小样保持原判据

### 是否再审

有 P1，改完开第 5 轮。

## 第 5 轮（2026-11-24）· 总评「修改」

本轮 P0/P1：0 条 · 停止线：连续第 1 轮无 P0/P1 · 留作待办的 P2：#2 的一部分（见下）。上一轮 3 条修复 codex 核实到位。

- **P2｜config.py:161 上限可以填 .inf / .nan，单价可以为负**（第 5 轮 #1）
  - 核实：属实
  - 处理：上限必须是有限正数，单价必须是有限非负数，开跑前拒绝；测试加进 `test_unknown_key_and_bad_values_rejected`
- **P2｜tests/test_data.py:110 真 loader 的对接测试只查名字**（第 5 轮 #2）
  - 核实：属实；真 loader（traindata 的 `data/loader.py`）还没合入本分支，本机也没有训练数据（D88）
  - 处理：补 `test_real_loader_resume_from_batch_k_equals_continuous`（worker 0 / 2，从第 k 批起取 = 连续取）。合入后设 `VDN_LOADER_SHARDS` / `VDN_LOADER_RECIPE` 自动跑
  - 留作待办：GPU 混音路径与真数据下的续训权重逐值比较，要云上显卡和分片，记在现状页「转交」，由 cloudrun 云上冒烟时跑

### 引用抽查

- codex 列的「核实属实」5 条；「无法核实」1 条（exp019 流式源码不在它的补读清单里）——本机测试 `test_offline_matches_exp019_stream` 实跑通过

### 是否再审

只连续 1 轮无 P0/P1，再审一轮确认。

## 第 6 轮（2026-11-24）· 总评「修改」

本轮 P0/P1：1 条 · 停止线：未满足（重新计数）。上一轮 #1 codex 核实到位。

- **P1｜data.py:156 worker = 0 时取数卡住，上限与信号都停不了**（第 6 轮 #1）
  - 核实：属实（DataLoader 只在子进程模式支持超时；信号只置位）
  - 处理：① 正式数据要求 `num_workers ≥ 1`；② 心跳线程兼作看门狗：已该停而主线程 `limits.hang_s`（默认 120 s）无进展时写 `hung.json` 后强退，最近检查点完整，下次按心跳续训补账。测试 `test_watchdog_fires_only_when_stop_wanted_and_stalled` · `test_stalled_fetch_after_signal_triggers_watchdog` · `test_loader_rejects_zero_workers`
- **P2｜tests/test_data.py:127 真数据续训验收没完成；记录里说的现状页「转交」不存在**（第 6 轮 #2）
  - 核实：属实。**「转交」那句是我上一轮写错了**：当时没写进现状页
  - 处理：现状页补「转交」一节（云上 GPU 续训等价的具体步骤、开发份基准、loader 合入后的测试）；README 把续训等价的通过范围限定为「CPU + 合成数据」
- **P2｜data.py:163 标定实际只有 63 批，不是配置的 200 批**（第 6 轮 #3）
  - 核实：属实（dev_n 2000 ÷ 批量 32）
  - 处理：标定单独建一份恰好 `calib_batches × batch` 段的开发集（同种子，前 dev_n 段与开发份相同）。测试 `test_calibration_gets_all_requested_batches_beyond_dev_n`

### 引用抽查

- 「不符」1 条（现状页「转交」）：属实，已补
- 「核实属实」4 条

### 是否再审

有 P1，改完开第 7 轮。

## 第 7 轮（2026-11-24）· 总评「修改」

本轮 P0/P1：1 条 · 停止线：未满足。上一轮 3 条修复 codex 核实到位（含现状页「转交」已补）。

- **P1｜train.py:222 看门狗依赖写盘成功，网络盘坏了或卡住时到上限也强退不了**（第 7 轮 #1）
  - 核实：属实（心跳在判断之前同步写；异常会结束线程；`_hard_exit` 要先写成 hung.json）
  - 处理：每轮先判断再写盘；心跳与 hung.json 都放进有时限的子线程写（`_bounded`），出错或超时只记次数、不抛、不挡强退。测试 `test_watchdog_fires_even_if_heartbeat_write_fails`（报错 / 卡住两种）· `test_hard_exit_exits_even_if_record_write_blocks`

### 是否再审

有 P1，改完开第 8 轮。
