# 现状页 · 手下 evalcmp（批次 20261116-b19）

## 当前：exp025 正式评测上云（2026-11-27 收工，人要求今晚停）

状态：暂停在原版成绩单这一步，明天续。我向大总管 `noise-canceling-meta` 报告；云上的机器由 `cloudrun` 开、停、取回。

**做到哪**
- 执行单正本：`docs/experiments/exp025-gtcrn-train/eval/cloud-run.md`（main 2a07e8ed 版本，加上 PR #235 的三处补记：改成 3 台（D123），`$OLD` 路径，B 对 A 的读法）。交给 cloudrun 的交代文件在主目录 `_tmp/20261126-evalcmp-to-cloudrun/brief.md`
- `EVAL_COMMIT` 钉在 `2a07e8ed2d77d30a700933f77727d48fc7229379`（main 之后多出的 #234 只改 trainer 文档，没动 `src/` 和 `eval/`）
- 机 1 `begw6apbx1rtl4` 上已经过的关口：
  - 6 个 best.pt 的 sha256 和清单一致
  - 6 个导出都过了对拍；原版 ONNX 的 sha256 是 384e2c68…
  - 账本起点：calls 3171，spent_usd 0.297396，写在 `batch-start.json`
  - 回归检查：两项都过，0 处变化；识别 0 次；用时 47 分钟；RUN=`20261127-015114-regress-gtcrn`
- 原版 `exp025-orig` 02:38Z 开跑。今晚是跑完取回还是中途停，看 cloudrun 的收工报告（它负责取回、删 pod、报账本读数）
- 判读副本 `../noise_canceling-evalrun`：分离头，HEAD 是 2a07e8ed，工作区干净；`target/release/vdn-eval` 已编译

**明天怎么续（新会话只看这里就够）**
1. 先读 cloudrun 的收工报告：原版跑完没有，取回目录在哪，账本读数是多少，云上是不是 0 台
2. 剩下的成绩单：如果原版跑完了，还剩 6 份（a-s0、a-s1、a-s2、b-s0、b-s1、b-s2）；如果没跑完，还剩 7 份。原版没跑完的那份按执行单第七节续跑，缓存用中断那份：`run_card external:_tmp/exp016/exp025-orig/formal1 exp025-orig $RUNS/<中断的 RUN>`
3. 开新机器前要做的三件事：
   - 核账本：`reserved_usd` 是 0，`review` 是空的；不是就照第七节核销
   - 沿用网络盘上的 `batch-start.json`，不要重写
   - 核花费：已花机器费加上下一台的最坏值不超过 1.92 美元（D123），识别批次累计不超过 1.2 美元
4. 新机器上 6 个 ONNX 不用再导出，已经在网络盘 `$OUT` 里（`SHA256SUMS`）。`REPO` 要用同一个 `/workspace/repo-2a07e8ed`
5. 原版那份出来后先核：识别次数约 1030 次，超过 1600 就停。然后按实测用时重估机器费，超过 1.92 美元报大总管
6. 7 份都回来以后，在 `../noise_canceling-evalrun` 里照执行单第六节跑：12 份 compare 和 2 份 accept。先看负载（1 分钟负载 > 20 就等）和 `_tmp/heavy.lock`
7. 判读时 B 对 A 按「A + 时间注意力」读：B 的条件调制层是恒等，见执行单第六节

**开着的 PR / issue**：#235（执行单补记，只改文档，等大总管合并）· #231（A 转流式，归 trainer）· #229（执行单）

---

## 旧：compare 子命令那一批（2026-11-24，issue #155）

状态：已收尾（2026-11-24，等大总管验收）

## 手上
管 3 个实体：评测器（`src/eval`）· 评测参考轨（exp013 生成器的参考输出）· 成绩单（规约 `specs/rubric/scorecard.md`）。
1. 配对比较子命令 `vdn-eval compare`
2. 早期回声参考轨 `owner_early.wav` + 成绩单旁列
3. 自审 + codex，连续两轮没有 P0/P1

## 做到哪
- 全部交付完成：compare 子命令 · 早期回声参考轨（生成端 + 成绩单旁列）· 规约第十 / 十一节 · 新 build `20261124165427-6323`（本机出，已提交小文件）· `src/eval/suite/scenes.json` 指向新 build
- codex 第 2、3 轮连续没有 P0/P1；整套重跑全绿（见流水 11-24 末几条）
- 下一步（不归我）：大总管亲手重跑、开 PR 并合并；合并后 cloudrun 在云上再出一份，用 `compare_builds.py` 与本机这份交叉核对

## 正本与交叉校验（CLAUDE.md 五.5）
| 东西 | 正本 | 第二处表示 · 校验 |
|---|---|---|
| 早期混响毫秒数 100 | `docs/experiments/exp013-scene-suite/rooms.py` `EARLY_REVERB_MS` | traindata `exp025-gtcrn-train/data/mix.py` import 它与 `early_rir`；对拍测试 `test_mix_loader.py`（traindata 写） |
| 有早期参考旁列的项 | `standard::EARLY_REF` | 按行为核对（`early_tests.rs`：换参考后读数变了的项必须正好是它）；规约第十一节表（`tests/scorecard_compare_doc.rs`） |
| 片段清单 `owner_early` 字段 | `scene.rs` `Tracks` | `specs/schema/eval-scene-clip.schema.json` 由 vdn-schema 导出，`--check` 比对 |
| compare 的三态名 · SC-AGG 参数 · 否决逐段判法 | `compare.rs` · `thresholds/table.rs` | 规约第十节三张表（`tests/scorecard_compare_doc.rs`） |

## 流水
- 11-16 报到；100 ms 正本与 trainer、traindata 商定放 `rooms.py`（大总管同意，要求旧 build 行为不变）
- 11-16 `rooms.py` 加 `EARLY_REVERB_MS` 与 `early_rir`；`compose.py` 加 `point_early`（不抽随机数）与 `owner_early` 轨；`manifest.AUDIO_KEYS` 加一项
- 11-16 实测：新代码重生成 build 17f5 的 3 段（cafe-c00-mba · meeting-room-s01-mba · street-outdoor-c00-ext），各轨样本哈希与旧清单逐个相同
- 11-16 `tests/test_early.py` 9 条过；旧 build 仓内 40 个文件哈希钉在 `tests/old_builds.sha256`
- 11-16 Rust：`scene.rs` 加可选 `owner_early`；`scorecard/early.rs` 旁列；`compare.rs` · `compare_out.rs` · `main.rs` 接 `compare`
- 11-16 名额内：scorecard 单测 65/65；vdn-schema 重导 schema（+7 −0，`--check` 一致）；external_e2e 12 · scorecard_e2e 11 · scorecard_standard 5 · scorecard_compare_doc 2 全过；exp013 Python 自检 77 条全过
- 11-16 按行为测出有早期参考旁列的项 10 个：V-ONSET V-WEAK V-PUMPV V-CLICK V-ESTOI V-TIMBRE V-SPLVL V-TSOS V-SWALLOW V-PSE-OVL
- 11-16 加 e2e `early_reference_side_column` 与 compare 真实打分压低 6 dB 测试
- 11-24 电脑休眠中断后接着做（人指示）。JSON 往返浮点差最后一位 → 改相对容差 1e-12，e2e 过；clippy -D warnings 过；xtask tidy 过；scorecard 单测 68/68
- 11-24 大总管 noise-canceling-e1 已不在会话列表，名额无法申请；按人「接着做」自行跑，记在此
- 11-24 compare 冒烟（只读分数）：a = dfn3-ll、b = dpdfnet2，旧 build 480 段全部配上；涨 10 · 退 8 · 不确定 5。延迟 V-LAT 判退（+46.25 ms）、b 新增否决 480 段。发现 Markdown 里否决段列表太长 → 最多列 10 个，全部在 compare.json
- 11-24 exp013 冒烟 build（4 段，写在会话临时目录，不进仓）：泄漏、电平验收都过，每段有 owner_early.wav；取 1 行做 Rust 解析夹具 `src/eval/tests/fixtures/scene-clip-early-example.jsonl`
- 11-24 traindata 已在 `exp025-gtcrn-train/data/mix.py:30` import `EARLY_REVERB_MS, early_rir`，并有测试和正本逐样本对拍（`test_mix_loader.py:64`）
- 11-24 本窗口的选项题「新 build 在哪出」收到了「上云」的回答。大总管 noise-canceling-33 事后说明：是它在 tmux 里代按的，人当时没看这个窗口。我无法核实是谁按的，按大总管说明记：这是大总管的决定（D97），人未参与
- 11-24 大总管 noise-canceling-33 接替 e1：认可此前自行跑的几次（下列）；批 cloudrun 同一台 CPU 机，和 traindata 合计最多 3 小时、1 美元。本分支合并后由 cloudrun 跑，小文件回到 `_tmp/20261125-cloudrun-exp013build/`，由我核 sha256 后提交到 `builds/<id>/`
- 自行跑的几次（无名额，补记）：11-24 scorecard_e2e 早期参考单条（220 s）· clippy 两次 · xtask tidy · scorecard 单测两次 · compare 冒烟（只读分数）· exp013 冒烟 build（4 段）· exp013 Python 自检
- 11-24 大总管发 1 个名额：codex 修完后整套 e2e 重跑用，跑完回报「名额已还」

## 新 build 与云上交叉核对（大总管 noise-canceling-33 定：本机、云上各出一份交叉核对；技术题，人未参与）
- 本机这份：`builds/20261124165427-6323`（480 段，42.3 s，泄漏检查与电平验收都过；每段有 owner_early；旧各轨样本哈希与 17f5 逐个相同 0 处不同）
- 素材索引：`material_index.sha256 = a8de10ada8c9f8215606613c88f82b8a234a459611941183e9b3212932a71b8e`，files = 1994（与 17f5 相同）
- 云上：本分支合并后，cloudrun 在网络盘上传本机素材（约 3.3 GB：test-clean 2707 个文件 356 MB · demand 160 个 1.7 GB · esc50 169 个 71 MB · music 1 个 · zh 6 个 · rir 442 个 1.2 GB），按 `requirements.lock` 装环境，再 `build.py --workers 4`
- 验收：`python docs/experiments/exp013-scene-suite/compare_builds.py <云上 build 目录> docs/experiments/exp013-scene-suite/builds/20261124165427-6323` 输出「一致」。按片段 id 逐轨比两种哈希 + 其余字段 + manifest 素材 / 代码 / 参数哈希；不比 clips.jsonl / hashes.txt 文件哈希，因为里面的路径带 build id
- 云上小文件回到主仓 `_tmp/20261125-cloudrun-exp013build/`，由 evalcmp 核对后提交（若那时本线已关，由大总管指派）

## 待人拍板
（暂无）

## 转交（范围外的发现）
- 吃字与两个评分模型（QA 层）的早期参考旁列本批没算：要换参考另跑模型，适合上云时一起做

## codex 评审记录（docs/reviews/ 不在我的写范围，记在这里；合并时大总管可转进 docs/reviews/）
### 第 1 轮（2026-11-24 · 模型 gpt-6-astra · 只读沙箱 · effort medium · 45 s）：P0×0 P1×1 P2×2 · 总评「修改」
- 第 1 轮 #1（P1，属实，已改）：否决统计把「判不了」当「没否决」。改成三态：两边都判得了才算新增 / 消失，其余进新列「一边判不了」。测试 `undecided_vetoes_are_not_counted_as_changes`（双向，覆盖缺项 · 判定未知）；规约第十节第 4 条同步
- 第 1 轮 #2（P2，属实，已改）：规约判断表只比名称。加 `Call::cond()` 作条件的正本，文档第二列逐字比对；另加 `call_boundaries` 按行为测端点恰为 0、两个方向、组数不够
- 第 1 轮 #3（P2，属实，已改）：早期轨「同电平系数」没独立验证。加 `SameCoefficients`：监听未定电平输出，每个要留的人用主轨解出系数，逐样本核早期轨；覆盖单人无余量调整（cafe-c00-mba）与全留多人且触发余量调整（meeting-room-s02-mba）。变异核对：删掉早期轨的余量乘法，这条测试变红
- 抽查「不符」1 条即 #2，已处理；「无法核实」2 条（bootstrap 内部、设备链）不在本次改动范围
### 第 2 轮（2026-11-24 · gpt-6-astra · 只读 · medium · 45 s）：P0×0 P1×0 P2×1 · 连续第 1 轮无 P0/P1
- 上一轮 3 条：codex 抽查都核实已改
- 第 2 轮 #1（P2，属实，已改）：报告没记实际比的系统，同一多系统文件选两个系统时分不出谁是基线。JSON `meta` 加 `a_system` / `b_system`，Markdown 路径后写「（系统 X）」；测试同一文件选不同系统、交换后跟着换
### 第 3 轮（2026-11-24 · gpt-6-astra · 只读 · medium · 43 s）：P0×0 P1×0 P2×1 · 连续第 2 轮无 P0/P1 → 满足停止线
- 上一轮 1 条：codex 抽查核实已改
- 第 3 轮 #1（P2，属实，已改，不另开 issue）：缺段测试删一段又补一段，两边段数仍相等。加 `unequal_clip_counts_both_directions`：b 只删 3 段（12 对 9），双向核配对数、只在一边的段、各项有效段数与差值
- 停止线：第 2、3 轮连续没有 P0/P1，已满足（codex-review 规则）；三轮的 P2 都已改，不另开 issue

## 整套重跑与偶发失败
- 11-24 16:31 整套跑（大总管预发的名额）：`scorecard_e2e::early_reference_side_column` 失败 1 次，断言「没跑成的段为空」（当时第 486 行），61.6 秒就停了；原因原文被输出过滤丢掉。clippy 1 条（多余 clone，已修）
- 单跑这一条通过（217.5 s）；并行跑 scorecard_e2e 13/13 通过（209 s）。同一时段本 worktree 没有别的测试进程（后台任务时间已核，大总管也确认）。本机另有 4 个别的仓库在跑 cargo test，耗时数字不能当测速基准
- 补了夹具隔离（真实隐患，根因未证实）：每次 setup 独立目录；会红的测试 `setup_dirs_are_isolated` 旧代码红、新代码绿。为守 600 行，夹具抽到 `tests/common/scene_fixture.rs`，早期参考 e2e 移到 `tests/scorecard_early_e2e.rs`
- 追踪 issue：#161（我开的，大总管要求）
- 11-24 约 08:42（北京时间 25 日）主仓 `_tmp/` 被人在另一会话批准的腾盘清理删掉（大总管查明，不恢复），含 exp002-venv、旧 build 17f5 音频、旧成绩单
- 11-24 按大总管指示：用 uv 重建环境 `_tmp/exp013-venv`（3.12.14 · numpy 2.5.4 · scipy 1.18.1 · soundfile 0.14.0，与 17f5 manifest 一致），冻结成 `exp013-scene-suite/requirements.lock`；README 与 build.py 的环境路径改过来
- 11-24 整套重跑（--no-fail-fast，完整输出 `_tmp/full-rerun-20261124-164525.log`）：vdn-eval 单元 171 · 各集成测试全过（含 scorecard_e2e 11、scorecard_early_e2e 3，并行）· clippy · tidy · schema 一致；Python 那一步缺旧 venv 失败（环境问题），新环境补跑 81/81 过
- 11-24 本机重出旧 build 17f5 音频到原路径：`hashes.txt` 4540 个 wav 全部 OK，480 段清单除路径外逐字段相同，素材索引相同——证明可重建。副作用：每段目录多一条不在旧 hashes.txt 里的 owner_early.wav
- 11-24 出新 build `20261124165427-6323` 并提交小文件；`tasks.py` 重写 `src/eval/suite/scenes.json` 指向它（只改了 clips 路径与 origin，120 行）；Python 81/81、`repo_scene_suite_parses_and_expands` 过
- 11-24 加 `compare_builds.py` + `tests/test_compare_builds.py`（3 条：换 id 一致 · 改一条轨哈希报出 · 素材索引与缺段报出）

## PR 正文要点（大总管开 PR 时用）
- Closes #155；新 build 由大总管 noise-canceling-33 定上云（D97），人未参与；合并后 cloudrun 在 CPU 机上出，小文件由 evalcmp 核 sha256 后提交
- 偶发失败照实写：只出现过 1 次，并行重跑 13/13 通过，补了夹具隔离，根因未证实，见 #161
- 新 build：`builds/20261124165427-6323`，本机与云上交叉核对用 `compare_builds.py`
- 加的测试：compare 单元 17 条（自比全不确定、真实打分压低 6 dB 判退、段数不一致双向、否决三态、判断边界等）· compare_out 2 条 · early 4 条 · e2e 早期参考旁列 + 生成行解析 + 夹具隔离 · 规约表比对 2 条 · exp013 Python `test_early.py` 10 条 · `test_compare_builds.py` 3 条

---

# 第二条线：exp025 原版 GTCRN 不回混基线（分支 `exp/025-baseline-scorecard`，大总管 noise-canceling-df 交代）

状态：代码与文档已收尾，codex 停止线满足（第 15、16 轮无 P0/P1），等大总管验收、开 PR；正式跑 480 段要另申请名额

## 交付
- `vdn-eval scorecard --no-safety`（不回混口径，`Suite::without_safety`），`meta` 记 `safety` 与 `asr`（后端、型号、各后端段数、不一致率、停发原因）
- 识别改走 OpenRouter（D103）：`src/eval/src/qa/asr_or.rs`（型号正本 `qwen/qwen3-asr-1.7b-20260813`；每个请求带 `data_collection = deny`；密钥经标准输入；退避重试；8 路并发）· `asr_ledger.rs`（账本：跨进程锁、原子落盘、先预留后结算、待核标记）· `asr.rs`（按段路由后端、预取、失败记忆与停发、`REPEATS` / `SPOT_FRACTION` / `MAX_UNSTABLE_RATE`）
- 外发按 D104（人定）：480 段都可送识别、存 RunPod 网络盘；exp013 README 补了 MIT、ACE 的指针
- `vdn-eval compare` 核同一批音频：`ClipRow::mix_sha256` 写进 `clips.jsonl`，两边哈希不同就拒绝，缺哈希计 `unverified_pairs`（大总管转来的 traindata 线索，属实）
- `docs/experiments/exp025-gtcrn-train/baseline/`：README（跑法、外发、估时与费用、验收、差距表判据）· `gap.py`（弱项表 / 提升空间表，完整性判据、边界并列）· `merge.py`（合并 / 只对拍，核同一批音频、并集比项）· `subset.py` · 测试 37 条 · 冒烟与提升空间预览结果

## 实测
- 5 段冒烟（本机 release，负载约 6.9，只作估时）：GTCRN + DSP 每段 1.4 s；评分模型每段 7.3 s；本机识别每次 1.75 s；对 exp021 不回混 19 项有配对、18 项差 0，只有 V-TIMBRE 不同（#119 改了量法）
- OpenRouter 真调用 3 次（同一条 LibriSpeech 干净人声 12 s）：逐字相同；每次 0.00009 美元；1.44 / 20.69 / 0.66 s；日志 `_tmp/exp025-asr-real-20261124-194229.log`，密钥不在日志里

## 待人 / 大总管
- 识别费 15 美元按**软上限**执行（人定，D103 补充）；OpenRouter 侧额度 100 美元
- 场景依据（弱项表还是提升空间表）基线出数后交人定
- 遗留 P2：#180（HTTP 200 内嵌可重试错误不退避）；#161（偶发 e2e 失败，上一条线）

## 流水（这条线）
- 11-25 写跑法、`--no-safety`、`gap.py` / `merge.py` / `subset.py`；5 段冒烟（名额内）
- 11-25 D103：查 OpenRouter 公开模型列表，D103 写的型号不存在，大总管改用 1.7B；写 OpenRouter 后端与测试
- 11-25 外发许可核对（只读公开页）：480 段里只有 20 段无疑问；MIT 原页没写许可；人定 D104
- 11-25 真调用可复现测试通过；并发、账本、停发按 codex 一轮轮补（第 5–16 轮，处理记录在本机 `_tmp/reviews/exp025-response-*.md`）

## 账本护栏（feat/asr-ledger-guard，2026-11-24）
- 已做（先红后绿）：缺账本拒发 · 显式 `--init-ledger` / `VDN_ASR_INIT_LEDGER=1` 才新建（created 四键、不记主机名）· `.sealed` 拒发 · `VDN_ASR_LEDGER` / `--asr-ledger` 可配路径
- 顺带：账本与路由文件的错误只写文件名，不带目录（会进成绩单 meta.asr，仓库已公开）
- 验证：vdn-eval 单元 226 条、clippy、tidy 过；codex 2 轮，第 2 轮无 P0/P1
- 整套集成测试由大总管跑，算验收；未推送
- 待命：云上基线结果回来后核对、出两张差距表（结论留空）、合并并封存本机账本；ort 在云上链接不过时可能要改成运行时加载

## 识别补跑前的备选办法（2026-11-25，建议待大总管定）
依据（实测，480 段缓存）：两次都有的 567 对里，修好判法后仍有 102 对字词不同（18%）。每段要三条音频都稳定才有吃字值，按独立估算约 45% 的段会判「不知道」（推断）。修复前覆盖到的 189 段里实测 107 段（57%）是「不知道」。
费用口径：整套 480 段 × 3 条 × 2 次 = 2880 次，每次约 12 秒 × 0.000008 美元 ≈ 0.0001 美元，合计约 0.28 美元（账本口径）。

| 办法 | 加多少费用 | 和 D28「不用 whisper」 | 还剩多少段「不知道」 |
|---|---|---|---|
| A 有分歧才调第 3 次，按分词后取多数 | 约 +18% 次数，约 +0.05 美元 | 不冲突（同一个模型） | 三次互不相同的才剩；比例未量化，补跑时实测 |
| B 两次都用：吃字取两次的平均，分歧大小另记一个读数 | 0 | 不冲突 | 因不稳定而判「不知道」的降到 0；要改规则 V7 的「剔除」，属于规则改动 |
| C 查接口能不能钉温度或随机种子 | 一次试调约 0.0003 美元 | 不冲突 | 能钉住就接近 0；文档没写这个参数，要实测 |
| D 换一个识别模型 | 看型号 | 换 whisper 就冲突；别的型号要另调研 | 未知 |

我的建议：先做 C（几乎不花钱）。钉不住就用 A，A 不改判据。B 要改规则，要人定。

## 探针去重 + 识别取多数（fix/asr-probe-dedup，2026-11-25）

- 起因：钉温度探针（cloudrun 跑）温度 0 钉不住，15 条里 1 条三次不一致（6.7%，线 2%）；探针输入 20 个文件去重后 15 条，旧代码不去重
- 改了什么：`asr-probe` 按 sha256 去重、报告 `duplicates`；识别前两次分词后不同就补第 3 次取多数（A），正本 `src/eval/src/qa/asr_vote.rs`（`TIE_BREAK_REP` = 3、`MAJORITY` = 2），接进 `Asr::transcribe` 与 `Asr::prefetch`（拆到 `asr_prefetch.rs`）；`meta.asr` 新增 `tie_breaks` · `tie_break_missing` · `no_majority`
- pi：`evalcmp-votecount`（isoquant/deepseek-v4.1-flash-1017-fast），提交 `98bdfa6b..acc34d33`（3 个，Agent: pi），花费 0.0153 美元；我读 diff、在 98bdfa6b 上实测新测试 5 条红、合入后全量重跑通过，再快进合入
- codex 评审（gpt-6-astra，只读，medium）：2 轮，P0×0 P1×0；第 1 轮 P2×2 全改，第 2 轮 P2×1 → #196；轮次原文在本 worktree `_tmp/reviews/probe-vote-codex-{1,2}.md`
- 验证：`cargo fmt --check` · workspace clippy `-D warnings` · `xtask tidy` · `cargo test --workspace`（99 组全过）

## 场景差距表第二版（exp/025-gap-v2，2026-11-26）

- 输入：cloudrun-39 识别补跑（云上 4 片，main ab0f4f0a，+1953 次 · +0.187488 美元），结果进仓 `docs/experiments/exp025-gtcrn-train/baseline/runs/rerun-20261126-013313/`
- 核对（实测）：混音 sha 480/480 同 run480；非吃字判定 0 变；每场景不过数增量 = 该场景吃字不过卡数；`gap.py` 重跑第一版输入逐字节相同；`--from-clips` 重汇总与补跑成绩单一致
- 表：`docs/experiments/exp025-gtcrn-train/baseline/gap/v2-20261126/README.md`。前 4 rail-air-cabin · canteen · meeting-room · transit-hub，与临时选定重合 1 →「不一致」；28 段没值全当过 / 全当不过，前 4 与判定都不变
- 没派 pi：这条线没有产品代码改动，只有数据与文档；敏感性改分脚本留在主目录 `_tmp/20261126-gap-v2/sens/sens.py`（不进仓）
- 收口（2026-11-26）：PR #208 已合并；人按第二版表定加训场景 = 弱项表前 4（D113）。之后待命：正式训练出模型后出新成绩单与配对比较

## 练完怎么评（开训前清单第 7 项，exp/025-eval-plan，2026-11-26）

- 计划：`docs/experiments/exp025-gtcrn-train/eval-plan.md`（评测集 6323 · 原版读数 rerun-20261126-013313 · 甲路离线 / 乙路流式 · 配对比较命令 · 014 第三节逐条落到评分项 · 「人耳未测」「只在合成集上判」· 估时与识别费）
- 新工具（pi 写，我审、合、重跑）：
  - `docs/experiments/exp025-gtcrn-train/eval/run_gtcrn_offline.py` · `parity.py` · `test_offline.py`（28 条）：pi `evalcmp-offline`，0.10 美元
  - `vdn-eval accept`（`src/eval/src/scorecard/accept.rs` + `accept/{inputs,judge}.rs`）与阈值正本 `SC-ACCEPT`（`thresholds/accept.rs`，014 第三节标键并有比对测试 `accept_doc.rs`）：pi `evalcmp-accept`，约 0.55 美元
- 真实对拍（实测）：官方权重内置流式 vs 离线甲路，冒烟 5 段，22 项 76 对最大差 3.8e-5（容差 1e-3），单侧缺值 0，9 项未验证（不带 --qa）。输出 `eval/runs/parity-20261126/`
- codex 评审（gpt-6-astra，只读，medium，按代码文件做 doc 型；diff 型被 hashes.txt 的假手机号拦）：13 轮，P1 依次 5·3·3·3·2·0·1·0·1·1·1·0·0；第 12、13 轮无 P0/P1 即停，剩余 P2 0。P1 全是「缺数时会不会误判过」，第 4、5、10 轮按「同类第三次改结构」改成：--suite 为准的应有清单、卡状态从绑定的 clips.jsonl 重新汇总、每份比较逐片段核覆盖、`item_complete` / `values_complete` 两个统一函数。处理记录在本 worktree `_tmp/reviews/evalplan-code-response-*.md`
- 记录在案：
  - #216：exp016 `_ext.py` 写死旧 build 17f5（这次不修）
  - 历史里有一个未推送提交（47e8c89a）写了本机绝对路径：已在新提交里换掉，大总管改写未推送历史后才推
  - 离线脚本 `rel()` 原先会写本机绝对路径，已修并有测试
  - 本机负载：一次在 1 分钟负载 20.5 时批了 pi 跑整包测试（超线），之后改成负载 < 20 且无 heavy.lock 才自动批
  - trainer 的 `losses.window()` 用 float32，和产品窗最多差 7.1e-7（不影响训练与评测，由 trainer 定改不改）
