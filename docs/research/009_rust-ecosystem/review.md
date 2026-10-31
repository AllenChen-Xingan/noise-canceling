# 009 自审记录（2026-10-30）

三个对抗视角子代理只读审 `decision.md` + `answers-p1.md`：A 事实与出处（抽查约 32 处行号、27 个 crate、11 个仓）· B 实时音频工程 + 架构反对派 · C 许可证 + 外行读者。

| 级 | 条目 | 处理 |
|---|---|---|
| P0 | B1 voice-strip 环判缺后填零（静音），不是原声；D03 需要对齐延迟的原声 + 防抖 + 交叉淡化 | 设计 1 加「D03 补丁」四条（`shm.rs:178,211-215`、`plugin.rs:803` 本人复核）|
| P0 | B2 延迟预算没列账，默认参数约 56 ms 超 D32 | §一.3 加分项延迟账；重采样目标每跳 ≤ 2 ms；少一跳方案转 rustarch |
| P1 | A1 设计 3 把我们自己的双缓冲 preallocate 记到 biglinux 名下 | 拆开：biglinux = 状态进出 + 预热；双缓冲 = 本项目（p1 实测）|
| P1 | B3 FZ 开在推理线程等于对 ORT 开 FTZ（DFN #713） | FZ 只包 DSP 段 |
| P1 | B4 缺 NaN/Inf 防线 | §一.2 加一行 |
| P1 | B5 跨进程 workgroup「没先例」→ 系统已有 `os_workgroup_copy_port` | 改写，引 SDK 头文件 :79-118（本人复核）|
| P1 | B6 唤醒是系统调用，需列命门 ② 例外；推理线程也要 time-constraint | 改写 |
| P1 | C1–C3 白名单两份不一、漏 Unlicense/GPL-2.0-or-later/Unicode-DFS-2016；BSD-1-Clause、LLVM-exception FSF 未表态；MPL Exhibit B 查不出 | answers-p1 §5 更新；正本以 008 为准；CI 加 grep 规则 |
| P1 | C4 or-later 题漏代价：有 -only 依赖时整体只能按 -only 发 | 写进拍板 2 代价 |
| P1 | C5 权重 / ORT 二进制许可未进主表 | 加「发布阻断」标记 |
| P1 | C6 「照搬」「抄思路」措辞与铁律 3 冲突 | 统一为「学思路（读懂后自写，不复制代码）」|
| P2 | A2 栈上限写错（应为输入 ≤ 6、输出 ≤ 4）· A3 tract 证据行 · A4 rtsan build.rs 行 · A5 只有设 RTSAN_ENABLE 才下载 · A6/C7 拍板 1 夸大 · A7/C10 一句话把工具当依赖、hound 无行 · A8 FE 首选 dns 组 · A10 sonora 路径 · B2-2 漂移估计 · B2-3 设备切换 · B2-5 coreaudio-sys + mach2 · B2-6 Info.plist Mach 服务 · C8 14.4 无证据 · C9 放行口径不一 | 均已改 |
| P3 | B3 CoreML 题不该问人；开发者账号、蓝牙已由 D06 / M0 验收覆盖；C11 术语表；C12 代号混用；C13 默认写明 | 拍板从 5 题改为 4 题；加术语表与进程/线程关系一句；默认写明 |
| 未采纳 | B2-4 用 cubeb 设备重建替换设计 4 | 未逐行读 cubeb，不能给行号；列为候补。设计 4 直接对应 intent「离线与实时同一份代码」，保留 |
