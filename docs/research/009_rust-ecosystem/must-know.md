# 决策点 · Rust 生态：每个模块用哪个依赖、学谁的设计

> 2026-10-30 rusteco 起草（AI 草稿，待人过目）。依据：D04 GPL-3.0 · D31 能耗约 1/5 核 · D32 端到端附加 ≤ 45 ms · D33 GTCRN 主 / FastEnhancer-M 备 · D03 出事送原声 · `docs/plan/003_tech-design.md`。

## 我要作的决定

**每个模块（音频 I/O · 实时零件 · DSP · 推理 · 声纹/VAD · 宿主架构 · 工具链）直接依赖哪个 crate（首选 + 备选），以及最值得学的 5 个设计。**

## 一、不可妥协的

| # | 不可妥协的 | 为什么它是命门 | 怎么算满足 |
|---|---|---|---|
| 1 | **许可证与 GPL-3.0 兼容**（直接依赖的整棵依赖树） | D04；一个不兼容的传递依赖就让发布违法 | MIT / Apache-2.0 / BSD / ISC / Zlib / MPL-2.0 / LGPL / GPL-3.0 = 满足；GPL-2.0-only、SSPL、无许可、权重未声明 = 不满足 |
| 2 | **能在实时音频线程里用：处理路径零分配、零锁、零系统调用** | 音频回调里一次 malloc/锁竞争就爆音；D32 的 45 ms 预算没有余量吸收抖动 | 源码可证（文件:行号）其热路径不分配不加锁，或提供「预分配 + 只在初始化分配」的 API |
| 3 | **在 macOS（Apple Silicon）上可构建、仍在维护** | 本项目只做 Mac；没人维护的 crate 等于自己维护 | 最近 12 个月有提交或发布；macOS/aarch64 在 CI 或文档里明载 |

（第 4 条软约束：**离线与实时同一份处理代码**——依赖要能同时跑在 WAV 批处理和音频回调里；第 5 条软约束：**省电**，D31。）

## 二、知道了它们，我要作的是哪一个决定

每个模块填一行「首选 / 备选 / 理由」，并挑 5 个设计让 rustarch（008）抄思路、p1 立刻用。

## 三、排除项（知道了也不改变决定）

- 跨平台支持深度（Windows / Linux / WASM）——只做 Mac
- GUI 框架、合成器 / 乐器功能、MIDI
- 训练框架（D29 训练放最后）；burn/candle 只看推理
- 插件格式 VST3/CLAP 的分发细节（我们做虚拟麦克风 HAL 驱动，不做 DAW 插件）——只学架构

## 四、预判（请回源码推翻）

| 我怀疑 | 请核实 |
|---|---|
| ort 是推理首选（GTCRN 是 ONNX），tract 纯 Rust 但对流式 GRU/状态输入支持可能不全 | ort 版本与 onnxruntime 链接方式、是否线程内零分配；tract 能否跑 GTCRN 流式 ONNX |
| rtrb 是 SPSC 首选，ringbuf 备选 | 两者 push/pop 是否无锁无分配，`rtrb` 维护状态 |
| realfft + rustfft 是 FFT 首选，planner 预分配 scratch 后热路径零分配 | `process_with_scratch` 一类 API |
| rubato 有零分配的 `process_into_buffer` | 文件:行号 |
| cpal 在 macOS 底层是 coreaudio-rs，回调里不分配；但做 HAL 虚拟设备 cpal 帮不上 | — |
| DeepFilterNet libDF 的逐帧 STFT/状态管理是最贴近我们的参考实现 | 其 tract 流式推理与帧缓冲 |
| nih-plug 的 `assert_process_allocs` 与 Buffer 设计值得学 | 文件:行号 |
| assert_no_alloc 是不分配断言首选 | 维护状态、与 rtsan 绑定的对比 |
