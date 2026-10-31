# ADR 003 · 模型接口：波形 hop trait + 引擎静态分发 + 手写 enum 选模型

- 状态：提议（2026-10-30，rustarch）
- 规约落点：`specs/rust/architecture.md` §3

## 背景
D33：GTCRN 与 FastEnhancer-M 人耳打平、证据弱，P1 起两个都接。命门 3：加一个模型 = 加一个适配器文件 + 一行注册，engine 与 eval 不改。p1 已写 `trait Model: Send { name, sample_rate, hop, latency_samples, reset, process }`、`Engine<M: Model>`、`enum AnyModel`（含小宏 `each!`），CLI 用 `AnyModel`。p1 `model.rs` 注释定了换模型的方向：外层同时持有两套「模型 + 分块」，补齐到较大延迟后交叉淡化，不塞进 Engine。

## 决定
1. 保留 p1 的**波形接口**（一个 hop 进、一个 hop 出）与 `Send` 约束；构造在非实时线程，实时线程只调 `process` / `reset`
2. 引擎 `Engine<M: Model>` 静态分发；运行期选模型用 p1 的 `enum AnyModel`（小宏 `each!`，不引入 `enum_dispatch` crate），不用 `Box<dyn Model>`
3. 模型之后的处理用 `trait Stage`，链是固定字段 struct，按设计稿 §3 的固定顺序静态调用
4. 两层入口：`Engine::process(input, output, ctx: FrameCtx) -> Status`（模型采样率，一个模型 hop）；`Pipeline::process(input, output, ctx)`（IO 采样率，任意长度：重分块 → 下采样 → Engine → Stage → 上采样 → 按 Status 与 48 kHz 原声混合）。`Status` 用新定义的 `BypassReason`，不复用故障注入器的 `FaultKind`
5. 换模型照 p1 的方向：外层持有两套 Pipeline，都补齐到较大延迟，交叉淡化后丢旧的；`L_total` 启动时按所有启用模型的最大延迟定死；新 Pipeline 在非实时线程建好后经 rtrb 送入，旧的经回收 rtrb 送回 drop，回收队列满就留在本地槽位

## 理由
- 外层统一成波形：GTCRN 自做 512/256 STFT【源】，FastEnhancer 有自己的前端；engine 不必知道谁做 STFT。设计稿「共用一次 STFT」以后用扩展 trait 补，不动 `Model`【推】
- 实时与非实时分开（构造 vs process）是 nih-plug（`initialize` / `process`）、firewheel（`AudioNode` / `AudioNodeProcessor`）、fundsp（`Net` 前端 / 后端）的共同做法【源】`notes/r3-traits.md` §1
- 每秒约 62 次调用，虚调用开销可忽略；动态分发真正的风险是在实时线程上 drop Box【推】`notes/r3-traits.md` §3。enum 是封闭集合（2 个变体），值经队列交接，没有 trait 对象释放问题
- Hobson（ADC 2025）：音频引擎对 DSP 模块泛型化 `AudioStream<M: AudioModule>`；控制面用封闭 enum 消息【官·幻灯片】https://hobson.dev/talks/adc_25
- 统一 `process(…, ctx)`：时间用样本计数传入（sans-IO），eval、proptest、RTSan 回放、golden 对拍都调同一个入口【推】`notes/r5-talks.md` §5 第 1 条

## 代价
- 每加一个模型，`AnyModel` 多一个变体
- 延迟较小的模型（换模型时）也要多付到最大延迟；双跑淡化期间 CPU 翻倍，不够就停流重建（与 nih-plug「改延迟可能重启播放」同一规则）`notes/r3-traits.md` §5
- p1 的 `Engine::process` 签名在 P2 要改（加 `ctx`、返回 `Status`），并新增 `Pipeline`
