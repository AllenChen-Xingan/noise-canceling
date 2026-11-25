# exp013 本机与云上 build 交叉核对（2026-11-25）

> 本机 build `20261124165427-6323`（macOS · arm64）对云上 build `20261125022300-aa9f`（cloudrun · RunPod CPU 机 · Linux）。
> 云上 build 的小文件原样放在 `builds/20261125022300-aa9f/`（取回目录与仓内逐文件 `diff -r` 相同）；音频只在云上网络盘 `/workspace/exp013-audio`。

## 结论

**两份 build 不是逐字节相同**。「逐字节一致」这条验收在跨平台时达不到，已改为按容差核对（见「定了怎么办」）。
原因（推断，有证据，没在两台机上逐步追到）：两台机器的浮点数学库不同（macOS arm64 与 Linux），同一个随机数推出来的参数在最后几位就不一样了。所以每段的音频样本都有微小差别。

## 证据（实测，`compare_builds.py` 与逐字段对比）

| 核对项 | 结果 |
|---|---|
| `compare_builds.py`（云上跑一次、本机复跑一次） | 不一致 1086 处，两边输出相同；退出码 1 |
| manifest：代码 · 参数 · 素材索引 · 依赖版本 · 基础种子 | 全部相同；素材索引 `a8de10ad…`，1994 个文件 |
| 泄漏检查 · 电平验收 · 各场景段数 · 排除表 `exclude.json` | 全部相同 |
| 逐段逐轨样本哈希 | 480 段里没有一段完全相同：owner 480 段 · owner_early 480 段不同，mix 177 段 · interf 279 段 · noise 142 段，分轨 1122 条 |
| 非哈希字段里的数值 | 最大差 4.5e-13（`sounds.src.params.lowpass_hz`）；`chain.direction_nodes_db` 差 3.6e-15。量级是浮点最后几位，不是逻辑差别 |
| 四舍五入后的实测值（SNR · SIR · 电平 · T60 等） | 相同 |

## 定了怎么办（大总管 noise-canceling-df 定，2026-11-25，技术题，人未参与）

- **评测只用一份音频**：本机 build `20261124165427-6323` 的音频（3.6 GB，5020 个文件，按 `hashes.txt` 全部核过）传上云，云上评测也读它；`src/eval/suite/scenes.json` 不改。原因：`vdn-eval` 读每段前核 wav 的 sha256，云上自己出的 `aa9f` 音频哈希对不上，每段都会记成「没跑成」
- **`aa9f` 只作「跨平台可复现」的证据**，不拿来评测
- **跨平台等价验收**（`compare_builds.py --tolerance`，阈值正本 `TOL_ABS` 在该文件里）：抽 5 段不同场景（cafe-c00-mba · meeting-room-s02-mba · street-outdoor-c00-ext · bus-metro-cabin-c00-ext · home-quiet-s01-mba），每段全部轨含分轨：
  - 样本数相等
  - 逐样本最大绝对差 ≤ 1e-9（约 −180 dBFS）。cloudrun 实测一段的最大差是 3.6e-15
  - manifest 的素材、代码、参数哈希逐位相等
- 测试：`tests/test_compare_tolerance.py`。差 2e-9 必须判「不一致」并点名哪段哪轨；只差浮点最后一位必须判「一致」；样本数不同、素材索引不同、段只在一边都判「不一致」
- 结果（实测，2026-11-25，cloudrun 取回 5 段共 45 个 wav）：**4 段过，1 段不过**，按规矩停下，待大总管重定阈值

| 片段 | 各轨逐样本最大差 | 判定 |
|---|---|---|
| cafe-c00-mba | 2.91e-11 | 过 |
| bus-metro-cabin-c00-ext | 1.46e-11 | 过 |
| home-quiet-s01-mba | 7.28e-12 | 过 |
| street-outdoor-c00-ext | 1.42e-14 | 过 |
| meeting-room-s02-mba | 1.79e-07（owner · owner_early · mix）；owner_dry 1.19e-07；noise 5.96e-08 | **不过**（8 条轨超 1e-9） |

45 条轨里 25 条逐样本完全相同。

**不过那一段的诊断**（实测）：这段同时有「全留多人」和「余量调整」（`headroom_db` = −1.313，两边相同）。
- 整段有一个约 2e-9 的相对增益差：云上对本机的样本比值均值 −2.8e-9（owner）、+2.1e-9（noise）。余量系数由整段峰值算出，跨平台差在最后几位
- 差信号比原信号低：owner −138 dB、mix −140 dB、noise 和分轨 −157 到 −158 dB，听不出
- **阈值 1e-9 比 wav 本身的精度还细**：wav 存的是 32 位浮点，幅度 0.5–1 的样本最小单位是 5.96e-8，比 1e-9 大 60 倍。整段增益差一点点，响的样本就会差整数个最小单位（这里最大 3 个：1.79e-7 = 3 × 5.96e-8）。另外 4 段过线，部分是因为没碰上这种情况，不能说明 1e-9 一般做得到（推断）
- 建议，待大总管定：改按 32 位浮点精度判，例如「每样本差 ≤ 4 个最小单位（按两边较大的幅度算）」并且「整轨差信号比原信号低 ≥ 120 dB」

## 怎么复现

```
python docs/experiments/exp013-scene-suite/compare_builds.py \
  docs/experiments/exp013-scene-suite/builds/20261125022300-aa9f \
  docs/experiments/exp013-scene-suite/builds/20261124165427-6323
```

跨平台容差核对（5 段 wav 取回后）：
```
python docs/experiments/exp013-scene-suite/compare_builds.py --tolerance \
  docs/experiments/exp013-scene-suite/builds/20261124165427-6323 <主目录> \
  docs/experiments/exp013-scene-suite/builds/20261125022300-aa9f <主目录>/_tmp/20261125-cloudrun-exp013build/audio-root \
  cafe-c00-mba,meeting-room-s02-mba,street-outdoor-c00-ext,bus-metro-cabin-c00-ext,home-quiet-s01-mba
```

## 阈值第 2 版（大总管 2026-11-25 定，在看到第 1 版结果之后改）

- **第 1 版**「逐样本绝对差 ≤ 1e-9」不合理：wav 存 32 位浮点，幅度 0.5–1 的样本最小单位是 5.96e-8，比 1e-9 大 60 倍。第 1 版结果（上表，meeting-room-s02-mba 原始读数 1.79e-7）保留，不删
- **第 2 版**：「每样本差 ≤ 4 个 32 位浮点最小单位，按两边中较大的幅度算」并且「整轨差信号比原信号低 ≥ 120 dB」。依据是数据的存储精度。正本 `compare_builds.py` 的 `TOL_ULP = 4`、`TOL_DIFF_DB = 120`；测试先红后绿
- **第 2 版结果（实测）：5 段全不过，16 条轨超线**。差信号那条全过：最高 −138.3 dB（meeting-room-s02-mba），其余 −232 到 −300 dB。不过的是「每样本几个最小单位」，读数高达 3.3e7 个
  - 原因：按每个样本自己的幅度算最小单位时，接近 0 的样本的最小单位极小，例如 1e-33 附近约 1e-40。绝对差只有 1e-15，也会折成几千万个最小单位。这一点我设计时就料到了，但没在实现前提出来（evalcmp 的疏漏）
  - 同一批数据改按「整轨峰值处的最小单位」算（每轨一个数）：5 段最多 3.0 个（meeting-room-s02-mba 的 mix），其余 ≤ 0.002 个
- 建议第 3 版，待大总管定：最小单位按整轨峰值算，其余不变

## 阈值第 3 版（最后一版，大总管 2026-11-25 定）

- **判据**：每样本差 ≤ 4 × 整轨峰值处的 32 位浮点最小单位（两边峰值取大的，每轨一个数）；整轨差信号比原信号低 ≥ 120 dB；样本数相等；manifest 的素材、代码、参数哈希逐位相等。正本 `compare_builds.py` 的 `TOL_ULP = 4`、`TOL_DIFF_DB = 120`
- **evalcmp 的疏漏**（如实记）：第 2 版「按每个样本自己的幅度算最小单位」在接近 0 的样本上必然失真。这一点设计时就料到了，但没在实现前提出
- **防过拟合**：阈值已经按旧 5 段改过两次，旧 5 段不能再单独当证据。另挑 5 段没看过的，**挑法在取回音频之前写在这里**：
  - 「全留多人 + 余量调整」：本 build 只有 meeting-room-s02 的两个设备版本，其中 -mba 已用过，只能取 **meeting-room-s02-ext**（与 -mba 同一份素材，不完全独立）
  - 其余 4 段：从没用过的 10 个场景里，用固定种子 20261030（`SC-AGG.seed`，Python `random.Random`）抽 4 个场景，每个场景再随机抽 1 段：**home-family-s03-ext · library-classroom-c01-ext · open-office-c08-ext · rail-air-cabin-c08-mba**
- **停止线**：新 5 段在第 3 版下还不过，就停下报大总管，不再改阈值；「跨平台等价」改记为「未证实」，并写出差信号的 dB 读数
- 新旧 10 段都按第 3 版判

### 第 3 版结果（实测，2026-11-25；cloudrun 取回第二批 52 个 wav，逐个核过 sha256）

**新 5 段（预先登记，没调过阈值）：全过**，`compare_builds.py --tolerance` 退出码 0。**旧 5 段：也全过**，退出码 0。

| 片段 | 轨数 | 每样本最大差（整轨峰值处最小单位的倍数，线 4） | 差信号最高（dB，线 −120） | 批次 |
|---|---|---|---|---|
| meeting-room-s02-ext | 10 | 2 | −140.6 | 新 |
| home-family-s03-ext | 11 | 1 | −142.4 | 新 |
| library-classroom-c01-ext | 11 | 3.05e-05 | −263.8 | 新 |
| open-office-c08-ext | 11 | 9.54e-07 | −291.1 | 新 |
| rail-air-cabin-c08-mba | 9 | 0.125 | −200.9 | 新 |
| cafe-c00-mba | 9 | 0.000977 | −232.3 | 旧 |
| meeting-room-s02-mba | 10 | 3 | −138.3 | 旧 |
| street-outdoor-c00-ext | 8 | 2.38e-07 | −300.1 | 旧 |
| bus-metro-cabin-c00-ext | 8 | 0.00195 | −232.4 | 旧 |
| home-quiet-s01-mba | 10 | 0.000244 | −241.2 | 旧 |

**结论**：跨平台（macOS arm64 对 Linux x86_64）按第 3 版判，10 段 97 条轨全部等价；最紧的是两段「全留多人 + 余量调整」（3 个、2 个最小单位，差信号 −138 到 −141 dB）。
- 局限：新 5 段里的 meeting-room-s02-ext 和旧的 -mba 是同一份素材，不完全独立；10 段只占 480 段的 2%，是抽查不是全量
- 评测仍只用本机 6323 这一份音频（云上也读它），这条结论只是「跨平台可复现」的证据
