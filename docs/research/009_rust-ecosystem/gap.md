# 两类发现 · Rust 生态：每个模块用哪个依赖、学谁的设计

> 2026-10-30 rusteco。细节见 `cards/`，取舍见 `decision.md`。

## 一、它们做到的（按三条命门）

| # | 命门 | 状态 | 差距 | 修起来要什么 | 分级 |
|---|---|---|---|---|---|
| 1 | GPL-3.0 兼容 | **已实现**（推荐的直接依赖全是 MIT / Apache / BSD / MPL-2.0）| ORT 原生库、模型权重、earshot 权重不在 Cargo 图里；TEN VAD 条款不兼容 | 接一根线：cargo-deny + cargo-about + 手补清单 | P1（发布前）|
| 2 | 实时线程零分配零锁零系统调用 | **部分**：rtrb、triple_buffer、realfft、rubato `process_into_buffer`、biquad、earshot 源码可证；ort Rust 侧用预分配可零分配（p1 实测）| ORT C++ 内部未测；cpal/coreaudio-rs 帧数变化时会分配；sonora、libDF、webrtc 绑定每帧分配；跨进程唤醒必然一次系统调用 | 接一根线：RTSan 一次实测；能力没有的部分（采集、HAL）自写 | P1 |
| 3 | macOS 可构建、仍在维护 | **部分**：核心依赖活跃；一批定型小库刚过 12 个月线 | 见 decision 待人拍板 1 | 人定口径 | P2 |

## 二、它们没做的

| 缺口 | 没顾上做 / 做不到 | 我们怎么办 |
|---|---|---|
| 跨进程共享内存 SPSC 环 / 原子发布的 crate | 没顾上（小众需求）；有 shmring、shm-rings、ringfire 等单人小仓，不建议依赖 | 自写，照 voice-strip 时间轴环 + rtrb 缓存索引 |
| HAL 驱动的 Rust 框架（成熟的）| 没顾上；tympan-aspl 只发过一版 | 照 voice-strip 自写 |
| 跨进程唤醒的 Rust 封装 | 没顾上 | libc `os_sync_wait_on_address` 自写 + 小实验 |
| 跨进程加入 IO 线程 workgroup | 没找到任何开源实例，可能是机会也可能做不到 | P3 实验 |
| 带 GRU 状态的模型热切换预热 | 没找到先例 | 照 Firewheel 换装协议 + biglinux 预热自己定 |
| 零分配的在线声纹注册 | 没顾上 | 自写，< 150 行 |
| 自动延迟补偿（两模型前瞻不同）| Firewheel 只声明 latency_frames，没用起来 | engine 自己补齐 |
| macOS 驱动 .pkg 签名公证的 Rust 工具 | 没顾上（cargo-dist 不覆盖）| 自写 shell 脚本 |
