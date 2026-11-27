# 大总管交接页（2026-11-27 约 02:20Z，旧窗口 vdn:coord，会话 439efe9c，上下文约 488k）

- **固定名**：`noise-canceling-meta`（轮换不变；旧会话退位时改名 `noise-canceling-meta-old`）
- **当前大事（b19）**：GTCRN 加训第一批。人定目标「开始训练，并且完成训练」，D119「开，练完接着评」
- **在做**：正式训练已完成（6 个运行练满 5000 步）；正在云上评测 7 份成绩单（D123：3 台 CPU 机先后开）

## 各线

| 窗口 | 会话名（会变，以 ListAgents 为准） | 分支 / worktree | 任务 | 状态 | 在等什么 |
|---|---|---|---|---|---|
| cloudrun | noise-canceling-cloudrun-01 | exp/025-cloud-baseline · `../noise_canceling-cloudrun`（本地有未推记账提交） | 开关机、记账 | 评测机 1 `begw6apbx1rtl4` 01:38Z 开、时限 5.5 h，在跑回归检查 | 关口报告；机 1 首份成绩单后重估机费，三台合计 > 1.92 美元先报 |
| evalcmp | noise-canceling-evalcmp-66 | exp/025-eval-cloud（#235 已合）· `../noise_canceling-evalcmp`；判读副本 `../noise_canceling-evalrun`（2a07e8ed，已编译） | 评测执行单 `eval/cloud-run.md`、判读 | 转发关口 | 7 份成绩单回来后在 evalrun 跑 12 份 compare + 2 份 accept，报结论 |
| trainer | noise-canceling-trainer-4d | feat/trainer-formal-results（#234 已合，分支可换） | 训练器 | 待命 | #231 A 组流式导出（评测后做）；#233 训练慢测速（人未定） |
| traindata | noise-canceling-traindata-75 | feat/data-batch5（#227 已合） | 训练数据 | 待命 | 无 |

## 关键事实（证据在看板与决策记录）
- 训练数据：第五批 build `df759bb8d606a209`，分片 `/workspace/shards/batch5`；batch2 按 D112 等第一个正式运行过开发份检查后删（已满足条件，未删，**待做**：按 D118「确定没用的都删」交 cloudrun 删前取回清单）
- 检查点：`/workspace/runs/exp025-formal/exp025-formal-<A|B>-s<0..2>/ckpt/`，清单 `_tmp/20261126-trainer-formal/batch2/trainer-formal-ckpt-manifest-20261127T002828Z.txt`；评测完前不删
- 开发份 best：A 3.6422/3.6486/3.6495 · B 3.6418/3.6439/3.6527（起点 3.8132）；A、B 分不出；B 的 FiLM 仍恒等，B = A + 时间注意力
- 评测钉 EVAL_COMMIT 2a07e8ed；延迟一行判「判不了」（#231）；报告种子 s1；判法 D120
- 开训前清单：`docs/plan/014_training-kickoff.md` 第七节，9 项全 ✅

## 花费
- 累计约 23.08 美元（训练后）+ 评测进行中；上限 100（D115）。评测批：机器费最坏约 1.92（D123），识别 ≤1.2；账本起点 calls 3171 · spent 0.297
- 网络盘 bzpbedqdsi 200 GB，用 163 GB

## 待人定
- `state/pending-decisions.md` 第〇·十一组已全关；#233 测不测速（约 5 min、0.5 美元）只在对话里提过，未登记

## 后台
- 旧会话有一个 `sleep` 兜底定时（无副作用，随旧窗口关闭）；无锁

## 规则提醒
- 本机重活前看 uptime（>20 等）与 `_tmp/heavy.lock`（同机 history 项目，大总管 history-meta）
- 手下换线：先等「可以关」→ 大总管建分支 → `close-worker.sh` → `spawn-worker.sh … --cmd "claude --continue"`
- 验收：独立副本（scratchpad）重跑；trainer 测试用 `noise_canceling-trainer/.venv`，data 测试用 `noise_canceling-traindata/.venv`
- 花钱超过人看到的数字先问人；时限按实测最慢 +50%

## 先读
coordinator-dispatch skill · `state/coord.md` 末 40 行 · 本页 · `docs/plan/001_decisions-20261029-b1.md` 末尾 D110–D123 · `state/pending-decisions.md`
