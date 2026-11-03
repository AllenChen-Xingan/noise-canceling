"""exp008 · T7 gmap_se：G-MaP-SE（MP-SENet 式 TS-Conformer + 说话人 GMM 先验，非因果，整段离线）。

调用 resources/G-MaP-SE 的模型定义（只读导入，不写 __pycache__），权重 ckpt/g_best + ckpt/clean_gmm_mu_k.pt。
说话人条件：官方 inference_g_map_se.py 与 MPNet.forward 都是对**带噪输入本身**跑 ECAPA（wespeaker
voxceleb_ECAPA512.onnx）取嵌入，再对 192 个干净 GMM 均值做 softmax 软分配 → 无需注册语音。本脚本照做，
不用 owner 干净轨、也不用任何额外注册素材。
前处理照官方：wav 先乘 norm_factor=sqrt(N/Σx²)（单位 RMS），STFT 400/100/400、压缩 0.3，出来再除回去。
istft(center=True) 输出长度为 hop 的整数倍（8 s=128000 恰为 100 的整数倍），不足/多余按输入长度截补。
`pesq` 包只在训练评测用，导入时用空模块占位（推理不调用）。
用法：<torch-venv>/bin/python enh/gmap_se.py <build> [--jobs N] [--limit N]
"""
import sys
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import _torchse  # noqa: E402

G = Path("/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/resources/G-MaP-SE")
ECAPA = Path.home() / "vdn-data/models/wespeaker/voxceleb_ECAPA512.onnx"


def factory():
    import json
    import types

    import numpy as np
    import torch

    sys.dont_write_bytecode = True
    if "pesq" not in sys.modules:
        stub = types.ModuleType("pesq")
        stub.pesq = lambda *a, **k: (_ for _ in ()).throw(RuntimeError("pesq stub"))
        sys.modules["pesq"] = stub
    sys.path.insert(0, str(G))
    from dataset import mag_pha_istft, mag_pha_stft
    from env import AttrDict
    from models.model_g_map_se import MPNet

    h = AttrDict(json.loads((G / "ckpt/config.json").read_text()))
    h.ecapa_model_path = str(ECAPA)
    h.gmm_embed_dir = str(G / "ckpt")
    model = MPNet(h)
    sd = torch.load(G / "ckpt/g_best", map_location="cpu", weights_only=False)["generator"]
    model.load_state_dict(sd, strict=True)
    model.eval()

    def enhance(x):
        with torch.no_grad():
            w = torch.from_numpy(x).float()
            nf = torch.sqrt(len(w) / torch.sum(w ** 2.0))
            w = (w * nf).unsqueeze(0)
            amp, pha, _ = mag_pha_stft(w, h.n_fft, h.hop_size, h.win_size, h.compress_factor)
            a_g, p_g, _ = model(w, amp, pha)
            y = mag_pha_istft(a_g, p_g, h.n_fft, h.hop_size, h.win_size, h.compress_factor)
            y = (y / nf).squeeze(0).numpy()
        n = len(x)
        return y[:n] if len(y) >= n else np.pad(y, (0, n - len(y)))

    return enhance


RECORD = {
    "family": "G-MaP-SE（MP-SENet 式 TS-Conformer 幅相双解码 + ECAPA 说话人嵌入 → 干净 GMM 先验门控融合）",
    "causal": False,
    "sr_model": 16000,
    "weights": [
        {"path": "resources/G-MaP-SE/ckpt/g_best", "sha256": None, "bytes": None,
         "license": "MIT", "license_source": "resources/G-MaP-SE/LICENSE（Copyright (c) 2026 Hello3orld）"},
        {"path": "resources/G-MaP-SE/ckpt/clean_gmm_mu_k.pt", "sha256": None, "bytes": None,
         "license": "MIT", "license_source": "resources/G-MaP-SE/LICENSE"},
        {"path": "~/vdn-data/models/wespeaker/voxceleb_ECAPA512.onnx", "sha256": None, "bytes": None,
         "license": "Apache-2.0", "license_source": "~/vdn-data/models/wespeaker/README.hf.md（HF 卡 license: apache-2.0）"},
    ],
    "code_source": "resources/G-MaP-SE（models/model_g_map_se.py、dataset.py、env.py 只读导入；推理流程照 inference_g_map_se.py）",
    "notes": "无注册：说话人嵌入取自带噪输入本身（官方推理即如此，MPNet.forward 内部对 noisy_wav 跑 ECAPA），"
             "未使用 owner 干净轨或任何额外同说话人素材。训练数据 VoiceBank+DEMAND（窄域，I 级）。"
             "单位 RMS 归一后推理再还原电平（照官方）。每个工作进程载一次权重，各段独立前向（无跨段状态）。"
             "怪现象：本格上表现很差（冒烟 S-SNR c00：20 dB 混合 SI-SDR 18.7 → 输出 2.2；-5 dB → -10.5），且压低电平。"
             "管线自检：VBD 样例 + 5 dB 白噪 → 5.0→12.8 / 5.0→11.1 dB，说明加载与前后处理无误；"
             "诊断（仅诊断，不作注册）：把 owner 轨本身喂入，输出对 owner 仅 12.2 dB，owner+5 dB 白噪 → 8.6 dB——"
             "推测为 VBD 窄域训练对本格的房间/设备染色失配（假设）。CPU 争用下每段约 100 s。",
}


def _fill_hashes():
    for w in RECORD["weights"]:
        p = Path(w["path"].replace("~", str(Path.home()))) if w["path"].startswith("~") else \
            Path("/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling") / w["path"]
        w["sha256"] = _torchse.sha256(p)
        w["bytes"] = p.stat().st_size


if __name__ == "__main__":
    _fill_hashes()
    _torchse.run("gmap_se", factory, RECORD)
