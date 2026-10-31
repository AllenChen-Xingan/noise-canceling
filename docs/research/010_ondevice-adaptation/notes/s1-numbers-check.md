# s1 · 怀疑派 A：核数字与出处

> 2026-10-30 · 只联网读 + 读 PDF，没跑程序。判：✅成立 · ⚠️出处不支持/过度概括 · ❌推算有误 · 🏷等级标错。

## 必须改

| 条目 | 原说法 | 核实结果 | 出处 |
|---|---|---|---|
| r1 LVC 饱和 | 41 耳 2 周 2.4 dB；再戴 2 周降到 0.95 dB → 2–3 周饱和 | ⚠️ 2.4 dB 是 41 耳（22 人）✅；**0.95 dB 只有 4 名用户**续戴，「2–3 周饱和」是 n=4 的结论，须注明 | Leenen 2007 p511–512 |
| r2 预算 | AdaBN 一次前向 ≈0.2 TMAC | ❌ 10 min 素材只过一遍：600×33M≈**2×10¹⁰ MAC=0.02 TMAC**；0.2 是把 10 轮也算进去了 | 【推算】 |
| r2 仿射参数 | BN 22×2×~16≈700 | ❌（量级不变）实际 BN 通道：ConvBlock 16+16+16+2，GTConvBlock (16+16+8)×6 → 290 通道×2=**580**；LN 4×2×33×16=4224；合计≈4.8K，约 10% ✅ | gtcrn.py L101/118/124/128/196/200 |
| r2 学坏 | EARS-D 上「两种 TTA 都不如源模型」 | ⚠️ 原文只就 **AM 模型**这么说；CMGAN 上 RemixIT 接近源模型 | arXiv 2509.04280 |
| r3/r4/r2 互相打架 | 1 h 素材训练：r4 ≈6 min；r3 的 RTF 法 = 3600×3×0.155≈**28 min**；r2「分钟级」 | ❌ 同一件事差约 5 倍，要统一假设（建议取 r3 上界 28 min，注明 r4 的 1 GMAC/s 只是假设） | 【推算】 |
| r3 实时功耗 | P 核 2.5 W×4%≈≤0.1 W，「上界」 | ⚠️ 只算了核心功率，没算簇/非核心和每 16 ms 唤醒；同一作者 2026 年实测「总能量比只有 11」，远小于核心功率比 200 → 这是**下界**，不是上界 | eclecticlight 2026/06/29 |
| r4 DAS | `thermalLevel>0` 时一律不放行 | ⚠️ 原文只有 mediaanalysisd **一个日志例子**，不能概括为规则 | eclecticlight 2023/01/23 |
| r5 pDNSMOS | 「专门惩罚旁人声」引 `dnsmos_local.py` 第 108 行 | ⚠️ 第 108 行是 `else:`；107 行只是加载 pDNSMOS 模型，源码里没有惩罚旁人声的字样 → 改引 DNS Challenge 5 论文，或删掉这条 | dnsmos_local.py L104–108 |
| r5 邻座陷阱 | EER 相对 +46%（3.59→2.05 s） | ✅ 数字对（8.72→12.8%，VoxCeleb1）；⚠️ 它讲的是**短语音声纹验证**，不是「邻座被当成主人」，是相关证据、不对题 | arXiv 1810.10884 表 1 |
| r5 老虎机 | K=4、Δ=0.3 约 270 次 | ⚠️ 量级对；但 Kaufmann 给的是**下界**，按 2σ²/Δ²·ln(1/2.4δ) 算约 190，实际算法要乘几倍 → 写「≥200，实际可能上千」 | arXiv 1407.4443 |
| r6 NaN 开销 | 每帧 160 样本、每帧约 33 万 MAC | ❌ 混用了 10 ms 和 16 ms 两种帧长。按 16 ms 帧：256 样本、33M×0.016≈**5.3×10⁵ MAC**，约占 0.05%；结论「<0.1%」仍成立 | 【推算】 |
| 🏷 Widex 84% | r1 标【官方】，r5 同一件事标【二手·厂商作者】 | 作者 3 人都是 Widex 员工，发在行业刊物上 → 统一标【官方·厂商自测】；另外「20 次迭代」是图注里的宣称，不是实测 | hearingreview（Townend/Nielsen/Balslev） |

## 核过成立

| 条目 | 核实 |
|---|---|
| Phonak 7 类、每 0.4 s、3 个互斥类、最多混合 3 类、200+ 种设置 | ✅ AB 白皮书 p1（这是人工耳蜗版，Sonova 同门） |
| Phonak 35 维特征；92% / n=149,675；n=14 | ✅ Rakita & Jones 2015 p1–3 |
| ReSound 4 s / 1 dB/s / 222 ms | ✅（原文写的是「4 s」，没有「≤」） |
| Yellamsetty：≤9 类，各类权重和 100%，38–100% | ✅；**67% 来自 Groth & Cui 2017（41–98%）**，不是本文自己的数字 |
| Signia SoundLearning 2.0：3 类、约 2 周、n=14、+20%、p<0.001 | ✅ 作者是 Siemens 员工 → 【官方·厂商】 |
| Nielsen 2015：10–20 次、5–10 min、≤4 参数、10 名受试者 | ✅ 摘要 |
| Starkey「预训练」「想做但没做」、DNN 在主音频通路 | ✅ CTO Bhowmik 原话 |
| GTCRN 22 BN + 4 LN | ✅ gtcrn.py |
| CCC：34.1/17.3、Tent 3.9/1.4、CoTTA 14.9/7.7、RDumb 49.3/38.9、每 1000 步、批 64；ViT EATA 38.5 vs 42.0 | ✅ 2306.05401 表 1 |
| SAR 表 4（批大小 1）：18.0→0.1；表 3：2.3、ViT 29.9→16.5 | ✅ 2302.12400 |
| MD-Long 61.0→100.3（CSUTA，N=1）；z>2 复位 | ✅ 2406.11064 表 2、§3 |
| 2601.14770 AM、CMGAN 数字；「the entire testset is used」；11.665→7.462 | ✅ |
| 2508.01847：2.961→3.015（MSP 任务的 bias 变体）、3.145、3.126→3.065 | ✅（3.015 只对应 MSP 任务） |
| E 核频率 M1 2064 / M4 2892 / M5 3048；M1 Pro E 簇 210 mW、P 簇 4 W | ✅ eclecticlight（按此推单个 E 核约 105 mW，取 0.3 W 是保守值） |
| EI 的 P/E 比 173、功率比 200 | ✅（作者补了一句：EI「夸大了」E 核省的电） |
| MBA 13" 53.8 Wh（M5） | ✅ apple.com |
| r3 的 (a)(b) 和 0.43% 的算式 | ✅ 复算无误 |
| `requiresExternalPower` 不支持原生 macOS；`isLowPowerModeEnabled` 从 macOS 12.0 起 | ✅ Apple docs JSON |
| ProcessObjectList 首见于 14.2 SDK | ✅ 只能算【二手→推算】：头文件 diff 证明它在 Xcode 15.1 加入，官方页面没标版本 |
| QoS 9 只跑 E 核、不能升到 P 核 | ✅【二手·实测】原话 |
| Chrome 看门狗 3 次 × 1 min（3 min ÷ 3）；sync_reader 非 Mac 0.95、Mac 0.5，超时补零 | ✅ L34/37/82–86 |
| WebRTC denormal「最慢可到 100 倍」、chromium:1227566、开关 `WebRTC-ApmDenormalDisablerKillSwitch` | ✅（开关在 0441bb62 的 diff 里，提交信息里没写） |
| opus 6 fuzzer × 4 = 24 | ✅ |
| MXMetricManager 从 macOS 12 起可用，27.2 弃用，替代是 MetricManager | ✅ |
| Hey Siri 5 条注册 + 上限 40；Google FL「idle, plugged in, free wireless」 | ✅ |
| DNSMOS 16 kHz、9.01 s、每 1 s 滑动；PCC 0.94 / 0.98 / 0.98 | ✅（「模型级」这个说法摘要里没写） |

未核（时间所限）：Vyas 中位数 25 次、Krisp CPU <2–3%、Face ID / Photos 原文、Oticon 1200 万。
