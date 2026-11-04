# 评审 021 · 开会场景目录（codex · gpt-6-astra · read-only）

> 对象：`docs/research/021_meeting-scenes/` 的 catalog · coverage · decision · sources · check_catalog.py · test_check_catalog.py（type research）。**最新一轮：第 1 轮（2026-11-03）· 总评「修改」· 待第 2 轮**。
> 累计处理：P1×3、P2×4 逐条回文件核实，属实 7、驳回 0、无法核实 0。新出的要人定的事：无（第 3 题题面已改，默认不变）。

## 第 1 轮（2026-11-03）· 总评「修改」

原文：`_tmp/reviews/catalog-codex-1.md`（不入库）。effort medium，耗时 72 秒。行号指评审时的版本。
本轮 P0/P1：3 条 · 停止线：未满足 · 留作 issue 的 P2：无

- **P1｜coverage.md:31 ● 判定违反本页自己的规则**（第 1 轮 #1）
  - 核实：属实。第一节写「粉噪、白噪不算任何地点」，但 home-quiet、phone-booth、hotel-room、library-classroom 的 F2 和 home-quiet、library-classroom 的 F11 是靠 R01 / R03 这些粉噪任务记的 ●；open-office 的 F2 也是邻座任务里的粉噪底噪。catalog 的 library-classroom 还把 R03 记成 ci 1
  - 处理：这 8 格改 ◐。第一节补两条：F6 / F7 只看取值、不看噪声素材；只靠粉噪 / 白噪撑着的格子记 ◐。改的是 ● → ◐，○ 的个数和补的先后不变。catalog 的 library-classroom 改成 ci 0。「一个 ● 都没有的因素」由 3 个改为 4 个（加 F11），decision 结论 4 同步
- **P1｜coverage.md:99「只能自录」忽略了已有的反例线索**（第 1 轮 #2）
  - 核实：属实。sources S10 写 DNS 2020 实录含开放办公区；S24 写 Krisp 工作组含开放工位层、手机通话组含街道与地铁站。这两条都没核，就写了「只能自录」
  - 处理：coverage 第六节改为「023 卡片里没有，两条候选待核」；catalog 三个地点的「公开真实录音」列同步改。decision 第 3 题题面补现状，建议改为先等候选核完。核设备、主说话人、许可的活经大总管转交数据线（decision 第四节）
- **P1｜catalog.md:28 依据弱的 SIR 推断在难点和先后里写成了事实**（第 1 轮 #3）
  - 核实：属实。[S31] 的 61 dB 是观看者耳边、观看者自选音量；catalog 难点写「和用户一样响」，coverage 第 1 批写「F4 SIR 0 ± 6 dB」，都没标是假设
  - 处理：catalog 难点改「可能和用户差不多响（未量化，依据弱）【推断】」；coverage 第 1 批改「SIR 档位待定（测试假设 0 ± 6 dB，未验证）」；decision 结论 5 同步
- **P2｜sources.md:12 条目不足以核出主表数字**（第 1 轮 #4）
  - 核实：属实。S02、S13 只写了表名，没写主表用到的具体数值
  - 处理：S01、S02、S13 补上主表用到的每个值和表号
- **P2｜check_catalog.py:145 K5 没查 sources.md 自己的引用**（第 1 轮 #5）
  - 核实：属实。`run` 传给 `check_sources` 时去掉了 sources.md
  - 处理：四份文件全查；补测试 `test_sources_self_refs_checked_via_run`（经 `run` 入口，先红后绿）
- **P2｜check_catalog.py:132 K6 漏掉重复的矩阵行**（第 1 轮 #6）
  - 核实：属实。用集合比较，同名地点出现两行也不报
  - 处理：加「每个地点恰好一行」；补测试 `test_duplicate_place_row`
- **P2｜decision.md:59 把「先补评测」说成「降噪先变好」**（第 1 轮 #7）
  - 核实：属实
  - 处理：第 4 题后果改为「先能在哪个地方量出降噪的好坏、发现问题；补评测本身不会让降噪变好」，默认后果同步

### 引用抽查

- codex 列的「不符」：1 条（R01 粉噪被算成地点覆盖），即 #1，已改
- codex 列的「核实属实」：5 条
- 无法核实：D62 正本不在本轮可读清单；第 2 轮把 `docs/plan/001_decisions-20261029-b1.md` 加进依据

### 是否再审

有 P1 → 改完开第 2 轮。改后 `check_catalog.py` 0 处问题，测试 20 条全过【实测】。
