# 034 · 训练算力：云机器选型 · 别人怎么训 · 云上做法

> 写的人：调研子代理（只读联网，2026-11-16）。不装、不下载、不注册。
> 用途：回答人问的三件事。① 云机器用哪家；② 同类小模型别人用什么卡、训多久；③ 云上训练的做法。
> 标记：【出处 [n]】见文末链接；【推断】是我推的，没有出处；「未找到」是查了没查到。
> 价格全是 2026-11-16 查的，美元，按小时，未含税。价格常变，开机前再看一眼。

## 〇、一句话结论

- **平台不改，仍用 RunPod（D96 第 1、2 条不变）。**
- **显卡要改：RunPod 的 4090 机器只配 6 个 vCPU（虚拟 CPU 核），和 D96 自己写的「vCPU ≥ 16」冲突。** 【出处 [1]】
- 建议先用 **RTX 3090**：Secure Cloud 每小时 $0.50，配 16 vCPU、125 GB 内存，比 4090 便宜 $0.39 / 小时。【出处 [1]】
- GTCRN 很小，显卡算力用不满；喂数据的 CPU 才是限速的地方。【推断，理由见第二节】
- 这是建议，待你定。如果你不回，默认用 3090；后果是每小时省 $0.39，但单步算得比 4090 慢一些（未量化）。

## 一、云机器用哪家

### 1.1 每家一览

下表用来横向比价。「持久存储」指关掉显卡机器后数据还在的那块盘。

| 平台 | 合适的卡与每小时价 | vCPU | 持久存储价 | 关机后数据 | 开关机 API / CLI |
|---|---|---|---|---|---|
| RunPod Secure | 4090 $0.89；3090 $0.50；A5000 $0.27；L4 $0.69 | 4090=6；3090=16；A5000=9；L4=12 | 网络存储盘 $0.07 / GB / 月（<1 TB） | 在 | 有：`runpodctl`、REST |
| RunPod Community | 4090 $0.34；3090 $0.22；A5000 $0.16 | 未在价目页列出 | 不能挂网络存储盘 | 删机即丢 | 同上 |
| Vast.ai | 4090 约 $0.31–0.52（市场价，第三方） | 看房东，可筛选 | 按房东定价，停机照收 | 停机在，删机丢 | 有：`vastai` CLI |
| Lambda | 无 4090；最便宜 A10 $1.29、A6000 $1.09 | A10=30；A6000=14 | 文件系统，文档示例 $0.20 / GB / 月 | 在 | 有：Cloud API |
| Modal | L4 约 $0.80；A10 约 $1.10（按秒） | 另付，约 $0.047 / 物理核·时 | Volume $0.09 / GiB / 月，每月 1 TiB 免费 | 在 | 有：Python SDK |
| HF Jobs | L4 $0.80（8 vCPU）；A10G-large $1.50（12 vCPU） | 见左 | 挂 HF Storage Bucket（价未查） | Bucket 在 | 有：`hf jobs` CLI |
| Colab | L4 约 1.71 计算单元 / 时；Pro $9.99 买 100 单元 | 未找到 | 无（靠挂 Google Drive） | 会话结束即丢 | 无正式开关机 API |
| AutoDL | 4090 约 ¥1.61–2.68 / 时（第三方） | 看机器 | 数据盘按日；文件存储 20 GB 内免费，超出 ¥0.01 / GB / 日 | 在，但连续关机 15 天释放 | 有弹性部署 API；开关机接口未找到 |

出处：
- RunPod 价、vCPU、存储【出处 [1][2][3]】。注意：RunPod 价目页写 4090 Secure $0.89，另一处页面写 $0.69，官方自己不一致。
- Vast.ai 价是第三方汇总，官方网格没加载出来【出处 [6][7]】；存储与停机规则【出处 [8][9]】。
- Lambda【出处 [10][11]】；Modal【出处 [12]】；HF Jobs【出处 [13]】。
- Colab 单元数来自博客实测，不是官方表【出处 [14]】。
- AutoDL 价来自第三方【出处 [15]】；计费与释放规则来自官方文档【出处 [16][17]】。

### 1.2 逐项看

**持久存储与关机后数据**
- RunPod 网络存储盘：机器删了数据还在；只在 Secure Cloud 能用；建机时就得挂上，事后不能加。【出处 [3]】
- RunPod 余额到 0：挂盘的机器会停，盘还在，但存储费照算；一直欠费，盘可能被删且找不回。【出处 [3]】
- RunPod 机器自带的容器盘：停机即清空。机器上的 volume 盘停机保留、删机即丢，停机时 $0.20 / GB / 月。【出处 [2][4]】
- Vast.ai：停机数据在，存储照收，直到删机。【出处 [8][9]】
- AutoDL：连续关机 15 天自动释放，数据清空不可恢复。【出处 [17]】

**下载速度（OpenSLR / Zenodo）**
- OpenSLR 有欧洲两个镜像、中国一个镜像；第 28 号资源（混响与噪声包，1.3 GB）没有美国镜像。【出处 [18]】
- Zenodo 有过多次变慢：2023 年升级后 1 GB 要约 1 小时；2026 年 1 月官方说被爬虫拖慢、限速也挡不住。【出处 [19][20]】
- 有人测：同一个 6 GB Zenodo 文件，用库函数下要 2 小时，直接 `wget` 只要 16 分钟。【出处 [21]】
- Zenodo 支持断点续传（HTTP Range）。【出处 [21]】
- 各云平台到 OpenSLR / Zenodo 的实测速度：未找到。开机后先用 `curl` 测一个文件再决定。
- AutoDL 在国内，访问欧洲镜像和 Zenodo 可能更慢；它有中国 OpenSLR 镜像可用。【推断】

**CPU 够不够现场混音**
- 一步要造 32 段 × 4 秒 = 128 秒音频：读语音、读噪声、卷一次房间混响、按信噪比相加。【推断】
- 6 个 vCPU 扣掉主进程，只剩约 5 个装数据的进程。GTCRN 一步的显卡时间很短，5 个进程很可能跟不上。【推断，未实测】
- 所以 vCPU 多的机器更合适：RunPod 3090 有 16 个，L4 有 12 个。【出处 [1]】
- 判定办法：先在本机只跑数据管线，量「每秒能造多少批」，再和显卡每秒能吃多少批比。量完再定卡。【推断】

**自动开关机**
- RunPod：机器里可跑 `runpodctl pod stop $RUNPOD_POD_ID`；彻底删机用 `runpodctl pod delete`；也有 REST 接口。【出处 [4][5]】
- 注意：`stop` 只停显卡，机器自带盘照收费；数据放网络存储盘时，训完直接 `delete` 最省。【出处 [2][4]】
- HF Jobs 默认 30 分钟超时，训练要显式设 `timeout`。【出处 [13]】

**抢占式实例（便宜但会被收回）**
- Vast.ai「可中断」实例便宜 50% 以上；被出价更高的人抢走时进程被杀，数据保留，等回来再续，可能等很久。【出处 [8][9]】
- RunPod Spot：官方价目页没列 Spot 价；社区回复说先发 `SIGTERM`，约 5 秒后 `SIGKILL`。这条只有论坛出处，官方文档未找到。【出处 [22]】
- 只有 5 秒，来不及临时存检查点；要靠「每 N 步定时存」。【推断】
- 建议第一批不用抢占式：一组只跑 5,000–20,000 步，省下的钱少，被打断的麻烦大。【推断】

**评测上云（480 段 + Qwen3-ASR + SCOREQ / Distill-MOS）**
- 这几个打分模型显存需求都远低于 24 GB，3090 / 4090 / L4 都装得下。【推断，未查各模型显存】
- 评测是一次性批量活，可以和训练用同一台机器、同一块网络存储盘。【推断】

### 1.3 推荐

1. **平台：RunPod Secure Cloud + 网络存储盘。** 理由：盘和显卡机器分开，换机不丢数据；有 CLI 可让 AI 开关机；价在中档。【出处 [1][3][4]】
2. **卡：先用 RTX 3090（16 vCPU，$0.50 / 时）。** 量完数据管线，若 CPU 有余、显卡吃不饱，再看 A5000（$0.27 / 时，9 vCPU）。【出处 [1]；选择是推断】
3. **建盘前先查：** 网络存储盘建在哪个机房，那个机房得有 3090 库存。盘只能在本机房用。【出处 [3]】
4. **备选：Vast.ai。** 更便宜，可以按 vCPU 数挑机器；但存储跟着单台机器走。只在 RunPod 没货时用。【出处 [8]】
5. **不建议：** Colab（会话会断、无持久盘）；Lambda（没有 4090 档，最便宜卡 $1.09 / 时）；Modal / HF Jobs（同档卡贵一倍左右，适合短任务）。【出处 [10][12][13][14]】
6. **AutoDL：** 价最低，但 15 天关机释放、国内访问 Zenodo 慢、要人民币实名账号。本项目先不用。【出处 [17]；后两条推断】

**D96 要不要改：** 平台与存储盘不改。第 3 条「RTX 4090 一张」建议改成「RTX 3090 一张，或先量 CPU 再定」，因为 RunPod 4090 只有 6 vCPU，和同一条里的「vCPU ≥ 16」矛盾。建议，待你定。

### 1.4 存储与花费估算

- 150 GB 网络存储盘 × $0.07 = 每月 $10.5。【出处 [2]】
- 以后数据到约 100 GB：若压缩包、解压后的原文件、16 kHz 分片三份同时留着，150 GB 不够。转成分片后删掉解压的原文件即可。【推断】
- 16 kHz 单声道 16 位：每小时音频约 115 MB（16,000 × 2 字节 × 3,600 秒）。【推断，按定义算】
- 一组训练花多少钱：每秒能跑几步还没量，未量化。公式是「步数 ÷ 每秒步数 ÷ 3,600 × 每小时价」。

## 二、同类小模型别人怎么训

下表把查到的训练设置放在一起。查不到的写「未找到」。

| 模型 | 显卡与张数 | 批量 | 训多久 | 数据与切段 | 出处 |
|---|---|---|---|---|---|
| GTCRN | 未找到 | 未找到 | 未找到 | DNS3 约 2,000 小时；每轮 4 万对 8 秒片段 | 【出处 [23]】 |
| ULCNet（Align 版） | 未找到 | 64 | 每轮 2 万步，总轮数未找到 | 约 1,100 小时，3 秒片段 | 【出处 [24]】 |
| DeepFilterNet v1 | 未找到 | 32 | 30 轮 | 3 秒片段 | 【出处 [25]】 |
| DeepFilterNet2 | 未找到 | 从 8 逐步加到 96 | 100 轮 | DNS4 英文部分 | 【出处 [26]】 |
| FSPEN | 未找到 | 未找到 | 未找到 | 未找到 | 论文全文未取到 |
| DPCRN | 1 张 GTX 1080Ti | 8 | 早停（验证集 10 轮不降就停） | 约 500 小时，5 秒片段，现场卷混响 | 【出处 [27]】 |
| PercepNet | 未找到 | 未找到 | 未找到 | 语音 120 小时 + 噪声 80 小时，48 kHz | 【出处 [28]】 |
| RNNoise | 未找到（Keras） | 未找到 | 未找到 | 语音 6 小时 + 噪声 4 小时，造出 140 小时 | 【出处 [29]】 |

补充：
- 论文里 GTCRN 是 2.37 万参数【出处 [23]】。本项目写的 4.8 万参数，可能是改过的版本，请大总管核一下。
- 这些论文几乎都不写显卡和总时长。唯一写了的 DPCRN 用的是一张 1080Ti（2017 年的消费卡）。【出处 [27]】
- 推论：这类模型一张消费级显卡就够训，不需要多卡。【推断】

**瓶颈在 CPU 装数据、不在显卡：有出处的部分**
- PyTorch 官方：`num_workers=0` 时装数据是同步的，训练进程要等数据；应设 `num_workers > 0` 并按 CPU、显卡、数据位置调。【出处 [30]】
- NVIDIA DALI 官方：「这些数据处理流程目前在 CPU 上跑，已经成了瓶颈」，所以它把预处理挪到显卡上。【出处 [31]】
- `torch-audiomentations`：「我们不希望数据增强成为训练速度的瓶颈」，所以支持在显卡上做增强。【出处 [32]】
- DeepFilterNet 专门用 Rust 写了装数据和增强的库 `libDF`，读预先打包的 HDF5 文件。【出处 [33]】
- 学术测评：显卡利用率低说明 CPU 或数据传输在限速。【出处 [34]】
- 「小降噪模型训练瓶颈在 CPU 混音」这一具体说法：未找到直接出处。上面是一般结论，用到本项目是【推断】。

**对本项目意味着什么**【推断】
- 混响卷积、重采样都别在每步现场做重活：重采样提前做完（第三节第 1 条）。
- 混响可以用 FFT 卷积，或挪到显卡上做（`torch-audiomentations` 一类）。
- 开训第一件事：看显卡利用率。长期低于约 70% 就是 CPU 在拖。

## 三、云上训练的做法

下面 8 条按「最该先做」排序，前 3 条最重要。

1. **数据先转成 16 kHz，再打成大分片放网络存储盘。**
   - 理由一：网络存储盘读大文件 200–400 MB/s，读大量小文件慢；RunPod 自己也把小于 256 KiB 的文件打包来避开这个问题。【出处 [3][35]】
   - 理由二：`webdataset` 用顺序读的 tar 分片，本地盘可比随机读快 3–10 倍，也适合对象存储。【出处 [36]】
   - 做法：每片几百 MB；开机后把分片从网络盘复制到机器本地盘再训。【复制到本地是推断】

2. **定时存检查点，可从任意一步续训。**
   - 存模型参数、优化器状态、轮数、损失，放进一个字典一起存。【出处 [37]】
   - 抢占式实例被杀前只有约 5 秒，所以必须定时存，不能等收到信号再存。【出处 [22]；结论是推断】
   - 检查点写到网络存储盘；关键的再传一份到仓外的对象存储。【推断】

3. **训完自动删机，花费设上限。**
   - 训练脚本最后一行跑 `runpodctl pod delete $RUNPOD_POD_ID`；或开机时就排一个 `sleep <上限>; runpodctl pod stop` 兜底。【出处 [4][5]】
   - `stop` 只停显卡，机器自带盘还收钱；数据都在网络盘上时用 `delete`。【出处 [2][4]】
   - 打开 RunPod 低余额提醒，防止欠费删盘。【出处 [3]】

4. **环境锁死：固定 Docker 镜像与 torch 版本。**
   - PyTorch 官方：不同版本、不同提交、不同平台之间，结果不保证能复现。【出处 [38]】
   - 做法：用带具体版本号的镜像（如 `pytorch/pytorch:<版本>-cuda<版本>`），依赖用锁文件。【推断】

5. **固定随机种子，包括装数据的进程。**
   - 要分别给 torch、Python `random`、NumPy 播种；`DataLoader` 要传 `worker_init_fn` 和带种子的 `generator`。【出处 [38]】
   - 现场混音全靠随机数，这条决定能不能重跑出同一组混音。【推断】

6. **每次运行记下配置与 git 哈希。**
   - W&B：`wandb.init(config=...)` 记配置；默认记当前 git 提交哈希，有未提交改动会存成 `diff.patch`。【出处 [39][40]】
   - 不用 W&B 也行：开训时把配置文件、`git rev-parse HEAD`、`git status` 输出写进本次运行目录。【推断】

7. **实验跟踪：W&B 或 TensorBoard 二选一。**
   - W&B 要联网和账号，能远程看曲线。TensorBoard 不出网，日志写在网络盘上。【推断】
   - 如果用 W&B，注意它会上传代码差异；不想外传就设 `WANDB_IGNORE_GLOBS=*.patch`。【出处 [40]】这和铁律 12「外发检查」相关。

8. **密钥不进镜像。**
   - Docker 官方：构建参数和环境变量「会留在最终镜像里」，不适合传密钥。【出处 [41]】
   - RunPod 用平台的 Secrets：模板里写 `{{ RUNPOD_SECRET_名字 }}`，开机时才注入成环境变量。【出处 [42]】
   - 本机的 RunPod API key 仍放 `.env.local`（D96 已定）。

## 四、没查到、要开机后实测的

这几项决定最终选卡和花费，网上查不到，只能实测：
- RunPod 各机房到 OpenSLR 欧洲镜像、Zenodo 的下载速度。
- GTCRN 一步的显卡时间，和 3090 / 4090 上数据管线每秒能造几批。
- Qwen3-ASR、SCOREQ、Distill-MOS 实际显存占用。
- RunPod Spot 的真实警告时长（官方文档未找到）。

## 链接

1. RunPod GPU Cloud 页（卡、vCPU、内存、价）：https://www.runpod.io/gpu-cloud
2. RunPod 价目页：https://www.runpod.io/pricing ；计费文档：https://docs.runpod.io/pods/pricing
3. RunPod 网络存储盘文档：https://docs.runpod.io/storage/network-volumes
4. RunPod 管理机器文档：https://docs.runpod.io/pods/manage-pods
5. `runpodctl` 停机参考：https://docs.runpod.io/runpodctl/reference/runpodctl-stop-pod
6. Vast.ai 价目页：https://vast.ai/pricing
7. Vast.ai 4090 第三方价：https://computeprices.com/providers/vast/gpus/rtx4090 ；https://getdeploying.com/reference/cloud-gpu/nvidia-rtx-4090
8. Vast.ai 实例类型文档：https://docs.vast.ai/documentation/instances/choosing/instance-types
9. Vast.ai 计费文档：https://docs.vast.ai/documentation/instances/pricing
10. Lambda 价目页：https://lambda.com/pricing
11. Lambda 文件系统文档：https://docs.lambda.ai/public-cloud/filesystems
12. Modal 价目页：https://modal.com/pricing
13. HF Jobs 文档：https://huggingface.co/docs/huggingface_hub/guides/jobs
14. Colab 单元数实测（博客）：https://mccormickml.com/2024/04/23/colab-gpus-features-and-pricing/
15. AutoDL 第三方价：https://gitcode.csdn.net/6a02e16154b52172bc73777e.html
16. AutoDL 计费文档：https://www.autodl.com/docs/price/
17. AutoDL 实例数据文档：https://www.autodl.com/docs/instance_data/
18. OpenSLR 28 号资源页：https://www.openslr.org/28/
19. Zenodo 2023 升级问题：https://blog.zenodo.org/2023/10/19/2023-10-19-upgrade-issues
20. Zenodo 2026-01 改进说明：https://blog.zenodo.org/2026/01/28/2026-01-28-improvements-and-support-expectations/
21. Zenodo 下载慢讨论：https://discourse.julialang.org/t/downloads-download-or-run-wget-much-slower-than-direct-wget/133501 ；断点续传：https://blog.zenodo.org/2021/12/07/2021-12-07-hardening-our-service
22. RunPod Spot 5 秒说法（社区论坛）：https://answeroverflow.com/m/1262811070603464704
23. GTCRN ICASSP 2024 海报：https://sigport.org/sites/all/modules/pubdlcnt/pubdlcnt.php?fid=9363 ；仓库：https://github.com/Xiaobin-Rong/gtcrn
24. Align-ULCNet：https://arxiv.org/html/2410.13620
25. DeepFilterNet v1：https://arxiv.org/html/2110.05588v2
26. DeepFilterNet2：https://arxiv.org/html/2205.05474v1
27. DPCRN：https://arxiv.org/abs/2107.05429
28. PercepNet：https://arxiv.org/abs/2008.04259
29. RNNoise：https://arxiv.org/html/1709.08243v3
30. PyTorch 性能调优指南：https://docs.pytorch.org/tutorials/recipes/recipes/tuning_guide.html
31. NVIDIA DALI 文档：https://docs.nvidia.com/deeplearning/dali/user-guide/docs/index.html
32. torch-audiomentations：https://github.com/asteroid-team/torch-audiomentations
33. DeepFilterNet 仓库：https://github.com/rikorose/deepfilternet
34. TBD 训练测评论文：https://arxiv.org/pdf/1803.06905
35. RunPod 卷缓存文档（小文件打包）：https://docs.runpod.io/serverless/development/volume-cache
36. WebDataset：https://github.com/webdataset/webdataset
37. PyTorch 存取模型教程：https://docs.pytorch.org/tutorials/beginner/saving_loading_models.html
38. PyTorch 可复现说明：https://docs.pytorch.org/docs/2.15/notes/randomness.html
39. W&B 配置文档：https://docs.wandb.ai/guides/track/config/
40. W&B 记 git 提交：https://docs.wandb.ai/support/save_git_commit_associated_run ；代码面板：https://docs.wandb.ai/models/app/features/panels/code
41. Docker 构建密钥：https://docs.docker.com/build/building/secrets/
42. RunPod Secrets：https://docs.runpod.io/pods/templates/secrets
