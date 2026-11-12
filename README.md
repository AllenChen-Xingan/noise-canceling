# vdn

> Open-source real-time voice denoising for macOS meetings: the other side hears you
> clearly and naturally, and no words get swallowed.

**License: GPL-3.0** (copyleft). See [License](#license).

**Status: early prototype.** There is no installer and no virtual microphone yet.
Today `vdn` can denoise WAV files offline and run a live mic → speaker loop for testing.

## Usage

Denoise a recording with the default model ([GTCRN][gtcrn]):

```console
$ vdn denoise noisy.wav clean.wav
```

The output is a 32-bit float WAV. Three safety layers are on by default:

- **Passthrough fallback:** if the model fails or produces bad output for a frame, that frame passes through unchanged.
- **Attenuation floor:** during pauses, the background is turned down by at most 12 dB, never removed completely.
- **Fast open, slow release:** when speech starts, the floor opens at once, so the first syllables are not cut.

You can tune or turn off each layer:

```console
$ vdn denoise --floor-db -18 --hangover-ms 200 noisy.wav clean.wav
$ vdn denoise --no-safety noisy.wav raw-model.wav    # model output only, for comparison
```

List audio devices (does not open a stream or ask for microphone permission):

```console
$ vdn devices
输入 MacBook Air Microphone（系统默认） · 默认 48000 Hz · 1 声道 · 支持 48 kHz：是
输出 MacBook Air Speakers（系统默认） · 默认 48000 Hz · 2 声道 · 支持 48 kHz：是
```

Hear it live (opens the mic and plays to the speakers, so **use headphones**):

```console
$ vdn run --denoise --seconds 30
```

`vdn --help` lists every command. CLI messages are currently in Chinese.

## Install

Requirements: macOS on Apple silicon and a Rust toolchain (the version is pinned in `rust-toolchain.toml`).

```console
$ git clone https://github.com/AllenChen-Xingan/noise-canceling.git
$ cd noise-canceling
$ cargo build --release -p vdn-cli
$ ./target/release/vdn --help
```

The model weights are not in this repository. Download `gtcrn_simple.onnx` from
[Xiaobin-Rong/gtcrn][gtcrn] (MIT). Then either put it at
`~/vdn-data/models/gtcrn/gtcrn_simple.onnx`, set `VDN_GTCRN_ONNX`, or pass `--model-path`.

## How quality is judged

A claim such as "it works" or "it is better" must come from a verifier with a score,
a threshold and a pass/fail result. Opinions do not count.

- **CI**, on every pull request: DSP checks, tests, lint, and a file-size limit (no code file over 600 lines). Run it locally with `cargo run -p xtask -- ci`.
- **QA**, on this machine before a release: model and meeting-scene scorecards (`vdn-eval scorecard`).
- **People**: blind listening tests for what machines cannot judge yet. The results are used to calibrate the verifiers.

## Background

This project is built mostly by AI coding agents, directed by one person.
[`CLAUDE.md`](CLAUDE.md) holds the rules the agents follow.
The design notes are in Chinese:

- [`.42cog/intent.md`](.42cog/intent.md): goal, hard problems, and non-goals
- [`docs/research/`](docs/research/): candidate models, datasets and their licenses
- [`docs/experiments/`](docs/experiments/): one folder per experiment, with parameters, hashes and results (no audio)
- [`docs/plan/001_decisions-20261029-b1.md`](docs/plan/001_decisions-20261029-b1.md): the decision log

## License

GPL-3.0. See [`LICENSE`](LICENSE).
Model weights and datasets keep their own licenses.

[gtcrn]: https://github.com/Xiaobin-Rong/gtcrn
