"""exp005 附：Rust fbank / 声纹 与 Python 逐值对拍。

- fbank 基准 = kaldi-native-fbank（pip `kaldi-native-fbank`，Kaldi 的 C++ 重写，sherpa-onnx / WeSpeaker runtime 用它），
  选项与 `src/speaker/src/fbank.rs::FbankConfig::wespeaker()` 一致：25/10 ms、80 mel、hamming、dither 0、snip_edges、
  去直流、预加重 0.97、low 20 Hz、high = 奈奎斯特、不带能量；输入 ×32768（`resources/wespeaker/wespeaker/bin/infer_onnx.py:31-50`）
- 声纹基准 = Python onnxruntime 跑同一个 ONNX，输入 = 基准 fbank 减均值（CMN）；比 Rust `vdn-speaker embed` 的输出
素材：exp005 的 c01 主人轨 / 混合轨 各取 3 s 段 + c06 主人轨 1.5 s 段（写成临时 WAV 喂 Rust）
用法（项目根下）：<主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp005-p2-speaker/parity.py <vdn-speaker 可执行文件>
输出：本目录 results/parity.json
"""
import json
import subprocess
import sys
from pathlib import Path

import kaldi_native_fbank as knf
import numpy as np
import onnxruntime as ort
import soundfile as sf

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "exp003-offline-ab"))
from _paths import MAIN  # noqa: E402

SRC = MAIN / "_tmp/20261031-exp005/src"
TMP = MAIN / "_tmp/20261031-exp005/parity"
ONNX = Path.home() / "vdn-data/models/wespeaker/voxceleb_ECAPA512.onnx"


def knf_fbank(x):
    o = knf.FbankOptions()
    o.frame_opts.samp_freq = 16000
    o.frame_opts.dither = 0.0
    o.frame_opts.frame_length_ms = 25
    o.frame_opts.frame_shift_ms = 10
    o.frame_opts.snip_edges = True
    o.frame_opts.remove_dc_offset = True
    o.frame_opts.preemph_coeff = 0.97
    o.frame_opts.window_type = "hamming"
    o.mel_opts.num_bins = 80
    o.mel_opts.low_freq = 20
    o.mel_opts.high_freq = 0
    o.use_energy = False
    o.use_log_fbank = True
    o.use_power = True
    f = knf.OnlineFbank(o)
    f.accept_waveform(16000, (x * 32768.0).tolist())
    f.input_finished()
    return np.array([f.get_frame(i) for i in range(f.num_frames_ready)], dtype=np.float32)


def main():
    exe = sys.argv[1]
    TMP.mkdir(parents=True, exist_ok=True)
    segs = [("c01_owner", "c01_owner.wav", 20.0, 3.0), ("c01_mix", "c01_mix.wav", 20.0, 3.0),
            ("c06_owner_1p5", "c06_owner.wav", 10.0, 1.5)]
    so = ort.SessionOptions()
    so.intra_op_num_threads = so.inter_op_num_threads = 1
    sess = ort.InferenceSession(str(ONNX), so, providers=["CPUExecutionProvider"])
    res = []
    for name, f, t0, dur in segs:
        x, sr = sf.read(str(SRC / f), dtype="float32")
        x = x[int(t0 * sr): int((t0 + dur) * sr)]
        w = TMP / f"{name}.wav"
        sf.write(str(w), x, sr, subtype="FLOAT")
        # fbank
        out = TMP / f"{name}.f32"
        info = json.loads(subprocess.run([exe, "fbank", str(w), str(out)], capture_output=True, text=True, check=True).stdout)
        rs = np.fromfile(out, dtype="<f4").reshape(info["frames"], info["bins"])
        py = knf_fbank(x)
        assert rs.shape == py.shape, (rs.shape, py.shape)
        d = np.abs(rs - py)
        # 声纹
        e_rs = np.array(json.loads(subprocess.run([exe, "embed", str(w), "--speaker-onnx", str(ONNX)],
                                                  capture_output=True, text=True, check=True).stdout)["emb"], dtype=np.float32)
        feats = (py - py.mean(axis=0))[None]
        e_py = sess.run(["embs"], {"feats": feats})[0][0]
        e_py = e_py / np.linalg.norm(e_py)
        res.append({"seg": name, "frames": int(rs.shape[0]), "fbank_max_abs": float(d.max()), "fbank_mean_abs": float(d.mean()),
                    "fbank_rel_max": float((d / (np.abs(py) + 1e-3)).max()),
                    "emb_dim": int(len(e_rs)), "emb_cos": float(np.dot(e_rs, e_py)), "emb_max_abs": float(np.abs(e_rs - e_py).max())})
        print(res[-1])
    out = {"reference": f"kaldi-native-fbank {getattr(knf, '__version__', '?')} + onnxruntime {ort.__version__}",
           "model": str(ONNX.name), "segments": res}
    (HERE / "results").mkdir(exist_ok=True)
    (HERE / "results/parity.json").write_text(json.dumps(out, indent=1) + "\n")


if __name__ == "__main__":
    main()
