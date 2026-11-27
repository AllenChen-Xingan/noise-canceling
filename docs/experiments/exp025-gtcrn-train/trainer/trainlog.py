"""训练记录的两样补充（#236）：梯度裁剪统计 · 开发评估存音频。

- `ClipStats`：每步喂一次「裁剪前的梯度范数」，在记录点吐这一段窗口的读数（被裁的步占比 / 步数 / 中位数 / 最大值）。
  窗口只在 `flush()` 时清零；**不进检查点**——续训后窗口从空开始（`train.py` 里也写了这句）。
- `save_dev_audio`：每次开发评估把固定几段开发录音的增强结果存成 wav（16 kHz · 16 位 PCM），放进
  `<运行目录>/dev_audio/step-NNNNNNN/enh-i.wav`；第一次调用另存同几段的带噪 / 干净参照到 `dev_audio/ref/`。
  只为给人听与事后对照，不参与任何判据。

写盘失败（磁盘满、目录不可写）只回一个错误字符串，训练照跑：这几条 wav 不是训练的必需品。
"""
from __future__ import annotations

import os
import statistics
from pathlib import Path

import numpy as np
import soundfile as sf
import torch

import losses

SR = 16_000  # 与 losses.SR 同一个采样率；这里只用于写 wav 的头部
# 每次开发评估存几段：**只在这里定**（别处一律引用本常量，别写字面量 4）
DEV_AUDIO_N = 4


class ClipStats:
    """一段窗口内的梯度裁剪统计。`add()` 每步一次，`flush()` 吐出读数并清零。

    被裁的判据与 `torch.nn.utils.clip_grad_norm_` 一致：范数**严格大于**阈值才算这一步被裁
    （等于阈值时缩放系数是 1，等于没裁）。
    """

    def __init__(self, threshold: float):
        self.threshold = float(threshold)
        self.norms: list[float] = []

    def add(self, gn: float) -> None:
        self.norms.append(float(gn))

    def flush(self) -> dict:
        """窗口的读数；窗口里一步都没有就返回 `{}`（什么都不多写）。"""
        if not self.norms:
            return {}
        norms, self.norms = self.norms, []
        clipped = sum(1 for g in norms if g > self.threshold)
        return {"clip_n": len(norms), "clip_frac": clipped / len(norms),
                "gn_pre_median": float(statistics.median(norms)), "gn_pre_max": float(max(norms))}


def _cond_for(model, batch_size: int, device):
    """条件输入一律全零（D98）。正本在 `train.cond_for`；这里延迟导入，因为 `train` 顶部就 import 本模块。"""
    import train
    return train.cond_for(model, batch_size, device)


def _write_wav(path: Path, wav: torch.Tensor) -> None:
    """先写同目录的临时名、再改名：人看到的文件要么完整要么不存在（`runrec` 写文件的同一做法）。"""
    tmp = path.with_name(path.name[: -len(".wav")] + ".tmp.wav")
    x = np.clip(wav.detach().cpu().numpy(), -1.0, 1.0)
    sf.write(tmp, x, SR, subtype="PCM_16")
    os.replace(tmp, path)


def save_dev_audio(model, dev, device, root: Path, step: int) -> str | None:
    """把开发份前 `DEV_AUDIO_N` 段（跨批累计，一批不够就接着下一批）的增强结果存成 wav。

    运行期间模型切评估模式、关梯度；结束后把**每个模块**原来的 `training` 标记逐个还回去（冻结的批归一化
    在训练里本来就在评估模式，不能被顺手打开）。不调用任何随机数，RNG 状态原样不动。
    成功返回 None；写盘出错返回错误字符串，不抛。
    """
    root = Path(root)
    out_dir = root / "dev_audio" / f"step-{step:07d}"
    ref_dir = root / "dev_audio" / "ref"
    need_ref = not ref_dir.exists()  # 参照只写一次：目录已在就当作写过了
    modes = [(m, m.training) for m in model.modules()]
    try:
        model.eval()
        with torch.no_grad():
            out_dir.mkdir(parents=True, exist_ok=True)
            if need_ref:
                ref_dir.mkdir(parents=True, exist_ok=True)
            done = 0
            for b in dev:
                if done >= DEV_AUDIO_N:
                    break
                take = min(DEV_AUDIO_N - done, b["noisy"].shape[0])
                noisy = b["noisy"][:take].to(device)
                clean = b["clean"][:take].to(device)
                pred = model(losses.stft(noisy), _cond_for(model, take, device))
                enh = losses.istft(pred, noisy.shape[-1])
                for i in range(take):
                    j = done + i
                    _write_wav(out_dir / f"enh-{j}.wav", enh[i])
                    if need_ref:
                        _write_wav(ref_dir / f"noisy-{j}.wav", noisy[i])
                        _write_wav(ref_dir / f"clean-{j}.wav", clean[i])
                done += take
    except OSError as e:
        return f"{type(e).__name__}: {e}"
    finally:
        for m, was in modes:
            m.training = was  # 直接改标记：m.train(was) 会连带盖掉子模块各自的状态
    return None
