# 现状页 · 手下 finetune（调研 027 · #121 · exp018）

- **大事**：GTCRN 加训（D86）· 本线只看「在现有权重上微调 + 训练数据怎么配」（2026-11-05 范围调整；注意力头归 arch 028，强化学习归 rl 029，辅助小模型 / 混回比例归 aux 030）
- **管的实体**：GTCRN 权重 · 训练数据集 · 微调方法
- **分支**：`research/027-gtcrn-finetune` · worktree `../noise_canceling-finetune`

## 已做

1. `.gitignore` 加训练产物规则（`*.pt` `*.pth` `*.ckpt` `*.safetensors` `*.npz` `/runs/` `wandb/` `lightning_logs/` `tb_logs/`）；不写裸 `runs/`，以免忽略已入库的 `docs/experiments/*/runs/`
2. 单步耗时冒烟（实测，已发大总管）：脚本 `docs/experiments/exp018-gtcrn-finetune/smoke/step_timing.py`（unittest 3 条全过）；读数 `docs/experiments/exp018-gtcrn-finetune/runs/20261105-145149-step/`
   - 批量 8：CPU 0.368 s / 步，MPS 0.593 s / 步；批量 32：CPU 1.346 s，MPS 0.821 s（每段 4 s 音频）
   - MPS 回落算子 0 个（局限：回落截获逻辑只用合成警告测过）
3. 读入 + 在线混音冒烟（实测，已发大总管）：脚本 `smoke/mix_timing.py`（unittest 5 条全过）；读数 `runs/20261105-145445-mix/`
   - 单进程 366–474 段 / 秒；4 进程 785 段 / 秒（≈ 3,100 小时音频 / 墙钟小时），是训练 156 小时的约 20 倍——不是瓶颈
   - 局限：系统缓存已热、未测冷盘；未测和训练同时跑时抢 CPU
4. 收到 cands 的「GTCRN 弱在哪」（`../noise_canceling-cands/docs/experiments/exp016-candidates/runs/weak-spots-20261105/`）：弱项 83 组定训练数据，守住项 71 组当「不许退」清单

5. MPS 正确性（实测）：不带绕法不过（前向差 61%）；根因是 PyTorch 带空洞逐通道卷积的 bug（不是 GRU）；`smoke/mps_fix.py` 绕开后正式尺寸全过。已报大总管
6. 下载清单 `docs/research/027_gtcrn-finetune/download-list.md`：第一批 15.4 GB、第二批 67 GB；已报大总管，**本线不自行下载**
7. 调研 027 初稿：README · decision（比较卡 + 待人拍板 F-1..F-3）· sources · notes/；exp018 README

## 方案里必须写到的（大总管 / rl 转来）

- 损失加两项：人声被压低（投影能量比，只罚超过约 3 dB）· 字头 50 ms 能量损失（只罚偏低）；**每项写一条依据**（论文出处，或「我们自定，待实验」）和**权重系数怎么定**
- 字头 · 背景跳变 · 音色容许量引用 rl 029 R-2，不另起
- 评分模型也偏爱「多压」（029 实测）：听感分涨不能单独当证据
- 验收分两种口径：成绩单（引擎混回）与 `safety: false`（模型本身），各量哪几项
- MPS 若和 CPU 对不上，训练时长按 CPU 算并写原因
- 下载清单先报大总管，自己不下；每条写许可出处 · 能否训练（D64）· 大小 · 时长 · 覆盖弱场景 · 是否和评测集撞源

## 下一步

1. 16:00 后（大总管通知）带绕法重测 MPS 单步耗时（批量 8 · 32），填进 decision 比较卡与 exp018 README
2. codex-review 1–2 轮（审结论是否被证据支撑），落 `docs/reviews/027_gtcrn-finetune-codex.md` + 索引一行
3. grep 本机路径为 0 → 报「finetune 可以收尾」
4. 请大总管：`docs/experiments/README.md` 索引加 exp018 一行（不在本线可写文件里）

## 待人拍板

见 `docs/research/027_gtcrn-finetune/decision.md` 第八节：F-1 下第一批数据 · F-2 第二批（CochlScene）何时下 · F-3「户外变好、室内持平」算不算可上线。守住项容许量用 029 的 R-2，不重复问
