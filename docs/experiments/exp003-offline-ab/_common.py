"""exp003 脚本共用的小函数（只放一份，score.py / enhance.py / eaten_words.py 都从这里取）。"""
import re


def vad_mask(ref, frame=320, thr_db=-35.0, dilate=5):
    """按干净参考做能量 VAD：20 ms 帧，高于最响帧 thr_db 算有声，前后各扩 dilate 帧（100 ms）。
    返回 (逐样本布尔掩码, 掩码覆盖的样本数)。"""
    import numpy as np  # 放在函数里：eaten_words.py 用系统 python 跑，不需要 numpy
    nf = len(ref) // frame
    e = 10 * np.log10(np.mean(ref[: nf * frame].reshape(nf, frame) ** 2, axis=1) + 1e-12)
    m = e > e.max() + thr_db
    d = m.copy()
    for k in range(1, dilate + 1):
        d[k:] |= m[:-k]
        d[:-k] |= m[k:]
    return np.repeat(d, frame), nf * frame


def tokens(text, lang):
    """转写切词：英文按词；中文汉字逐字，夹在中文里的英文 / 数字按词（否则一句英文会按字母算成几十个插入）。"""
    if lang == "zh":
        return re.findall(r"[一-鿿]|[a-z0-9']+", text.lower())
    return re.findall(r"[a-z0-9']+", text.lower())
