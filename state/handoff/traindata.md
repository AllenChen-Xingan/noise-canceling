# 现状页 · traindata（issue #156，批次 20261116-b19）

状态：进行中

## 手上
实体：训练数据集 · 数据切分 · 混音配方。产出都在 `docs/experiments/exp025-gtcrn-train/data/`。

## 做到哪
读完交代与调研；核过第一批 A1–A5 与 exp013 原料的下载地址（只发 HEAD 请求，本机没下数据）。正在写下载脚本与来源清单。

## 下一步
下载脚本 → 切分 → 交集检查 → 混音配方 → 分片与加载接口 → exp013 云上重建脚本 → 自审 + codex。

## 依赖
- 早期混响 100 ms：正本 `docs/experiments/exp013-scene-suite/rooms.py` 的 `EARLY_REVERB_MS` 与 `early_rir`（evalcmp 提交 f7c71e15，分支 `feat/eval-paired-compare`，未合并 main）。本分支要等它进 main 后合进来，相关测试才会绿

## 待人拍板
（暂无）

## 发现
- QUT-NOISE 的下载服务器 HEAD 响应里没有 `Accept-Ranges`，断点续传可能不支持；脚本遇到服务器不认续传时从头下

## 流水
- 2026-11-16 报到；读 014 · 027 下载清单与第六节 · 034 第四节与修正 · exp013 leak.py / build.py / pools.py / rirs.py
- 2026-11-16 HEAD 核下载地址：OpenSLR 12（train-clean-100 6,387,309,499 字节 · dev-clean · test-other · test-clean）与 md5 清单；OpenSLR 28 1,311,166,223 字节；QUT-NOISE 六个 zip（合计 8,278,905,862 字节）；TUM 风噪 Zenodo 6687982 一个 zip 137,141,772 字节（md5 已取）；DEMAND Zenodo 1227121 全部文件 md5
- 2026-11-16 evalcmp 告知早期混响常量正本位置；回复：配方 import 它，不另写
