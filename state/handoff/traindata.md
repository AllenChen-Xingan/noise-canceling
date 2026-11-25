# 现状页 · traindata（issue #156，批次 20261116-b19）

状态：进行中（第一批产物已带回、核过、进仓；待 exp013 重建与收尾评审）

## 手上
实体：训练数据集 · 数据切分 · 混音配方。产出都在 `docs/experiments/exp025-gtcrn-train/data/`，说明见那里的 `README.md`。

## 做到哪
混音已改成 GPU 批量做：CPU 只出 plan（读片段 + 抽随机数），`gpu_mix.render_batch` 在 GPU 上混。CPU 版留作对拍。
同一批 plan 两边逐样本差 ≤ 1e-4，写成了测试。本机 75 条测试全过（torch CPU 后端）。下载脚本已备好，没在云上跑。

## 下一步
1. exp013 重建不单独跑：等云上 main 复刻 6323 的 build，按 README 6.5 用 `rebuild_exp013.py --check-dir` 核
2. cloudrun 的分支合进来后全量重跑测试
3. 收尾：多视角自审 + codex 评审

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
