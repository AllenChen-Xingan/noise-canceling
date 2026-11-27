"""训练记录的两样补充（#236）：梯度裁剪统计 · 开发评估存音频。

- `ClipStats`：每步喂一次「裁剪前的梯度范数」，在记录点吐这一段窗口的读数（被裁的步占比 / 步数 / 中位数 / 最大值）。
  窗口只在 `flush()` 时清零；**不进检查点**——续训后窗口从空开始（`train.py` 里也写了这句）。
- `save_dev_audio`：每次开发评估把固定几段开发录音的增强结果存成 wav（16 kHz · 16 位 PCM），放进
  `<运行目录>/dev_audio/step-NNNNNNN/enh-i.wav`；第一次调用另存同几段的带噪 / 干净参照到 `dev_audio/ref/`。
  只为给人听与事后对照，不参与任何判据。

写盘失败（磁盘满、目录不可写、libsndfile 自己报错）只回一个错误字符串，训练照跑：这几条 wav 不是训练的必需品。
停下信号（信号 / 时长与花费上限）一到就立刻不写：存档排在 `best.pt` 之后，不拖住该停的运行。
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
CLIP_EPS = 1e-6  # 与 torch.nn.utils.clip_grad_norm_ 分母里加的是同一个数
# 写 wav 会抛的错：磁盘满 / 不可写是 OSError；libsndfile 自己的错是 `sf.SoundFileError`（RuntimeError 子类，
# **不是** OSError）。只包住写盘那几句——模型前向出的错不该被当成「写盘失败」吞掉。
WRITE_ERRORS = (OSError, sf.SoundFileError)


def _err_text(e: BaseException) -> str:
    """错误字符串。`LibsndfileError.__str__` 要回头问 libsndfile、极端情况下自己会抛：错误路径不许再抛一次。"""
    try:
        msg = str(e)
    except Exception:  # noqa: BLE001 —— 连错本身都读不出来，就只留类型名
        msg = "<无法读取错误信息>"
    return f"{type(e).__name__}: {msg}"


def clip_coef(gn: float, max_norm: float) -> float:
    """`clip_grad_norm_` 的缩放系数 `max_norm / (total_norm + 1e-6)`，float32 算（与 torch 逐位一致）。

    float32 是必要的：3.0 − 1e-6 这一步在 float32 里系数恰好等于 1（没改梯度），用 float64 算会误判成被裁。
    """
    return float(max_norm / (torch.tensor(float(gn), dtype=torch.float32) + CLIP_EPS))


class ClipStats:
    """一段窗口内的梯度裁剪统计。`add()` 每步一次，`flush()` 吐出读数并清零。

    被裁的判据与 `torch.nn.utils.clip_grad_norm_` 逐位一致：缩放系数 `max_norm / (gn + 1e-6)` 小于 1 才算这一步
    被裁。**等于**阈值时系数略小于 1（分母多了 1e-6），torch 会缩一点点，所以也算被裁；3.0 − 1e-6 时系数在
    float32 里恰好是 1，不算。
    """

    def __init__(self, threshold: float):
        self.threshold = float(threshold)
        self.norms: list[float] = []

    def add(self, gn: float) -> None:
        self.norms.append(float(gn))

    def clipped(self, gn: float) -> bool:
        """这一步的梯度被 torch 改过没有。"""
        return clip_coef(gn, self.threshold) < 1.0

    def flush(self) -> dict:
        """窗口的读数；窗口里一步都没有就返回 `{}`（什么都不多写）。"""
        if not self.norms:
            return {}
        norms, self.norms = self.norms, []
        clipped = sum(1 for g in norms if self.clipped(g))
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


def save_dev_audio(model, dev, device, root: Path, step: int, should_stop=lambda: False) -> str | None:
    """把开发份前 `DEV_AUDIO_N` 段（跨批累计，一批不够就接着下一批）的增强结果存成 wav。

    运行期间模型切评估模式、关梯度；结束后把**每个模块**原来的 `training` 标记逐个还回去（冻结的批归一化
    在训练里本来就在评估模式，不能被顺手打开）。不调用任何随机数，RNG 状态原样不动。
    `should_stop()` 在每批之前、每份文件之前各问一次：要停就立刻返回 None（不是错误），已经写完的留着。
    成功（含中途停下）返回 None；写盘出错返回错误字符串，不抛。
    """
    root = Path(root)
    out_dir = root / "dev_audio" / f"step-{step:07d}"
    ref_dir = root / "dev_audio" / "ref"
    # 参照只补缺、不重写：上一次写到一半失败（OSError / libsndfile 报错）时，这次评估把缺的几份补齐；已在的一律不动。
    # 判据是「文件在不在」，不是「目录在不在」——目录建好之后写文件才失败的话，按目录判就永远缺那几份。
    ref_files = {f"{tag}-{j}.wav" for j in range(DEV_AUDIO_N) for tag in ("noisy", "clean")}
    ref_missing = {n for n in ref_files if not (ref_dir / n).exists()}
    modes = [(m, m.training) for m in model.modules()]
    try:
        model.eval()
        with torch.no_grad():
            if should_stop():  # 一开头就该停：连目录都不建
                return None
            try:
                out_dir.mkdir(parents=True, exist_ok=True)
                ref_dir.mkdir(parents=True, exist_ok=True)
            except WRITE_ERRORS as e:
                return _err_text(e)
            done = 0
            for b in dev:
                if done >= DEV_AUDIO_N or should_stop():
                    break
                take = min(DEV_AUDIO_N - done, b["noisy"].shape[0])
                noisy = b["noisy"][:take].to(device)
                clean = b["clean"][:take].to(device)
                pred = model(losses.stft(noisy), _cond_for(model, take, device))
                enh = losses.istft(pred, noisy.shape[-1])
                for i in range(take):
                    if should_stop():  # 写下一份之前再问一次：停下的信号一到就不再多花时间
                        return None
                    j = done + i
                    try:
                        _write_wav(out_dir / f"enh-{j}.wav", enh[i])
                        if f"noisy-{j}.wav" in ref_missing:
                            _write_wav(ref_dir / f"noisy-{j}.wav", noisy[i])
                        if f"clean-{j}.wav" in ref_missing:
                            _write_wav(ref_dir / f"clean-{j}.wav", clean[i])
                    except WRITE_ERRORS as e:
                        return _err_text(e)
                done += take
    finally:
        for m, was in modes:
            m.training = was  # 直接改标记：m.train(was) 会连带盖掉子模块各自的状态
    return None
