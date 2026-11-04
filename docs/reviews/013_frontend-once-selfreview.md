# 013 每帧只算一次频谱 · 多视角自审（2026-11-04）

- 对象：分支 `feat/frontend-once` 相对 `990e7843` 的改动
- 做法：`vdn-review` 工作流，3 个互相对抗的视角并行，只读源码；fecore 逐条核对后处理
- 收工判据：本线范围内 P0 / P1 / P2 为零；P3 酌情

## 本线范围内的意见与处理

| 级 | 意见 | 处理 |
|---|---|---|
| P2（两个视角都提了） | 借谱入口只查长度。别的模型借出 257 个频点、但帧移或窗不同时会被放行，场景特征会悄悄算错。`spectrum_spec()` 没有生产代码调用 | **已修**：`push_hop_with_power(x, spec, power)` 多收借出方的口径，每帧和 `FRONTEND` 比，不同就退回自算。新测试 `scene/tests/frontend.rs::mismatched_spec_falls_back_to_self_compute` 喂一份帧移 128、同样 257 个频点、内容故意改过的谱，要求结果与自算相同、每帧自算一次。`SpectrumTap::spectrum_spec` 的说明改成「吃谱方每帧比对」 |
| P2 | 场景的编译期断言只比 n_fft 和 hop，不比窗和采样率；运行期断言的报错文案手写「16 kHz / hop 256」 | **已修**：编译期断言加上 `sample_rate == MODEL_RATE` 和窗种类；报错文案改成引用 `FRONTEND` |
| P3 | 新测试里手写 `HOP = 256` 和 `new(MODEL_RATE, 256)` | **已修**：改为引用 `FRONTEND.hop`（`behavior.rs` 不是本线新写的，未动） |
| P3 | `frame.rs`、`polish/src/lib.rs` 的注释用文字复述「512 / 256 sqrt-Hann」 | **已修**：改为指向 `FRONTEND` |
| P3 | 计时例子没写该看哪一列、预期多少 | **已修**：文件头补两行 |
| P3 | `FeatureExtractor` 留着 hop、sr 字段，频点间隔自己算 | 不改：改成 `bin_hz()` 有可能动到浮点结果，红线是数值不变；字段已被断言锁在正本上 |
| P3 | 修音频谱级仍收散开的 `(n_fft, hop, sr)`，没有 `from_spec` | 不改：大总管定了修音的槽和接线留到以后（013 第 4 节），到时一起改 |
| P3 | models 的 dev 依赖 `vdn-scene` 用 path 写法，没走 workspace 依赖表 | 不改：根 `Cargo.toml` 不在本线可写范围；转交大总管 |
| P3 | `src/eval/src/system.rs` 的参考降噪器手写 512 / 256 | 不改：`src/eval/` 本线不许动；转交（该处是否应跟正本，需 eval 线定） |

## 本线范围外（视角顺带提出，本次改动没有引入）

都是改动前就有的问题，原样转交大总管记进待办：
- P1：`src/README.md` 还是初始化模板。
- P1：模块说明里有大量内部编号，没有白话解释。
- P1：「A / B / C 档」没有定义。
- P2：
  - 「档」一词用在三个意思上。
  - L0–L3 定标级别没有逐级说明。
  - verifier 代号没有中文解释。
  - `types/src/lib.rs` 开头写「分三块」，表格却有四行；帧长一处写 10 ms，另一处写 16 ms。
- P3：
  - NR、MOS 缩写没有展开。
  - 注释用评审轮次编号代替理由。

## 修后复跑（实测）

- `cargo clippy -p vdn-scene -p vdn-models -p vdn-types -p vdn-polish --all-targets -D warnings`：无告警。
- `cargo test -p vdn-scene`：全绿。`frontend.rs` 6 条，含新加的一条。
