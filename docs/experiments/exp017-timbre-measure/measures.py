"""exp017：音色项三种量法（代码版移植 · 标准文档版 · 1/3 倍频程参照）。

代码版照 `src/eval/src/verifiers/perceptual.rs` 的 `v_timbre`（main 90ed08f2）逐步移植；
文档版照 `specs/rubric/denoise.md` N4 原文。移植是否一致由 `crosscheck` 与 Rust 读数比对。
"""

import numpy as np

MASK_FRAME = 160  # 10 ms @ 16 kHz，同 analysis.rs


def spectrogram(x, n_fft, hop):
    """同 analysis.rs Spectrogram：帧 k 覆盖 [(k+1)hop − n_fft, (k+1)hop)，开头补零，周期 Hann。"""
    win = 0.5 - 0.5 * np.cos(2 * np.pi * np.arange(n_fft) / n_fft)
    nfr = len(x) // hop
    pad = np.concatenate([np.zeros(n_fft - hop), x[: nfr * hop]])
    idx = np.arange(n_fft)[None, :] + hop * np.arange(nfr)[:, None]
    fr = pad[idx] * win
    return np.abs(np.fft.rfft(fr, axis=1)) ** 2


def centers(nfr, n_fft, hop):
    return (np.arange(nfr) + 1) * hop - n_fft // 2


def truth_from_clean(clean):
    """同 analysis.rs truth_from_clean：20 ms 帧，比最大帧低 35 dB 以内算有声。返回 10 ms 掩码。"""
    f = 320
    p = np.array([np.mean(clean[i : i + f] ** 2) for i in range(0, len(clean) - f + 1, f)])
    on = p > p.max() * 10 ** (-3.5)
    nf = len(clean) // MASK_FRAME
    m = np.zeros(nf, bool)
    for i, v in enumerate(on):
        if v:
            m[2 * i : min(2 * i + 2, nf)] = True
    return m


def dilate(m, g):
    out = m.copy()
    for s in range(1, g + 1):
        out[s:] |= m[:-s]
        out[:-s] |= m[s:]
    return out


def speech_frames(owner, nfr, n_fft, hop):
    c = centers(nfr, n_fft, hop)
    mf = np.where(c >= 0, c // MASK_FRAME, -1)
    return np.array([k for k in range(nfr) if 0 <= mf[k] < len(owner) and owner[mf[k]]])


def quiet_frames(noise_only, nfr, n_fft, hop):
    """同 perceptual.rs noise_spec_frames：中心所在掩码帧前后各 2 帧都只有噪声。"""
    c = centers(nfr, n_fft, hop)
    q = noise_only
    out = []
    for k in range(nfr):
        if c[k] < 0:
            continue
        m = c[k] // MASK_FRAME
        if m >= 2 and m + 2 < len(q) and q[m - 2 : m + 3].all():
            out.append(k)
    return np.array(out, int)


def db10(v):
    return 10 * np.log10(np.maximum(v, 1e-30))


CODE_BANDS = [(100, 1000), (1000, 2000), (2000, 4000), (4000, 7900)]
DOC_BANDS = [(100, 300), (300, 500), (2000, 5000), (5000, 8000)]


def band_mean(sg, ks, bands, sr, n_fft, upper_inclusive=False):
    hz = np.arange(sg.shape[1]) * sr / n_fft
    if len(ks) == 0:
        return np.zeros(len(bands))
    mean = sg[ks].mean(axis=0)
    out = []
    for lo, hi in bands:
        sel = (hz >= lo) & ((hz <= hi) if upper_inclusive else (hz < hi))
        out.append(mean[sel].sum())
    return np.array(out)


def code_timbre(out, clean, owner, noise_only=None, sr=16000):
    """代码版：512/256，四带相对四带总和，扣停顿帧残留，取最大绝对偏差（带符号）。"""
    so, sc = spectrogram(out, 512, 256), spectrogram(clean, 512, 256)
    sp = speech_frames(owner, len(so), 512, 256)
    q = quiet_frames(noise_only, len(so), 512, 256) if noise_only is not None else np.array([], int)
    pc = band_mean(sc, sp, CODE_BANDS, sr, 512)
    po = band_mean(so, sp, CODE_BANDS, sr, 512)
    pn = band_mean(so, q, CODE_BANDS, sr, 512)
    po = np.maximum(np.maximum(po - pn, 0.1 * po), 1e-20)
    dev = (db10(po) - db10(po.sum())) - (db10(pc) - db10(pc.sum()))
    return dev[np.argmax(np.abs(dev))], dev


def doc_timbre(out, clean, owner, sr=16000):
    """文档版 N4：1024 Hann 跳 512；有声段整体电平对齐（全频带功率）；四带功率差取均方根。"""
    so, sc = spectrogram(out, 1024, 512), spectrogram(clean, 1024, 512)
    sp = speech_frames(owner, len(so), 1024, 512)
    pc = band_mean(sc, sp, DOC_BANDS, sr, 1024, upper_inclusive=True)
    po = band_mean(so, sp, DOC_BANDS, sr, 1024, upper_inclusive=True)
    align = db10(so[sp].mean(axis=0).sum()) - db10(sc[sp].mean(axis=0).sum())
    dev = db10(po) - db10(pc) - align
    return float(np.sqrt(np.mean(dev**2))), dev


def third_octave_bands(lo=100, hi=8000):
    cs = 1000 * 2 ** (np.arange(-10, 10) / 3)
    return [(c / 2 ** (1 / 6), c * 2 ** (1 / 6)) for c in cs if c / 2 ** (1 / 6) >= lo * 0.89 and c * 2 ** (1 / 6) <= hi]


def third_oct_dev(out, clean, owner, sr=16000):
    """参照：1/3 倍频程（中心 100 Hz–6.3 kHz），有声段整体电平对齐后的逐带偏差。"""
    bands = third_octave_bands()
    so, sc = spectrogram(out, 512, 256), spectrogram(clean, 512, 256)
    sp = speech_frames(owner, len(so), 512, 256)
    pc = band_mean(sc, sp, bands, sr, 512)
    po = band_mean(so, sp, bands, sr, 512)
    align = db10(po.sum()) - db10(pc.sum())
    return db10(po) - db10(pc) - align, bands


def erb_bands(lo=100.0, hi=7900.0, step=1.0):
    """按 ERB 数等距切带（Glasberg & Moore 1990：ERB 数 = 21.4·log10(1 + 0.00437·f)）。"""
    e = lambda f: 21.4 * np.log10(1 + 0.00437 * f)  # noqa: E731
    inv = lambda n: (10 ** (n / 21.4) - 1) / 0.00437  # noqa: E731
    # 两端钉在 lo / hi，带数 = 跨度 ÷ 步长取整，每带约 1 步宽（codex 026 第 1 轮 #2：旧写法只留整步，末带止于约 7.2 kHz）
    n = max(1, int(round((e(hi) - e(lo)) / step)))
    edges = inv(np.linspace(e(lo), e(hi), n + 1))
    return list(zip(edges[:-1], edges[1:]))


def erb_shape(out, clean, owner, noise_only=None, sr=16000, step=1.0, sym=False):
    """候选第三种：1024/512，ERB 带（默认 1 ERB 宽，100–7900 Hz），只用有声帧，
    扣停顿帧残留噪声（同代码版），去掉整体电平（减各带偏差均值），
    报 形状差标准差 · 相邻带斜率差标准差 · 最大绝对形状差。"""
    bands = erb_bands(step=step)
    so, sc = spectrogram(out, 1024, 512), spectrogram(clean, 1024, 512)
    sp = speech_frames(owner, len(so), 1024, 512)
    q = quiet_frames(noise_only, len(so), 1024, 512) if noise_only is not None else np.array([], int)
    pc = band_mean(sc, sp, bands, sr, 1024)
    po = band_mean(so, sp, bands, sr, 1024)
    pn = band_mean(so, q, bands, sr, 1024)
    po = np.maximum(np.maximum(po - pn, 0.1 * po), 1e-20)
    if sym:  # 参考也扣它自己停顿帧里的能量（混响尾巴），两边同样处理
        pr = band_mean(sc, q, bands, sr, 1024)
        pc = np.where(pc > 0, np.maximum(np.maximum(pc - pr, 0.1 * pc), 1e-20), 0.0)
    keep = pc > 0
    d = db10(po[keep]) - db10(pc[keep])
    # 先按中位数去掉整体增益，再截下限，最后去均值：截断点随增益平移，整体音量不改读数（codex 026 第 1 轮 #3）
    d = d - np.median(d)
    d = np.maximum(d, -40.0)  # 相对中位数最多记 −40 dB，空带不拖成 −100 dB 级
    d = d - d.mean()
    return dict(std=float(d.std()), slope=float(np.diff(d).std()), max=float(np.abs(d).max()), n=int(keep.sum())), d
