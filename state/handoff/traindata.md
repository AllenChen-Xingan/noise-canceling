# 现状页 · traindata（issue #156，批次 20261116-b19）

状态：已收尾（2026-11-25：多视角自审 + codex 4 轮，第 3、4 轮无 P0/P1；待大总管重跑测试、开 PR）

## 手上
实体：训练数据集 · 数据切分 · 混音配方。产出都在 `docs/experiments/exp025-gtcrn-train/data/`，说明见那里的 `README.md`。

## 做到哪
混音已改成 GPU 批量做：CPU 只出 plan（读片段 + 抽随机数），`gpu_mix.render_batch` 在 GPU 上混。CPU 版留作对拍。
同一批 plan 两边逐样本差 ≤ 1e-4，写成了测试。本机 75 条测试全过（torch CPU 后端）。下载脚本已备好，没在云上跑。

## 下一步
1. 大总管：重跑测试、开 PR（Closes #156）；开 3 件 issue（见 `docs/experiments/exp025-gtcrn-train/data/reviews/156_traindata-codex.md` 页首）；评审索引 `docs/reviews/README.md` 加一行
2. 现行训练 build 是 `f032a75eedad80d6`（2026-11-25 云上用 main 86698924 重新登记），trainer 可以用

## 依赖
- 早期混响 100 ms 的正本在 `docs/experiments/exp013-scene-suite/rooms.py` 的 `EARLY_REVERB_MS` 与 `early_rir`。它们在 evalcmp 的分支 `feat/eval-paired-compare` 上，还没合进 main。合并前本分支的混音测试会红；本机验证用的是那个分支的 rooms.py
- 那个分支合并后，`test_rebuild.test_code_unchanged_since_build` 可能会红：exp013 的 rooms.py 变了，要在云上确认旧的 4,540 个文件仍逐位一致

- 网络盘配额正本 `docs/experiments/exp025-gtcrn-train/cloud/config.py` 在 cloudrun 的分支 `feat/cloud-runpod` 上。合进来之前，`test_real_cloud_config_quota` 会红，`fetch.py` 在 /workspace 上也会拒绝开下

## 待人拍板
（暂无）

## 发现
- QUT-NOISE 的服务器没声明支持断点续传（HEAD 响应里没有 `Accept-Ranges`）。脚本遇到这种情况会从头重下
- 官方 SEtrain 混音代码 `alpha = 10**(-snr*1.5/20)`：它的实际信噪比是名义的 1.5 倍（−7.5～22.5 dB）。本配方按真实信噪比取，已写进 `recipe.json` 的出处

## 流水
- 2026-11-16 报到；读 014 · 027 · 034 · exp013 的 leak.py / build.py / pools.py / rirs.py
- 2026-11-16 用 HEAD 请求核下载地址与 md5：OpenSLR 12 / 28 · QUT-NOISE 6 个 zip · TUM Zenodo · DEMAND Zenodo · Commons 3 个文件
- 2026-11-16 和 evalcmp 定早期混响正本（rooms.py）；和 trainer 定加载接口（区间由我给，掩码由 trainer 做；第 k 批只由 seed 与 k 决定）
- 2026-11-16 记下 exp013 原料 1,994 个文件的 sha256（`exp013_inputs.json`），原料索引和正式 build 一致
- 2026-11-16 写 sources.json · fetch.py · split_rules.json · split.py · leakcheck.py · recipe.json · mix.py · shard.py · loader.py · rebuild_exp013.py，各带测试
- 2026-11-24 恢复。按 D100 / D102：混音拆成 plan + render；新写 gpu_mix.py；加对拍测试（GPU 版对 CPU 版 ≤ 1e-4，电平与区间对 exp013 dsp.py 逐个相同）
- 2026-11-24 loader 改成 worker 只出 plan_batch，主进程调 `render_on_gpu`；DataLoader 用 2 个 worker 加 start_batch 测过
- 2026-11-24 fetch.py 加了两样：开下前先查盘（含解压峰值），以及 `--head`（只核大小，不下载）。本机发 HEAD 核了 A3，大小一致
- 2026-11-24 写 requirements.txt（版本钉死，和 exp013 旧 build 一致）· cloud_steps.sh · data/README.md · exp025 README 的数据一节
- 2026-11-24 大总管（noise-canceling-33）转来 cloudrun 的发现：网络盘上 df 报整个集群的剩余（158 TB）。查盘改成「配额 − du 已用」，配额从 cloud/config.py 按路径读，加了 5 条测试。用 cloudrun 分支上的真 config 临时核过：配额 150 GB，第一批含解压峰值约 37.2 GB，放得下
- 2026-11-24 大总管转 cloudrun：E3（Wikimedia）回 429 没下成。fetch.py 改了三处：429 / 503 按 Retry-After 等（没有就 5·2^n 秒，上限 600 s）；404 等立刻报错；对 wikimedia.org 每次请求前停 5 s。下载本来就是串行的，User-Agent 仍是通用名。加 6 条测试；cloud_steps.sh 加 `fetch-e3` 一步单独补下
- 2026-11-24 记下 OpenSLR 28 号包的读数（cloudrun 云上实测，经大总管转）：md5 e6f48e25… 写进 sources.json，sha256 3b50cfde… 写进新建的 fetched.json。加 1 条测试，核 fetched.json 和 sources.json 两边的字节数与 md5 一致
- 2026-11-24 建本 worktree 的 `.venv`（已被 git 忽略）：torch 2.15.0 · numpy 2.5.4 · scipy 1.18.1 · soundfile 0.14.0
- 2026-11-24 合并 cloudrun 取回的第一批读数（21 个包，A1–A5 + DEMAND）进 fetched.json：字节数 · 文件名 · 上游已给的 md5 逐个和 sources.json 核过，无冲突；28 号包与仓里已记的一致。加 1 条测试：第一批每个包都有钉死的 sha256
- 2026-11-24 E3 由 cloudrun 下完，3 个文件 sha256 都和 sources.json 一致，fetched.json 不用加。云上 7 次 429，第 8 次（最后一次）才成，每次等 120 s（不是默认退避，按退避应是 5、10、20… s，所以是服务器给的）。fetch.py 改两处：429 / 503 另算次数，最多 30 次（30 × 120 s ≈ 1 h），不占断线重试的名额；日志打出 Retry-After 原值。顺手修了改写时引入的死循环（连接正常结束但没下全时不计次）。加 2 条测试
- 2026-11-24 大总管放行切分、分片、登记。为云上一次跑通做了三处：① split_rules.json 每个来源加 expect（上游公布的文件数 / 说话人数，读文档得来），对不上就停、不写表；② loader 改成用时才导入 mix，登记 build 不再依赖还没合进来的 rooms.early_rir；③ cloud_steps.sh 加 inventory（清单写进 inventory.txt）与 pack（打包带回）两步。新写 tests/test_products.py 核带回的产物，先在夹具产物上跑通。README 第六节写了确切命令、预期产物、出错怎么办与核对方法。共 100 条测试，93 过 6 跳过 1 红（缺 cloud/config.py）
- 2026-11-25 云上 split 失败（cloudrun 跑，日志在主仓 _tmp/20261125-cloudrun-shard/）：「RWCP_type4_rir_000l.wav 不在任何 rir_list 里」。按 HTTP Range 只读上游 zip 的目录与 rir_list（约 8.5 MB 元数据，不下音频）查清：
  · 文件在表里。RWCP type4 那 38 行的 room-id 本身就写成文件名（`--room-id RWCP_000l.wav`），旧代码取每行第一个 .wav 当路径，取成了 room-id。改成按「--键 值」成对读，剩下的才是路径
  · 顺带查出旧代码按文件名查房间：模拟 RIR 小中大三套文件名重复，600 间房并成了 200 间。改成按完整路径查
  · AIR 同一间房按位置、角度、假人头、手机拆成 35 个 room-id；RWCP 的 room-id 是阵列与设置代码。上游 README 没给房间对照，两库各自整库归一个单位。REVERB 2014 的 9 个 room-id 是房间名，照用。实测 11 个单位与模拟 600 间房分层切
  · 4 份 rir_list 的 sha256 钉进 split_rules.json，上游变了就停
  · TUM：清单显示合成的文件夹叫 artifitial_wind_noise（上游拼写），旧的分层正则没认出来，已补；手机录音按文件名前缀并成 32 次录音（20 次录音跨 normal / strong 两个文件夹）
  · 提速：先认单位再算哈希；逐文件读数缓存在网络盘上（filecache.py），重跑只算变了的；小文件只开一次
  · 加测试：test_openslr28_rooms（8 条，含云上失败那个文件名，先红后绿）· test_tum_units（4 条，先红后绿）· test_filecache（5 条）。共 117 条，110 过 6 跳过 1 红（缺 cloud/config.py）
- 2026-11-25 云上重跑六步（cloudrun，提交 8b4194d3）全部退出码 0：split 1346 s · shard 1212 s。包 sha256 cf5c6cc9… 核对一致；inventory.txt · splits/ · builds/49349c158a4defdc/ 进仓。tests.test_products 7 条全过；本机按现行规则逐行重算单位 0 处不符；AIR 归终测原料池、RWCP 归训练，与 README 一致。全量 117 条 116 过，1 条红（缺 cloud/config.py）
- 2026-11-25 openslr28-rir.jsonl 16.4 MB 被 hook 的 10 MB 线挡在库外。先写 7 条会红的测试（tests/test_split_gzip.py），再让 split 把超 8 MB 的表存成 .jsonl.gz（哈希仍按解压后正文）。仓里改存 2.6 MB 的 .gz；全量 124 条 123 过，1 条红同上
- 2026-11-25 合入 origin/main（evalcmp 已合，83280aa0）。本机核 6323 对 17f5：旧 4540 个逐个相同，exclude 与版本相同，多 480 条早期混响轨。rebuild_exp013.py 加 --check-dir（先写会红的测试）。全量 127 条 125 过，2 条红：缺 cloud/config.py；等云上核对报告
- 2026-11-25 大总管定：跨平台差约 4e-13，不再要求云上逐位同 17f5。判据改为本机 6323 同 17f5 + 云上 6323 音频按 hashes.txt 全对 + aa9f 的 exclude 与版本同 + 原料全对；官方源可复现记「未验证」。--check-dir 换成 --check-cloud（先改测试再改码）。用 aa9f 真小文件干跑通过（两份日志是仿造的）
- 2026-11-25 cloudrun 带回两份日志：6323 音频 5020/5020 按 hashes.txt 对上；原料全对。合入 origin/main（971c7eec，aa9f 小文件已在 main）。--check-cloud 四项全过，报告 rebuild/exp013-20261124-192451.json 和两份日志进仓；官方源可复现记「未验证」。全量 128 条 127 过，1 条红（缺 cloud/config.py）
- 2026-11-25 收尾评审：多视角自审（第一次误审了 src/，改参数重跑）+ codex gpt-6-astra 4 轮，P1×5 · P2×15 全核实属实，第 3、4 轮无 P0/P1 停。exp013 判据升到第 4 版，现行报告 rebuild/exp013-20261124-200416.json。合入 cloudrun 的 main（df145344）后 test_real_cloud_config_quota 转绿，全量 139 条全过
- 2026-11-25 cloudrun 用 main 86698924 重跑 register + pack：新 build f032a75eedad80d6。包 sha256 53e59cdc… 核对一致；包里其余文件和仓里逐字节相同；按记录字段重算 id 相同，代码与配方哈希等于现行，交集检查 6 个来源 83,460 行 0 处。进仓于分支 chore/exp025-register2
- 2026-11-25 大总管交代：CPU 镜像 rust:1.99.1-trixie 没有 pip，setup 的 pip install uv 失败。新增 setup_uv.sh：有 uv 直接用；没有就下 uv 0.13.7 官方二进制包，按钉死的官方 sha256 核对，解压到 /workspace/bin。测试 tests/test_setup_uv.py 10 条（哈希不对拒绝、路径、联网核官方 sha256 要设 VDN_NET=1）。分支 fix/cloud-setup-uv；全量 149 条全过（1 条联网的跳过，另手动跑过也过）
- 2026-11-25 大总管派活（D106 · D107），分支 feat/data-batch2：① 第二批调研清单 download-list-2.md（CochlScene 3 类 · DEMAND OHALLWAY/DLIVING · FSD50K 按标签；飞机舱 · 会议室 · 电话亭写「没有」；大总管定按上传者排除 ESC-50 同源）。② loader target_scene.scene 支持列表（pick_scene，单名抽法不变），测试 test_target_scenes 9 条先红后绿。③ 配方 high_snr 旋钮（默认关；10 % SNR U(20,40) dB · 5 % 不加噪声，暂定），测试 test_high_snr 9 条先红后绿。loader · mix · recipe 改了，build 身份会变，下载完一起重跑 register。全量 167 条全过（跳过 1 条联网）
- 2026-11-25 人定（大总管转达）：第二批只下 CochlScene 3 类；DEMAND 走廊 / 客厅与 FSD50K 人未选、暂不下；盘可扩到 200 GB；高信噪比旋钮认可。等 cloudrun 下完交校验值给我核对

- 2026-11-25 大总管改名 noise-canceling-c4（uds:/tmp/cc-socks/79882.sock），报到、进度、名额都发它。新规则：代码派 pi 写。已报：压缩包建议核完 md5 再删（待人定）；下一步派 pi 登记 CochlScene 3 类。
- 2026-11-25 CochlScene：6 个分卷包已被 cloudrun 的下载脚本自动删掉（大总管 c4 删机前看过），所以「删不删压缩包」这题作废，md5 也没补核。完整性证据是间接的：7z 逐文件 CRC 通过，17629/17629 个文件和 want.txt 一致。
- 2026-11-25 CochlScene：人声没筛，方法待定（大总管 c4 定：这次先登记、不筛）。目标场景（D105）待人定，这次只加噪声池、不改 target_scene。
- 2026-11-25 pi traindata-cochlscene（isoquant/deepseek-v4.1-flash-1017-fast）开工时 HEAD = e56fee16；之后的自动提交（从 c5394187 起）含 pi 未验收的代码，验收通过的那次提交另记。
- 2026-11-25 pi traindata-cochlscene 验收通过。代码提交范围 c5394187…8420eef2（自动提交，署名 Claude，实为 pi 写），花费 $0.072 / $2。我亲手重跑 200 条测试，全过（跳过 1 条网络测试）。共 8 项：A6 来源（标手工、fetch 拒下）· cochlscene 按录音人 9:1 切（认不出录音人就停）· 4 个噪声池 · shard 来源从 recipe 导出 · 噪声池来源缺失即停、build.json 记 noise_scenes · test_cochlscene.py。上云命令见 data/cloud-batch2.md。
- 2026-11-25 D109：以后派 pi 用 --repo/--base，pi 在自己的分支上每一小步提交一次，我验收后快进合入。
- 2026-11-25 已知偏差：这批数据里拥挤室内权重偏高。cowork 与 open-office 共用同一批录音，target_scene 为 null 时被抽两倍（推断）。c4 定修法：按来源去重抽样，派 pi 写，带一条会红的测试。现在 pi-worker 正在改，暂不能起 pi，所以这批先在 recipe.json 的 noise_pools._说明 里记明（配方哈希进 build 身份）。
- 2026-11-25 pi traindata-dedup 验收通过，快进合入。pi 提交 0137540a…2eaf21b9（各带 Agent: pi），花费 $0.0125 / $2。我亲手重跑 209 条测试，全过（跳过 1 条网络测试）。成员完全相同的噪声池（cowork 与 open-office）抽样只算一次权重，报出 cowork；build.json 记 scene_groups。没有重复组时随机数与以前逐样本相同（旧测试不改照过）。只影响 register 和训练时的抽样，split 与 shard 不变。
- 2026-11-25 D111 补充：飞机舱先不补，人不给 Freesound 密钥；飞机舱暂由地铁车厢录音代替，已记进 recipe.json 的说明。Freesound 抓取已停，只抓到日志，放在主仓 _tmp/20261116-b19/freesound-cabin-partial/，不进仓。BUT 下载清单在 data/download-list-but.md：只下 2019 版 RIR-Only 包（9,308,593,693 字节）。上游没有校验码，首次下载时记 md5 与 sha256；按 Apache 2.0 处理，read_me 副本在 data/third_party/。
- 2026-11-25 pi traindata-screen 验收通过：rebase 到 fbee43b4 后快进合入，pi 提交 8af06c05…f98ea491（各带 Agent: pi），花费 $0.0178。我亲手重跑 231 条测试，全过（跳过 1 条网络测试）。screen.py 筛人声（Silero VAD、PANNs 的 Speech 标签）、像不像目标场景（PANNs 标签、混响时间）和平稳度；阈值只在 screen_rules.json，全部暂定。两个模型云上才装（silero-vad、panns-inference），版本没钉，下次装时钉。命令行只筛噪声片段，不传冲激响应，所以会议室和电话亭的混响时间那一项记「未查」。
- 2026-11-26 第二批产物进仓：splits/cochlscene.*、builds/6173bf9ca6a6e143/（云上 fac82500）。我核过：包 sha256 bfdc87dd… 对得上；包内其余文件和仓里逐字节相同；3 类条数 5835 / 5897 / 5897；没有录音人跨份；build id 能重算对上。删了「未建表」标记。pi traindata-products（5be0fda4，$0.0083）把两处手抄的来源清单改成从正本导出；我重跑 231 条测试，全过。第一轮 build 889a389708c8a1e9 作废。
- 2026-11-26 开 issue #202、PR #203（feat/data-batch2 → main）。强推被拦，所以没 rebase，改为合并 origin/main（e4d014c9）。合并后重跑 231 条测试，全过。本机留了一个分支 rebased-backup-20261126：那次 rebase 的结果，没推送。删分支要大总管做（钩子拦了）。
- 2026-11-26 BUT 混响库已下完（cloudrun）：BUT_ReverbDB_rel_19_06_RIR-Only.tgz，9,308,593,693 字节（和 HEAD 读到的相同），md5 3ad91777f6cadec81fb4c07dfbef6f19，sha256 d0d14cdd626b4b05401027c5a774b4f24369f4ea38f57480c6fc6e04e9f3f5b5（首次下载记下，mode=bytes-only）。gzip -t 过，包内没有 LibriSpeech。9 个房间，RIR 2,325 个、silence 9,982 个。解压只取 RIR、silence 的 wav 和全部 .txt，不解 .py、.jpg。下一批要把这几个值钉进 sources.json。
- 2026-11-26 BUT 已解到 /workspace/train-raw/but-reverbdb/（cloudrun，712 秒）：wav 12,307 个 = RIR 2,325 + silence 9,982，txt 1,748 个，都和清单对得上；解出 23 GB，盘 153 / 200 GB。解出比包大是正常的：silence 9,982 条 × 60 秒 × 16 kHz × 2 字节 ≈ 19.2 GB（推算）。env_meta 实读：R112 酒店房间 4.43 × 2.84 × 2.64 米加 2.20 × 1.17 米的小间，40 立方米，底噪 35 dBA；C236 会议室 6.96 × 4.12 × 3.56 米，102 立方米，底噪 38 dBA。压缩包还在 archives/，删不删待大总管定。
- 2026-11-26 第三批开工：分支 feat/data-batch3（大总管建，从 37c75609），issue #205。派 pi traindata-but，任务文件在主仓 _tmp/20261116-b19/pi-traindata-but.md。BUT 底噪先只收 C236（会议室）和 Room112（酒店房间）；单位是「房间 + 扬声器布置」，每个房间分开按比例切，开发份至少一个布置。⚠️ 已知局限（大总管 c4 定的写法）：会议室、酒店房间各只有一间房，开发份和训练份是同一间房，开发份对这两类只测同房不同场次，不测换房间。放收音机的布置（spk_meta 里有第 2 个扬声器）切分时排除；其余底噪先过 screen.py。
- 2026-11-26 pi traindata-but 验收通过，快进合入：pi 提交 2f52f14c…949a0bdf，共 22 个，各带 Agent: pi；花费 $0.086；已 clean。我读了 diff，亲手重跑 276 条测试，全过（跳过 1 条网络测试）。期间 pi 两次用 python3 内联脚本改测试文本，被误判成重活申请名额，我直接放了（大总管同意：约 1 分钟内、不计时、不构建的不用名额）。上云命令写在 data/cloud-batch3.md。
- 2026-11-26 pi traindata-shards-env 验收通过，快进合入：pi 提交 2e709452…364720a9（4 个，各带 Agent: pi），花费 $0.0068，已 clean。cloud_steps.sh 去掉 first-batch 默认值，shard、register、pack 三步不设 VDN_SHARDS 就退出码 2，其余步骤照旧。我重跑 282 条测试，全过（跳过 1 条网络测试）。另开 issue #206（增量分片，下一件）；README 写明旧分片只能用当时的提交复现。旧分片删不删要问人，人回之前不删。
- 2026-11-26 人定 D112：first-batch 在第三批 valid、取回核对后由 cloudrun 删，删前先取回 build 小文件；batch2 留到第一个正式运行过了开发份检查。README 已改成引用 D112。
- 2026-11-26 云上发现 screen.py 的 bug（cloudrun）：panns-inference 0.1.1 里 labels 本身就是 list，代码却读 .labels。pi traindata-panns-labels 修好：先写会红的测试 261f0b59，再修 d5b8b696。假模型照真 PANNs 的返回形状写；加了契约测试 RealModels，本机跳过，云上设 VDN_MODELS=1 人工跑。花费 $0.0056，已 clean。我重跑 288 条测试，全过（跳过 3 条）。模型版本钉进 data/requirements-screen.txt：silero-vad 6.3、panns-inference 0.1.1、librosa 1.0.1、torchlibrosa 0.1.0。收音机排除没生效的事正在核（等 cloudrun grep 真 spk_meta）。
- 2026-11-26 收音机核清楚（cloudrun grep 实测）：全库 54 份 spk_meta 里只有 L207 的 2 份带 EnvSpk2，C236 和 R112 一份都没有，868 条是对的，不是 bug。两份真 spk_meta 进了 tests/fixtures/but_spk_meta/（9291ec39）。pi traindata-spkmeta（627547e3…403e9dbc，$0.0068，已 clean）把夹具改成用真文件按原样字节写，加了 RealSpkMeta 5 条测试。我重跑 293 条测试，全过（跳过 3 条）。
- 2026-11-26 第三批筛查结果（cloudrun 跑，7badd9a1；我读了 jsonl，实测）：R112 keep 209 / 248，正常。C236 只 keep 32 / 620，每条 10 秒（我原先按 60 秒估错了）。C236 被丢的主要原因：不平稳 545 条（逐秒电平标准差中位 7.5 dB，阈值 6），说话帧占比略超 355 条（中位 0.013，阈值 0.01）。前 5 个标签里 609 条有 Music、382 条有 Tuning fork；Speech 标签概率很低。推断底噪里有个一直响、电平起伏的单音，VAD 被它骗了，但没人听过。按规矩停在 split 前，机器删了，筛查结果留在网络盘 /workspace/out/batch3/。给大总管三条路（A 只登记酒店房间【建议】· B 会议室只用 32 条 keep · C 先查清 C236），等定。阈值不为凑数放宽。
- 2026-11-26 人定 D113：加训 4 个目标场景 = rail-air-cabin · canteen · meeting-room · transit-hub。会议室池等人听完 C236 再定，所以这一批 target_scene 先写 3 个（大总管定）。这批 build 只用来提前验证数据管线，不正式训练（D110）。限制：rail-air-cabin 只有地铁车厢录音；canteen 只有 DEMAND PCAFETER 一个来源。pi traindata-b3b（rebase 后快进合入，4 个提交，$0.0276，已 clean）：BUT 规则只留 R112（248 条），去掉 meeting-room 池，target_scene 写 3 个；新增 diagnose.py（逐秒电平 · 频谱峰，平稳度复用 screen.level_std_db）。筛查结果 jsonl 868 行进仓（89ab84e8）。我重跑 316 条测试，全过（跳过 3 条）。下次开机命令：data/cloud-batch3b.md。
- 2026-11-26 第三批续跑产物进仓（云上 b81a9645，cloudrun）：splits/but-reverbdb-silence.*、builds/10aa3a9fb8cef48f/。我核过：包 sha256 e8c5c3d0… 一致；包里只多 3 个新文件；209 行全是 R112、在筛查表里都是 keep；5 个布置不跨份（dev 是 SpkID05）；build id 能重算对上，配方和代码哈希等于 b81a9645；7 个来源；12 个噪声场景，没有 meeting-room；missing 为空。删了「未建表」（e2b…，见 git log）。test_products 两条测试没跟上排除与分层，已派 pi traindata-products2。已告诉 cloudrun 按 D112 可以删 first-batch；它说要等下一台批下来的机器，开机先删，删前问 trainer。⚠️ 盘只剩约 32 GB，离 30 GB 门槛只差 2 GB。
- 2026-11-26 **C236 诊断结果（给大总管，c4 会话已找不到，没送达）**：cloudrun 跑了 diagnose.py，10 条，实测。① 每条第 1 秒比后面高 20–30 dB（第 1 秒约 −28 到 −40 dBFS，后 9 秒稳定在 −58 到 −62 dBFS），这就是「不平稳」的来源，去掉第 1 秒后很平稳。② 10 条的最强频谱峰都在 250 Hz，比频谱中位高 53–65 dB，是很强的单音；PANNs 标成 Tuning fork、Sine wave，VAD 的 1% 多半也是它触发的。推断（没听过）：每段开头有一声 250 Hz 提示音。但频谱按整段平均，分不出 250 Hz 是只在第 1 秒还是一直都有。建议，待人听完 _tmp/20261126-c236-listen/ 的 3 段再定：如果只是开头一声哔，就在规则里加「去掉开头 1 秒」（数值放 json），重筛 C236，阈值不动；如果整段都有，会议室池就不用 C236。请人留意两件事：开头那一下是什么；后面 9 秒有没有一直在响的嗡声。
- 更正上一条：删「未建表」那次提交是 913e911b（上条写成了「e2b…」）。
- 2026-11-26 pi traindata-products2（8cb490f0…6588132a，rebase 后快进合入，$0.0099，已 clean）：test_products 跟上排除计数，以及按房间分层加每房间开发份保底；加了「筛查表里都是 keep」的断言。pi 第一版改了 split.py（只是抽函数），我打回了：split.py 算进 build id 的代码哈希，改一个字节，刚登记的 build 10aa3a9f 在新代码下就读不了。现在 split.py 的 sha256 = 56380967…，与 build 记录一致。我重跑 316 条测试，全过（跳过 3 条）。教训：loader.CODE 里的文件（mix · gpu_mix · loader · shard · leakcheck · split）在两次登记之间不要动。
- 2026-11-26 接替会话开工（旧会话 traindata-f0 上下文太大；tmux 约 04:44Z 整个消失过）。大总管改为 noise-canceling-87。这次的活：D114 的 C236 判定 + 开训前清单（014 第七节）第 2 项「4 个场景逐场景表」与第 3 项「新 build 登记 · valid · 与 exclude.json 交集为空」，做完开 PR（Closes #205）。派了两个 pi：traindata-tone（diagnose.py 加逐秒 250 Hz 单音测量，数值放 screen_rules.json 的 tone_check）· traindata-scenes（新增 scene_table.py，从 build.json + 切分表 + recipe 生成逐场景表，不手抄）。两个都不碰 loader.CODE 的 6 个文件。会议室备选来源（若 C236 不能用）：BUT 3 间办公室 L207 · L212 · Q301 的底噪（download-list-but.md 第 45 行原计划就含它们，已在云盘，Apache 2.0；L207 有 2 份放收音机的布置要排除），还没筛查。
- 2026-11-26 D114 本机判定（实测，3 段，diagnose.py --tone，判据 screen_rules.json tone_check：250±5 Hz 比 150–350 Hz 中位高 ≥10 dB 算「有」）：片段 1 只在第 1 秒；片段 2 第 1–2 秒；片段 3 第 1–3 秒（第 2 秒 17.7 dB、第 3 秒 14.9 dB）。按 D114 → 会议室池不用 C236。⚠️ 阈值改 20 dB 则 3 段都只在第 1 秒；10 dB 是开测前定的，不为凑数改。结果 screen/c236-tone-local3.jsonl。备选：BUT 3 间办公室底噪。云上步骤 data/cloud-batch4.md（A 段只读：全部 620 段 C236 与办公室逐秒单音 + 办公室筛查；B 段改规则后登记）。pi traindata-tone（a4ebc9e1…895112fb，$0.0094，已 clean）· pi traindata-scenes（a2552000，rebase 后快进，$0.0127，已 clean）。我重跑 334 条测试，全过（跳过 3 条）。scene_table 在 build 10aa3a9f 上出了 3 个场景的表。已答 trainer：target_scene.scene 的格式（字符串列表，exp013 场景名），formal.py 调 loader.load_recipe + scene_table.target_scenes。
- 2026-11-26 大总管 87 批了第四批 A 段（只读，机器费 ≤0.2 美元），确认 10 dB 判 C236 不用、阈值不改。写了 data/scene-gaps.md：会议室=办公室底噪的限制；食堂 0.67 小时（16 声道同一次录音，约 150 秒不同内容），正式运行每秒噪声约用 44 次（推算）；补救 A–E，建议 C（食堂池加 PRESTO，只改 recipe.json），D（CochlScene Restaurant）要人定。已请 cloudrun 在 A 段后只读查 CochlScene 的 Restaurant 与 Cafe 在不在盘上。等：A 段结果 · 大总管对 C/D 的回复（不回默认 C）。
- 2026-11-26 消融（#212，大总管 87 定）：和 trainer-0e 商定格式：data/recipes/ablation/{no-eq,no-clip,no-n_noises,no-reverb,high_snr-on}.json，由 ablation_recipes.py 从 data/recipe.json 生成，顶层 _ablation{base_sha256,knob,on}，只改 knobs.<旋钮>.on；基线原地在 data/recipe.json。pi traindata-ablation（bcbcb1da，$0.0047，已 clean）。我重跑测试全过（跳过 3 条），--check 退 0。⚠️ B 段改 recipe.json 后要重跑 ablation_recipes.py，否则测试红。B 段要在同一批分片上登记 6 个 build（基线 + 5 变体）。D（CochlScene Restaurant）盘上没有、要重下约 54 GB，已写进 scene-gaps.md，待人定。
- 2026-11-26 cloudrun A 段中途：C236 全量 620 条 first_only 331 · later_too 289，确认不用。办公室 4154 条（L207 1798 · L212 1488 · Q301 868，都 60 秒；L207 收音机 806 条已排除）。大总管 87 先定全量筛、后改回「取 3 个麦克风位置、约 400 条、现机跑」。pi traindata-tonemerge（24187d47，$0.0076，已 clean）：screen_tone.py 把单音结论并进筛查表，drop_where 放 screen_rules.json。pi traindata-unique（3497f326 rebase 后快进，已 clean）：场景表加「不同内容（小时）」，规则在 scene_notes.json same_content；食堂训练份不同内容只有 0.04 小时（150 秒）。我重跑测试全过。
- 2026-11-26 第四批 A 段结果进仓（cloudrun，包 sha256 7df39381…，sm5qjwcfp8slir 已删）：办公室 01·11·21 号麦克风 402 条筛查 keep 390，并入单音后 keep 375 · drop 27（因单音新丢 15）；收音机 78 条。证据 screen/office-3mic-screen.jsonl · office-3mic-tone.jsonl · c236-tone.jsonl · office-layout-mic.txt；全量 office-tone.jsonl 24 MB 不进仓，sha256 eb13a41a…，在云盘 /workspace/out/batch4/ 与主仓 _tmp/20261126-cloudrun-batch4/。pi traindata-batch4（12ad412d…33990076，rebase 后快进，$0.0825，已 clean）：规则 728 条 · 4 个目标场景 · 会议室=3 间办公室 · 食堂 PCAFETER|PRESTO · 未建表标记 · 5 个变体重生成；recipe.json sha256 d07618e4…。我重跑 408 条测试全过（跳过 3 条），--check 通过。B 段步骤 data/cloud-batch4b.md（登记 6 个 build）。

- 2026-11-26 15:05Z 第四批 B 段第一次跑（a632a660）在切分被 split.py 拒绝：仓里 splits/but-reverbdb-silence.json 是 10aa3a9f 在用的旧表，不原地覆盖。大总管 87 定用新名字。pi traindata-b4name（$0.028）：新来源 `but-reverbdb-silence-b4`（4 间房 728 条），配方 hotel-room / meeting-room 指向它；旧规则逐字节还原（rule_sha 8d8b3750… = 旧表）；未建表只剩 -b4。本机全量 414 OK（skipped 3）。推 254ebece，发 cloudrun 重跑。
- 2026-11-26 大总管固定名改为 `noise-canceling-meta`（人定，轮换后不变），以后一律报这个名字；旧名 noise-canceling-87 不再用。
- 2026-11-26 ~16:10Z B 段结果（cloudrun，代码 254ebece）：包 traindata-products-20261126T155724Z.tar.gz sha256 a2b67102… 本机核一致（主仓 _tmp/20261126-cloudrun-batch4b/）。6 个 build 进仓（59f7b776）+ splits/but-reverbdb-silence-b4.{json,jsonl}。我亲手核：id 重算、valid、leak 0、demand_segments 0、missing []、切分表 / 代码哈希一致、分片同、变体只差一个旋钮。基线 bc0e9e7b566421f0；no-eq 574ef5c7 · no-clip 748aee58 · no-n_noises 36ea9561 · no-reverb 966a229d · high_snr-on 10ebb73b。batch3 已按 D116 删（cloudrun）。pi traindata-evidence 在做：删未建表 · 场景表测试改 bc0e9e7b · build_evidence.py 生成第 3 项证据。
