# BUT 混响库下载清单（给 cloudrun）

这份清单给 cloudrun 照着在云上下载 BUT Speech@FIT Reverb Database 的冲激响应与底噪（人定 D111 补充）。
写的人：traindata（2026-11-25）。本机没下载任何音频；下面的字节数和日期是对下载服务器发 HEAD 请求读到的。

## 1. 下什么

只下这一个包：

| 文件 | 地址 | 字节数 | 服务器上的修改时间 |
|---|---|---|---|
| BUT_ReverbDB_rel_19_06_RIR-Only.tgz | http://merlin.fit.vutbr.cz/ReverbDB/BUT_ReverbDB_rel_19_06_RIR-Only.tgz | 9,308,593,693 | Wed, 05 Jun 2019 12:31:28 GMT |

同一目录里还有这几个包，**都不下**：
- `BUT_ReverbDB_rel_19_06_LibriSpeech-Only.tgz`：重放的 LibriSpeech test-clean，正是 exp013 评测集的人声来源（D57）
- `BUT_ReverbDB_rel_19_06_HUB5_Eval-Only.tgz`、`BUT_ReverbDB_rel_19_06_SPETEMonly.tgz`：重放语音，用不上，且有 LDC 限制
- `BUT_ReverbDB_rel_20_10_RIR-Only.tgz`（26,272,444,726 字节）：超过人定的 20 GB

## 2. 校验

- **上游没给校验码**：下载目录和 `read_me.txt` 里都没有 md5 或 sha256（实读）。服务器的 ETag `22ad5de1d-58a92c6c861a7` 是 Apache 用「文件大小 + 修改时间」拼的，不是内容哈希（推断：`0x22ad5de1d` = 9,308,593,693）
- 所以按「首次下载时记下」来做：
  1. 字节数必须正好是 9,308,593,693
  2. `gzip -t` 通过，`tar -tzf` 能完整列出目录
  3. 下完把 md5 与 sha256 都记进 `checksums.txt`，并发给 traindata；traindata 把它钉进 `sources.json`，以后再下就按这个值核
- `fetch_archives.sh` 的清单要三列「名字 md5 地址」，md5 对不上就停。这次没有可填的 md5：请 cloudrun 自己定怎么跑（例如先单独用 aria2c 下、核完字节数再算 md5），不要随便填一个值让它报 BAD 后继续

## 3. 解压

- 只解 `RIR` 与 `silence` 两类目录，其他一律不解（推断：RIR-Only 包里本来就只有这两类，`read_me.txt` 写着标 `S` 的录音只有 RIR 和 silence）
- 解前先列目录核一遍：`tar -tzf BUT_ReverbDB_rel_19_06_RIR-Only.tgz | grep -c -i librispeech` 应为 0；不是 0 就停下发 traindata
- 解到 `/workspace/train-raw/but-reverbdb/`
- 解完请发：总文件数、`*/RIR/*.wav` 与 `*/silence/*.wav` 各多少个、每个房间（路径第一层）的文件数、解压后大小（`du -sh`）

## 4. 许可与署名

- `read_me.txt` 写的是 Apache License 2.0（实读，副本存在本仓 `third_party/BUT_ReverbDB-read_me.txt`，sha256 `4f62b0015e5c61a9f34672430c9f3d96a210bae1530c9e29fa325c6e6969b9fa`）；论文写的是 CC BY 4.0。人定按 Apache 2.0 处理：附许可文本与版权声明「Copyright 2018 Brno University of Technology, Faculty of Information Technology, BUT Speech@FIT」
- 引用：I. Szöke, M. Skácel, L. Mošner, J. Paliesek, J. Černocký, "Building and evaluation of a real room impulse response dataset", IEEE JSTSP 13(4), 2019

## 5. 下完以后（traindata 做，不在这次）

1. `sources.json` 加一组，钉字节数与首次下载记的 md5 / sha256
2. `split_rules.json` 加一条：按房间切（路径第一层，如 `VUT_FIT_C236`），9 个房间
3. 底噪过人声筛查（`screen.py`，pi 在写）。`read_me.txt` 说有一个位置录底噪时放着收音机，那一段必须筛掉
4. 噪声池：会议室用 C236 与 3 间办公室的底噪；酒店房间用 R112；电话亭照 3A 只取风扇类底噪。冲激响应另议
5. 登记跟第二版差距表选定的场景一起进下一批
