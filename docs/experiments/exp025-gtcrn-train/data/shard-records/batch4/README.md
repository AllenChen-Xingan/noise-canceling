# 第四批分片（/workspace/shards/batch4）删前留底

网络盘上的 batch4 已于 2026-11-26 按 D118 删除（cloudrun；trainer-c6 确认无依赖）。用它的 build：bc0e9e7b566421f0 与 5 个消融变体。

- `shards.sha256`：94 个分片文件的 sha256（云上 `sha256sum`，路径相对 batch4 目录）
- `du.txt`：删前总字节数 22289901880
- `index.sha256`：`index.json` 的 sha256，等于 build bc0e9e7b 记录的 `index_sha256`（traindata 本机核过）
- `index.json` 本体 37.6 MB，超过 10 MB 不进仓（CLAUDE.md 铁律 8）；在主仓 `_tmp/20261126-cloudrun-batch5/batch4-final/index.json`，打包件 `batch4-final.tar.gz` sha256 aa74b90d817a9f3003958724144923380dbd4c5aac225eec0e338e59b09d707e
