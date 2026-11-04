# vdn-speaker-lab · P2 离线实验命令行

> exp005（`docs/experiments/exp005-p2-speaker/`）用的命令行。2026-10-31 p2fix 从 `vdn-speaker` 的 `offline` feature 挪出，让 `vdn-speaker` 的依赖闭包只剩 dsp + earshot + realfft（+ 可选 ort）。
> tier = tool：不进实时线程、不进产品 bin。规约 §2.3 的 `vdn-eval` 建起来后并进去。

依赖：`vdn-speaker`（带 `onnx`）· `vdn-engine` · `vdn-models` · ort · hound · serde_json。

```sh
cargo build --release -p vdn-speaker-lab      # → target/release/vdn-speaker-lab
```

```sh
vdn-speaker-lab denoise <noisy.wav> <wet.wav> [--model-path gtcrn.onnx]
vdn-speaker-lab run   <noisy.wav> <out_dir> --wet <wet.wav> [--speaker-onnx ecapa.onnx] [--replay log.json --tag .x] [--overwrite] [门控参数…]
vdn-speaker-lab envelope <noisy.wav> <wet.wav> <out_prefix>
vdn-speaker-lab fbank <in.wav> <out.f32>
vdn-speaker-lab embed <in.wav>
vdn-speaker-lab bench-split <in.wav> [--reps 6] [--win 1.5] [--judge-hop 0.5] [--speaker-onnx ecapa.onnx]
```

`run` 一次出 `raw / floor / energy / vad / personal / aggressive` 六个版本和一份 `log.json`（判断回路每次判决、注册事件、逐 hop 增益与耗时）。目标文件已存在时拒绝覆盖，除非换 `--tag` 或加 `--overwrite`；`--replay` 必须带 `--tag`。

`bench-split`（exp015）按判断回路的窗长 / 步长切窗，把一次声纹计算拆成 fbank · 整个 embed · 推理（两者之差）· 注册与余弦分别计时，输出各项 p50 / p90 / 最慢（µs）。计时要拿名额、插电、`caffeinate`。
