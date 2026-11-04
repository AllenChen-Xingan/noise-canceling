# 设备键表（022 · Mac 内置麦克风）

> 状态：**草稿 v0（2026-11-03）**，只用了 011 / 003 / 016 已有结论和 mtgdata 转交，调研未开始。数字列待调研补；查不到的写「未查到」。
> 用途：scenes、mtgdata 给数据和场景打「设备」标签时，一律用下表第一列的键。缺键发消息给 macmic 加，不自造。
> 来源标记：【实测】【官方文档】【源码】【论文】【论坛 / 博客，个人说法】【推断】。

## 一、键表

每行一种「声音从说话人嘴到降噪模型输入」之间的设备链。

| 键 | 是什么 | 通道（第三方拿到的） | 第三方经普通采集（HAL）拿到的处理阶段 | 已知处理 | 频响 / 底噪 / 自动增益 |
|---|---|---|---|---|---|
| `mba-builtin` | Apple Silicon MacBook Air 内置麦 | 1 声道 48 kHz【实测，011：Mac17,3 本机枚举】 | 已做阵列波束成形的单声道【官方文档：「Three-mic array with directional beamforming」】；不含语音突显（只对开了苹果语音处理的 app 生效）【官方文档，WWDC23 10235】 | 波束成形；其余（高通、降噪、自动增益）未查到 | 未查到 |
| `mbp-builtin` | Apple Silicon MacBook Pro 14 / 16 寸内置麦 | 待查（推断同上 1 声道） | 同上【推断】；官方写「studio-quality three-mic array」【待核原文】 | 同上 | 未查到 |
| `mac-intel-builtin` | 2020 年前 Intel MacBook 内置麦（老机型，含 2008 款） | 待查 | 待查 | 待查 | 未查到；MOBIO 有 2008 款录音（mtgdata 转交） |
| `imac-builtin` | iMac 24 寸（M 系列）内置麦 | 待查 | 待查（官方写三麦阵列，待核） | 待查 | 未查到 |
| `studio-display` | Apple Studio Display 三麦阵列（USB 接入） | 待查 | 待查 | 待查 | 未查到；Firefox 在它输入输出配对时禁用苹果语音处理【源码，011 #13】 |
| `airpods-hfp` | AirPods 作麦克风（蓝牙通话链路） | 1 声道，窄带【011 #2：「AirPods 也只给单声道窄带」】 | 耳机端已做降噪 / 波束成形【推断】；采样率待查（16 kHz 或 24 kHz） | 编解码压缩 + 耳机端处理 | 未查到 |
| `bt-hfp-generic` | 非 Apple 蓝牙耳机作麦（HFP，CVSD 8 kHz / mSBC 16 kHz） | 1 声道窄带 | 耳机端处理因品牌而异 | 窄带编解码 | exp006 现用「蓝牙窄带」近似（假设） |
| `usb-headset` | USB 或 3.5 mm 有线耳麦（嘴边近讲） | 1–2 声道 | 多数不处理，少数带硬件降噪【推断】 | 因品牌而异 | 未查到 |
| `external-mic` | 桌面外接麦（USB 电容 / 动圈，如 Shure MV5） | 1–2 声道 | 一般不处理 | 无 | 因型号而异 |
| `iphone-continuity` | 连续互通相机 / 麦克风：iPhone 当 Mac 的麦 | 待查 | 待查（可能带 iPhone 端处理） | 待查 | 未查到 |
| `laptop-unknown` | 笔记本内置麦，品牌型号未写（如 3D-Speaker 的 9.6 万条） | — | — | — | — |
| `non-laptop` | 数据集用的不是笔记本（手机、近讲参考麦、录音棚） | — | — | — | — |

## 二、和现在评测集的对应（推断，待 simulation.md 定）

- exp006 `recipes.json` 的 `device`：`none` ≈ `external-mic` 或 `non-laptop`；`bt_narrow` ≈ `bt-hfp-generic`；`clip` 与 `mic_tilt_db_per_oct` ±1.5 不对应任何一个真实设备，都是假设
- 016 F8 的 `laptop`（100 Hz 高通 + 7 kHz 低通）≈ `mba-builtin` 的猜测，无出处

## 三、待定（本草稿的问题）

1. Mac 内置麦是否要按机型年份再分（如 M1 Air 与 M3 Air）——看调研能否找到机型间差异
2. 「语音突显开 / 关」是不是设备键的一部分，还是另起一个「系统处理模式」字段——建议另起字段 `mic_mode`（standard / voice-isolation / wide-spectrum / unknown），不进键名
