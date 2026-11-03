# 要你用 sudo 跑的一条命令（耗电 · 实时线程设置）

用途：量每种线程设置下「每帧耗电」，回答「截止期报紧后，工作组能不能让降噪更省电」（推翻或确认 D74 的依据）。`powermetrics` 只能用 root 跑，所以要你输密码。

**什么时候跑**：大总管说「名额给 applebe、可以跑耗电」之后。跑的时候请不要开别的重活，插着电。约 35 分钟，跑完自己结束（3 轮，每轮先空闲 30 秒，再依次跑 10 种设置）。

**在终端里粘贴这一行**（会问一次你的开机密码）：

```sh
cd /Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling-engine/docs/experiments/exp010-inference-bench/apple-rs && sudo tools/rt-matrix.sh /Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/_tmp/exp010-apple/20261102-173000-b2 /Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/_tmp/exp010-apple/bin/rustbe-f9215181bcf6/exp010-bench /Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/_tmp/20261031-exp010/gtcrn_dns3 /Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/resources/gtcrn/test_wavs/mix.wav
```

**它会做什么**：
1. 先空闲 30 秒，量电脑本身的功耗当基线。
2. 跑 3 轮，每轮先空闲 30 秒量基线，再跑 10 种设置、每种 60 秒（苹果引擎 BNNS 6 种 + 手写 Rust 3 种 + ORT 1 种作对照），每轮换一个起点。每种都按真实节奏（每 16 ms 一帧）跑降噪，同时每 0.5 秒记一次 CPU / GPU / 神经引擎功率。
3. 最后打印一张表，最后一行是结果目录。

**权限**：只有测功耗的 `powermetrics` 用管理员权限；降噪基准程序和算结果的脚本都以你自己的账号跑（和产品一样）。结果目录先以你的账号建好。

**它不会做什么**：不改系统设置，不装东西，不联网，不在管理员权限下编译或运行我们自己写的程序。只写结果目录（在仓库的 `apple/results/` 下）。

**跑完请做**：把最后打印的那一行路径发给大总管就行。如果中途报错，把屏幕上最后几行发过来。


> 手写 Rust 用的是 rustbe 二进制的固定副本（/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/_tmp/exp010-apple/bin/rustbe-f9215181bcf6/exp010-bench，sha256 `f9215181bcf69d5021ad0713b35b4a3a171c69ea77cf1385f0a3e7d9c04a5afb`），rustbe 重编不影响这次测试。
