# 评审 002 · H100 冒烟方案与两处新代码（codex · gpt-6-astra · read-only）

> 对象：`plan-h100-smoke.md` · `smoke.py --write-known-dev`（含 `check_known_dev` 的 meta 核对）· `cmp_ckpt.py` · `tests/test_smoke_tools.py`。
> 重点（大总管定）：会不会多花钱；判据会不会误判通过。停止线照 001：连续两轮没有 P0，且除故障注入类之外没有 P1。
> 原文在 `_tmp/codex-h100/`（不入库）。

## 第 1 轮（2026-11-24）· 总评「修改」

范围 `2362d715...HEAD`，effort medium，耗时 40 秒。P0×0 P1×4 P2×1。逐条回源码核实，5 条全部属实，全修。

- **P1｜测试红会被管道掩盖**（第 1 轮 #1）：`pytest … | tail -20` 的退出码是 tail 的，甲段「测试红就停」会失效
  - 处理：方案里的命令去掉管道（`job-start` 已经把输出写进日志），并写明命令里不用管道
- **P1｜`--write-known-dev` 输出目录不存在时算完写不出**（第 1 轮 #2）：`open(out, "x")` 不建父目录，方案也没先建 `$O`
  - 处理：`write_known_dev` 在计算前建父目录；方案甲段加第 0 步 `mkdir -p $O`
  - 测试：`test_write_known_dev_creates_missing_parent_dir`
- **P1｜6.95 美元不是最坏花费**（第 1 轮 #3）：cloud 允许目录价高 20%，每小时 5 美元时含宽限可达 7.92 美元
  - 处理：方案乙段改为开机前读 `rp.py offers` 的价 p，时限 `H = min(1.5, 7/p − 5/60)`；H < 1.0 不开；开出来按实际价再核
- **P1｜失败路径没有删机**（第 1 轮 #4）：「非 0 就停下」而删机在最后一步，H100 会一直计费到时限
  - 处理：方案加「出错时的必经收尾」：任何一步失败或超时，跳过本段剩下的步骤，先取回、再删机，然后才报
- **P2｜两份检查点都缺 step 能通过**（第 1 轮 #5）：`None == None`
  - 处理：不带 `--ignore-step` 时要求两边 step 都是非负整数且相同
  - 测试：`test_cmp_missing_or_bad_step_fails`（缺失、字符串、布尔、负数、浮点 5 种）

处理后：新测试 24 条通过。随后按大总管要求合进 main `73631939`（PR #182），方案填了提交说明。
