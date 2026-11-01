"""exp008 · T4 urgent_bsrnn：URGENT 2026 Track1 BSRNN 判别式基线（非因果，整段离线）。

模型结构照 resources/urgent2026_challenge_track1/baseline_code/models/bsrnn.py 读懂后自己搭：
espnet2 的 STFTEncoder/STFTDecoder（48 k 下 n_fft 960 / hop 480，按 fs 等比缩放 → 16 k 下 320/160）
+ BSRNNSeparator(num_channels=196, num_layers=6, target_fs=48000, causal=False)。
模型原生多采样率，直接以 16 k 输入（与官方 inference.py 一样 fs=输入采样率），不重采样。
不做官方 inference.py 里的「峰值归一到 0.9」（那只是写盘习惯，会改变绝对电平）。
用法：<torch-venv>/bin/python enh/urgent_bsrnn.py <build> [--jobs N] [--limit N]
"""
import sys
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import _torchse  # noqa: E402

CKPT = Path.home() / "vdn-data/models/urgent2026_bsrnn/bsrnn.ckpt"


def factory():
    import pickle
    import types

    import torch
    from espnet2.enh.decoder.stft_decoder import STFTDecoder
    from espnet2.enh.encoder.stft_encoder import STFTEncoder
    from espnet2.enh.separator.bsrnn_separator import BSRNNSeparator

    class BSRNN_SE(torch.nn.Module):
        def __init__(self, num_channel=196, num_layer=6):
            super().__init__()
            self.encoder = STFTEncoder(n_fft=960, hop_length=480, use_builtin_complex=True, default_fs=48000)
            self.decoder = STFTDecoder(n_fft=960, hop_length=480, default_fs=48000)
            self.bsrnn = BSRNNSeparator(input_dim=self.encoder.output_dim, num_spk=1,
                                        num_channels=num_channel, num_layers=num_layer,
                                        target_fs=48000, causal=False)

        def forward(self, wav, lens, fs):
            f, fl = self.encoder(wav, lens, fs=fs)
            pre, fl, _ = self.bsrnn(f, fl, None)
            out, _ = self.decoder(pre[0], lens, fs=fs)
            return out

    # Lightning ckpt 里的 hparams 是 baseline_code.config.Config 对象；不导入它，用占位类解开
    class _U(pickle.Unpickler):
        def find_class(self, m, n):
            if m.startswith("baseline_code"):
                return type(n, (dict,), {"__setstate__": lambda s, st: s.update(st if isinstance(st, dict) else {})})
            return super().find_class(m, n)

    pm = types.SimpleNamespace(Unpickler=_U, load=lambda f, **k: _U(f, **k).load(), __name__="pm")
    ck = torch.load(CKPT, map_location="cpu", weights_only=False, pickle_module=pm)
    mc = ck["hyper_parameters"]["cfg"]["model_configs"]
    model = BSRNN_SE(num_channel=mc["num_channel"], num_layer=mc["num_layer"])
    sd = {k[len("se_model."):]: v for k, v in ck["state_dict"].items() if k.startswith("se_model.")}
    model.load_state_dict(sd, strict=True)
    model.eval()

    def enhance(x):
        with torch.no_grad():
            w = torch.from_numpy(x).float().view(1, -1)
            y = model(w, torch.tensor([w.shape[-1]]), 16000)
        return y.squeeze(0).numpy()[: len(x)]

    return enhance


RECORD = {
    "family": "BSRNN（频带分割 RNN，判别式）· URGENT 2026 Track1 官方基线",
    "causal": False,
    "sr_model": 16000,
    "weights": [{
        "path": "~/vdn-data/models/urgent2026_bsrnn/bsrnn.ckpt",
        "sha256": "5d6b24eb0ba387428f3490a36238d17902cdc96da534fd2707a8e44f0d2431c8",
        "bytes": 151456890,
        "license": "MIT",
        "license_source": "HF lichenda/icassp_2026_urgent_baseline 模型卡 cardData.license=mit；代码 Apache-2.0（resources/urgent2026_challenge_track1/LICENSE.txt）",
    }],
    "code_source": "resources/urgent2026_challenge_track1/baseline_code/models/bsrnn.py（结构读懂后自写于本脚本）+ pip espnet==202611 的 STFTEncoder/STFTDecoder/BSRNNSeparator（--no-deps，另装 torch_complex、typeguard）",
    "notes": "原生多采样率模型，16 k 直接输入（STFT 320/160，161 频点，频带切分按输入频点截断，与官方 inference 同路径）。"
             "ckpt hparams: num_channel=196, num_layer=6；state_dict strict 加载通过。"
             "未做官方 inference.py 的峰值归一 0.9。每个工作进程载一次权重，各段独立前向（模型无跨段状态）。"
             "怪现象（冒烟实测）：输出 RMS 约为输入的 3–6 倍（-26 dBFS 输入 → 约 -10 dBFS 输出）；对输入电平敏感，"
             "同一段乘 0.25/1/4 的 SI-SDR：c00_20dB 6.6/14.0/15.9，c00_-5dB -27.3/-22.0/-19.9（-5 dB cafe 段锁错目标）。"
             "按约定输入原样喂入、不做电平预归一。",
}

if __name__ == "__main__":
    _torchse.run("urgent_bsrnn", factory, RECORD)
