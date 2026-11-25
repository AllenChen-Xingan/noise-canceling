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

## 第 3 轮（2026-11-25，模型 gpt-6-astra，只读沙箱）

结果：P0×0 P1×5 P2×1。#1、#2 是第 1、2 轮同一主题的第三次提出；能落到具体代码的缺口照改，剩下的交大总管裁决。

| 编号 | 问题 | 处理 | 测试 |
|---|---|---|---|
| P1 #1 | 建机请求结果未知（503、断网）时无人接管 | 4xx 才算「明确没建」；其余按这次唯一机器名查云上，查到就删，并报「结果未知」；查也失败就提示 reap。队列取消失败仍只提示（排队的机器不计费，推断，未实测） | `test_create_post_is_not_retried` · `test_lost_create_response_finds_and_deletes_pod` |
| P1 #2 | 看门狗死后换账号，新看门狗会动旧账号的登记 | 每条登记带「密钥指纹 + 账本」；看门狗只处理自己那份 | `test_tick_skips_entries_of_other_account` |
| P1 #3 | 账本路径写错会按 0 放行 | 花钱前核对：云上我们的机器、网络盘都要在账本里，否则拒绝 | `test_wrong_ledger_refuses_when_cloud_has_our_volume` |
| P1 #4 | 机器全删后网络盘长期闲置仍会越过上限 | **驳回，交大总管裁决**：盘里是训练数据，何时删盘是保留期限的决定，不该由脚本自动删。README 写明闲置约 4.5 个月用完上限；ledger.py 的算法说明同步 | — |
| P1 #5 | reap 一台失败就停 | 每台单独处理，汇总失败，非零退出 | `test_reap_continues_after_one_failure` |
| P2 #6 | 假接口仍缺有状态故障 | 同第 2 轮 #7，留 issue | — |

测试：75 条全过。

## 第 4 轮（2026-11-25，模型 gpt-6-astra，只读沙箱）

结果：P0×0 P1×4 P2×2，另有 1 处引用不符（就是 #1）。

| 编号 | 问题 | 处理 | 测试 |
|---|---|---|---|
| P1 #1 | 机器名只到秒，结果未知时按名字清理可能删到同秒开的另一台 | 名字加 8 位随机尾巴 | `test_same_second_creates_get_distinct_names` |
| P1 #2 | 建机、建盘要先持久登记操作，持续核销 | 部分改：见第 3 轮 #1。**其余交大总管裁决**（同主题第 4 次） | — |
| P1 #3 | 旧格式登记、别的账号的登记长期占位 | 看门狗只剩别的账号的登记时也退出。**按账号分进程等其余交大总管裁决** | `test_loop_exits_when_only_other_accounts_entries_remain` |
| P1 #4 | 资源全删后，账本路径写错仍按 0 放行 | 花钱前要求账本文件已存在（新账本须先 `ledger.init`）。**核对历史账单交大总管裁决** | `test_missing_ledger_refuses_even_with_empty_cloud` |
| P2 #5 | 源文件比清单大时可能先写满盘 | 留 issue | — |
| P2 #6 | 有状态故障测试仍缺 | 留 issue | — |

测试：78 条全过。

### 交大总管裁决的残余风险（第 2–4 轮反复提出）

codex 要的是一套持久的操作日志、按账号分开的看门狗进程、对历史账单的核对。现状靠三层兜底：机器内自删（截止时刻写死，不靠本机），本机看门狗，`reap` 按 `vdn-` 前缀删。本项目只有一个账号、一本入库的账本。
建议：接受这部分残余风险，不再加机制；codex 停在「连续两轮无新类问题」而不是「无 P1」。

### 大总管裁决（noise-canceling-df，2026-11-25）

- 建机前持久登记、看门狗按账号分进程、账本切换核历史账单：接受残余风险，不再加机制。依据：只有一个账号、一本入库的账本，另有三层兜底；同一主题逐轮加层，按 CLAUDE.md 第二节「审检查器只防无心之失」处理，最高算 P2。三件各开一个 P2 issue。
- 网络盘闲置费：驳回 codex 那条。删不删盘由人定，大总管放进待拍板清单。
- 再跑 1 轮 codex，只看有没有新类问题；没有就开 PR（Closes #158），有就先报大总管。
- 已开 issue：#163 建机结果不明的持久登记 · #164 看门狗按账号分进程 · #165 账本核平台历史账单 · #166 predownload 源文件比清单大 · #167 假接口有状态故障测试。

## 第 5 轮（2026-11-25，模型 gpt-6-astra，只看新类问题）

结果：P0×0 P1×3 P2×0。三条都是新类（前四轮没提过），核实属实，已修：

| 编号 | 问题 | 处理 | 测试 |
|---|---|---|---|
| P1 #1 | 建盘、开机两个进程同时跑，各自过了上限检查、合起来超额 | 「查上限 → 建资源 → 记账」放进账本旁的跨进程文件锁（`<账本>.lock`，git 忽略） | `test_budget_lock_held_during_create` |
| P1 #2 | `put` 先替换正式文件再核哈希；`get` 直接覆盖本机文件 | `put` 先核 `.part` 再替换，默认不覆盖远端已有文件，`--overwrite` 时旧文件改名留作 `.bak-时间`；`get` 默认不覆盖，先写临时文件再改名 | `test_put_refuses_overwrite_and_keeps_backup` · `test_put_bad_hash_leaves_original` · `test_get_refuses_to_overwrite_local` |
| P1 #3 | `job-start` 把清旧退出码也放进后台，可能读到上次的退出码 | 准备步骤同步做完；同名任务还在跑（有日志没退出码）就拒绝 | `test_start_job_refuses_same_name_still_running` |

测试：83 条全过。按裁决，有新类问题先报大总管，不自行开第 6 轮。

## 第 6 轮（2026-11-25，模型 gpt-6-astra，只看新类问题）

结果：P0×0 P1×4，另有 1 处引用不符（同 #1）。都出在第 5 轮改的代码里，算新类，已修：

| 编号 | 问题 | 处理 | 测试 |
|---|---|---|---|
| P1 #1 | 「不覆盖」只在开始时查；并发上传共用 `.part` | 每次上传用自己的 `.part-随机`；发布用 `ln`（已存在就失败）；`get` 本机也用硬链接发布 | `test_put_does_not_clobber_file_that_appears_during_upload` · `test_each_upload_uses_its_own_temp_file` |
| P1 #2 | 备份名到秒会撞；备份失败仍替换 | 备份名加随机尾巴，用 `ln`（不覆盖）；备份成功才 `mv` | `test_overwrite_backup_names_are_unique` |
| P1 #3 | 同名任务检查不是原子的 | `mkdir <job>.run` 原子占位，跑完撤 | `test_job_name_claim_is_atomic` |
| P1 #4 | 脚本拼进括号，行尾注释会吞掉右括号；语法检查查的不是实际执行的东西 | 脚本作为独立参数交给 `sh -c`；脚本和包装都 `sh -n`；放后台后等日志出现才算启动 | `test_trailing_comment_in_script_is_safe` · `test_start_job_rejects_sh_syntax_error` |

另：第 6 轮之前我实测踩到「bash 专有写法在 sh 下解析失败，后台任务连日志都没有、一直像在跑」，已一并修（`sh -n` 检查）。
在真机上冒烟：put 不覆盖 / 覆盖留备份 / 带行尾注释的任务，都按预期。测试：89 条全过。

## 第 7 轮（2026-11-25，模型 gpt-6-astra）

结果：P0×0 P1×2 P2×1。两条 P1 是第 6 轮 #1、#3 的加深，先报大总管。

### 大总管裁决（noise-canceling-df）

- P1 #2 同名任务等待者拿到别次结果：同一类第二次出现，**改结构**。`job-start` 每次生成唯一运行编号（任务名-UTC 时间-随机），日志与退出码按编号存，`job-wait` 只认编号；任务名只作显示，复用从结构上不可能。先写会红的测试 `test_same_name_twice_waiter_gets_its_own_result`，再改到绿。原来的 `mkdir <job>.run` 占位随之去掉。
- P1 #1 两个进程同时覆盖上传同一文件：开 P2 issue #169，不修。理由：put 已先写临时文件、核对后原子替换，「后写覆盖先写」可接受；用法上不会对同一目标并发。
- P2 #3（`ln` 目标是目录时链接进目录里）：并入 #169 一起留档。
- 从第 8 轮起重新数，要连续两轮没有新类 P1。#163–#165、#169 再提一律按 P2。

测试：88 条全过（去掉 1 条被新结构取代的占位测试）。

## 第 8 轮（2026-11-25，模型 gpt-6-astra）

结果：P0×0 P1×2。都是第 7 轮结构改动带出的新类问题，大总管确认是真问题，已修。两条测试都先在修之前的代码上跑红（1 错 1 败），修后转绿。

| 编号 | 问题 | 处理 | 测试 |
|---|---|---|---|
| P1 #1 | 启动请求发出后断网，调用方拿不到运行编号，只能重跑 | 编号在启动前生成（`new_run_id`），CLI 先打到 stderr；启动途中出错报「结果未知」并带编号，提示先 `rp.py job-wait <机器> <编号>` 核查、不要重跑 | `test_lost_start_response_reports_run_id` |
| P1 #2 | probe 不看退出码，空日志被当成测速成功 | 退出码 0 且最后一行是带 `nproc` 的 JSON 才写结果；否则带编号、退出码、日志尾非零退出，机器照删 | `test_probe_fails_on_nonzero_rc_or_empty_log` |

测试：90 条全过。

## 第 9 轮（2026-11-25，模型 gpt-6-astra）

结果：P0×0 P1×0 P2×2。**连续第 1 轮无 P0/P1。**

- P2 #1 两处同时删同一台机器会把它的花费记两次：开 issue #171。先后删不会重复记，第二次按 0 记，因为账上已没有未关的开机行。
- P2 #2 「结果未知」只有中文提示，缺机器编号：提示改成完整命令 `rp.py job-wait <机器> <编号>`；结构化错误字段并入 #171 留档。
- 不符（记录里少写机器参数）：已改。

## 第 10 轮（2026-11-25，模型 gpt-6-astra）

结果：P0×0 P1×2，另有 1 处引用不符（同 #2）。都是新类，已修；停止线从第 11 轮重新数。

| 编号 | 问题 | 处理 | 测试 |
|---|---|---|---|
| P1 #1 | 手动 `down` / `reap` 没有确认就删 | 都要带 `--yes`；不带时只列出要删的机器（编号、名字、状态）和「容器盘一起没、网络盘不受影响」。看门狗、机器内自删、开机失败的自动清理不走 CLI，不受影响 | `test_manual_delete_needs_yes` |
| P1 #2 | 日志还没出现时，真机上 `tail` 失败，`job_status` 直接报错，「结果未知时按编号核查」走不通 | 查询加 `|| echo NOLOG` 兜底，按「没完」继续等；返回 `log_seen` | `test_wait_keeps_polling_until_log_appears`；假接口改成和真 shell 一样（日志不存在时 tail 失败），改前 `test_job_status_and_failure` 先变红 |

测试：92 条全过。
