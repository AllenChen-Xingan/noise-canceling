# GTCRN 核对（预热段补零版）：不通过，根因是预热段

- `check.md`：内置 GTCRN（默认保命）对比外部目录默认预混（预热段补零）+ 关保命，480 段。154 段降噪量不同，**不通过**
- `bias-zero-head.md`：两者的配对差（外部 − 内置），即 exp016 里外部入口相对内置模型的口径差（#129）
- 修正后的核对见 `../20261105-parity-gtcrn-head/`（带真实预热段，逐段差 0）

## 根因诊断（实测）

worktree 根目录，需要 GTCRN 权重：

```sh
VDN_GTCRN_ONNX=<主目录>/resources/gtcrn/stream/onnx_models/gtcrn_simple.onnx \
  target/release/exp021-remix diag \
  --clips docs/experiments/exp013-scene-suite/builds/20261103230756-17f5/clips.jsonl \
  --clip home-quiet-c01-mba
```

2026-11-05 的输出（两段）：

| 段 | 判为有声的帧数：真模型 / 补零回放 | 补零回放第一处不同 | 同进程两次原始输出不同 | 补回真实预热段后与真模型不同的样本数 |
|---|---|---|---|---|
| bus-metro-cabin-c00-ext | 687 / 682 | 第 0 个样本 | 0 | 0 |
| home-quiet-c01-mba | 606 / 596 | 第 0 个样本 | 0 | 0 |

两段的引擎故障计数都是 0（没有复位、没有送原声）。
机制：引擎的有声包络只看模型输出能量、从第一帧起跟踪底噪（`src/dsp/src/envelope.rs` 第 82–107 行）；外部回放在模型延迟内补零（`src/eval/src/system/external.rs` 的 `replay`），真 GTCRN 这段有输出（峰值约 1e-3 到 3e-2），底噪起点不同，整段开合分岔。
回归测试：`driver/src/tests.rs` 的 `head_makes_remix_equal_real_model_path`（合成假模型：带预热段 = 直接走引擎，补零 ≠）。
