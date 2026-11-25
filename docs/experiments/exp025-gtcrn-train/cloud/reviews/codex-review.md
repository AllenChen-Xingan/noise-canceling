# cloud/ 换谱系评审记录（codex）

对象：`docs/experiments/exp025-gtcrn-train/cloud/` 整个目录（类型 contract，effort high）。
放在这里而不是 `docs/reviews/`：cloudrun 的交代只许写 `cloud/**`。
停止线：连续两轮没有 P0/P1；剩下的 P2 开 issue。

## 第 1 轮（2026-11-25，模型 gpt-6-astra，只读沙箱）

结果：P0×0 P1×9 P2×3。逐条回源码核实，结论都是「属实」（#1 风险低但改了也便宜）。

| 编号 | 问题 | 处理 | 测试 |
|---|---|---|---|
| P1 #1 | reap 补账时不核前缀 | 先查云上：404 才补账；名字不是 `vdn-` 开头就跳过并提示 | `test_reap_skips_ledger_pod_renamed_away_from_prefix` |
| P1 #2 | 花费上限只算开着机器已跑的部分；建盘不查上限 | 开着的机器按跑满登记时限 + 宽限算；建盘先按一个月盘费（`VOLUME_RESERVE_HOURS`）查 | `test_open_pods_reserve_their_full_max_hours` |
| P1 #3 | 实际价只在超 1.5 倍时才删，和目录容差不一致 | 统一用 `PRICE_TOLERANCE`；按实际价再查一遍上限，不过就删 | `test_price_moderately_above_tolerance_deletes` · `test_actual_price_rechecks_budget` |
| P1 #4 | 建机 POST 遇 503 会自动重发，可能多开一台 | 建机、建盘的 POST 不重试 | `test_create_post_is_not_retried` |
| P1 #5 | 建机后写账本等步骤出错，机器无人管 | 建机后的步骤包在保护里：出错先删机；删也失败就报 `down`/`reap` 命令 | `test_failure_after_create_deletes_pod` |
| P1 #6 | 看门狗子进程不继承 `--ledger`/`--env-file` | 原样传给子进程；同时只跑一个看门狗的局限写进注释 | `test_spawned_watchdog_gets_same_ledger` |
| P1 #7 | 登记表读改写无锁；看门狗退出与新登记交错 | 加文件锁；看门狗空表退出前在锁里撤掉自己的 pid 文件 | `test_exiting_watchdog_drops_pidfile_so_next_up_spawns` · `test_exit_keeps_other_watchdogs_pidfile` |
| P1 #8 | 容器重启会重新睡满时限 | 开机时写死绝对截止时刻，重启只睡剩下的 | `test_restart_only_sleeps_the_remainder` |
| P1 #9 | 盘量上限只看下载前；du 失败当 0 | 把本包大小算进去再判；du 失败退出 6 | `test_stops_before_a_file_would_cross_the_cap` · `test_du_failure_stops` |
| P2 #10 | 错误详情若回显密钥会进日志 | 错误详情里把密钥替换成 `<key>` | `test_echoed_key_is_redacted_in_errors` |
| P2 #11 | `.rc` 空着也算完 | 只有合法整数退出码才算完 | `test_job_with_empty_rc_is_not_done` |
| P2 #12 | 假接口不模拟关键失败 | 上面新测试补了 POST 失败、写账失败、容器重启、下载越线；账户切换未补，留 issue | — |

测试：63 条全过（`python3 -m unittest discover -s tests`）。

## 第 2 轮（2026-11-25，模型 gpt-6-astra，只读沙箱）

结果：P0×0 P1×4 P2×3，另有 1 处引用不符。都是第 1 轮问题的延伸，核实属实。

| 编号 | 问题 | 处理 | 测试 |
|---|---|---|---|
| P1 #1 | 只在建盘时查一个月盘费，之后开机不再预留 | 没删的网络盘每次都按往后一个月盘费算进最坏总额（`volume_reserve`）；README 同步 | `test_budget_counts_pods_volume_and_projection` · `test_deleted_volume_has_no_reserve` |
| P1 #2 | exec 遇 503 仍自动重发 | exec 只在 409（容器在启动）、429（限流）时重发，这两种肯定没执行 | `test_exec_not_resent_on_503` |
| P1 #3 | 登记看门狗、建盘写账失败不在保护内 | `bring_up` 从建机后到 exec 就绪整段保护，出错先删机；建盘写账失败就删掉空盘 | `test_register_failure_after_create_deletes_pod` · `test_volume_ledger_failure_deletes_new_volume` |
| P1 #4 | 看门狗跨账号、跨账本复用 | pid 文件记「密钥指纹 + 账本绝对路径」；已有看门狗管的是别的，开机前就拒 | `test_watchdog_for_other_account_refuses_before_create` |
| P2 #5 | 断点续传时整包重算、下完不复查 | 只预留还差的字节；下完查 `.aria2` 标记和实际用量 | `test_partial_file_only_reserves_missing_bytes` |
| P2 #6 | 先截断后脱敏会留密钥残片 | 先脱敏再截断 | `test_redaction_happens_before_truncation` |
| P2 #7 | 假接口仍缺有状态故障 | 留 issue（队列状态、执行后响应丢失、caffeinate 包装进程） | — |
| 不符 | README 说容器只跑 sleep | 改成「睡到截止时刻再自删」 | — |

测试：71 条全过。
