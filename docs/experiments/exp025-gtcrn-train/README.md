# exp025 · GTCRN fine-tune (b19)

Plan: `docs/plan/014_training-kickoff.md`. Each worker writes only its own section below.

## 数据

训练数据的下载、切分、混音和加载接口都在 `data/`，详见 `data/README.md`（负责的手下：traindata，issue #156）。
- 混音在 GPU 上批量做（2026-11-24 起）。CPU 只读原始音频片段、抽随机数
- 训练目标只留前 100 ms 早期混响，这个长度的正本在 exp013 `rooms.py`
- 每个训练 build 都要先通过和 exp013 评测素材的交集检查，再登记进 `data/builds/`
