"""候选训练噪声的机器筛查（gap-fill-options.md 第 1 节）：每条片段给 keep / drop 与理由。

- 判据只有一个正本：`screen_rules.json`。代码里不写任何阈值，改判据只改那份 json
- 两个神经模型（Silero VAD · PANNs CNN14）是**注入**的：本模块不在导入时碰它们，
  测试用假函数喂；真要模型时用 `default_vad()` / `default_tagger()` 惰性导入
- 声学量全部纯 numpy（重采样用 exp013 `dsp.resample`，与 `shard.py` 同一份正本）
用法（云上）：
    python3 screen.py --root /workspace/noise --list candidates.txt --scene meeting-room --out screen.jsonl
    python3 screen.py ... --no-models        # 只跑声学量，空跑用（人声与标签记「未查」）
"""
import argparse
import hashlib
import json
import re
import sys
from pathlib import Path

import numpy as np
import soundfile as sf

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
EXP013 = HERE.parents[1] / "exp013-scene-suite"
sys.path.insert(0, str(EXP013))
from dsp import SR as SR_VAD, resample  # noqa: E402  SR=16000，与 exp013 同一份正本

RULES = HERE / "screen_rules.json"
SR_TAG = 2 * SR_VAD  # 标签模型吃 32 kHz
SEG = 4096  # 长期平均频谱的窗口
FIT_HZ = (100.0, 4000.0)  # 长期平均频谱斜率只在 100 Hz … 4 kHz 上拟合
TAIL_MS = 20  # 冲激响应最后这段的 EDC 由个别样本决定，不算衰减（见 rt60_schroeder）
DBFS_FLOOR = 1e-7  # 整秒静音会给出 -inf；压到 -140 dBFS（低于 16 位本底）。逐秒电平的正本，diagnose.py 也用它


def _x(x):
    return np.asarray(x, np.float64).ravel()


# ---------------------------------------------------------------- 人声

def frames_speech_prob(x, sr, vad):
    """逐帧说话概率。vad 是注入的可调用体：吃 16 kHz 单声道，回逐帧概率。
    帧长由 vad 自己定（规则里的 frame_ms 描述期望值；Silero 实际吃 512 样本 ≈ 32 毫秒一块）。"""
    x16 = resample(_x(x), sr, SR_VAD)
    if x16.size == 0:
        raise ValueError("片段是空的")
    return np.asarray(vad(x16), np.float64).ravel()


def speech_frac(probs, thr):
    """说话概率 > thr 的帧占比。"""
    p = np.asarray(probs, np.float64).ravel()
    if p.size == 0:
        raise ValueError("一帧说话概率都没有")
    return float(np.count_nonzero(p > thr) / p.size)


# ---------------------------------------------------------------- 标签

def top_tags(x, sr, tagger, k):
    """前 k 个 AudioSet 标签与概率，按概率降序。tagger 吃 32 kHz 单声道，回 {标签: 概率}。"""
    x32 = resample(_x(x), sr, SR_TAG)
    if x32.size == 0:
        raise ValueError("片段是空的")
    probs = dict(tagger(x32))
    items = sorted(((str(lab), float(p)) for lab, p in probs.items()), key=lambda kv: -kv[1])
    return items[:int(k)]


# ---------------------------------------------------------------- 混响

def rt60_schroeder(rir, sr):
    """Schroeder 反向积分法算混响时间：在 -5 … -35 dB 上拟合直线，退 60 dB 的时间 = T30 × 2。
    末尾补的零先剪掉。够不到 -35 dB（衰减被截断）就报错：末尾 TAIL_MS 毫秒里的 EDC
    只剩几个样本的能量，硬拟合会把收尾的陡降当成衰减，算出的时间偏短且无意义。"""
    h = _x(rir)
    nz = np.flatnonzero(h != 0)
    if nz.size < 2:
        raise ValueError("冲激响应几乎是空的")
    h = h[nz[0]: nz[-1] + 1]
    edc = np.cumsum((h * h)[::-1])[::-1]
    if edc[0] <= 0:
        raise ValueError("冲激响应全零")
    db = 10.0 * np.log10(edc / edc[0])
    t = np.arange(db.size) / float(sr)
    over5 = np.flatnonzero(db <= -5.0)
    over35 = np.flatnonzero(db[:-int(round(sr * TAIL_MS / 1000.0))] <= -35.0)
    if over5.size == 0 or over35.size == 0:
        raise ValueError("衰减没走到 -35 dB（冲激响应太短或尾巴被截断），算不出混响时间")
    a, b = int(over5[0]), int(over35[0])
    if b <= a + 1:
        raise ValueError("衰减太快，-5 到 -35 dB 之间没有可拟合的点")
    slope = float(np.polyfit(t[a:b + 1], db[a:b + 1], 1)[0])
    if slope >= 0:
        raise ValueError("衰减曲线没有下降，算不出混响时间")
    return float(-60.0 / slope)


# ---------------------------------------------------------------- 声学量

def level_std_db(x, sr, sec=1.0):
    """平稳度：逐秒 RMS 电平（dBFS）的标准差。片段不足 2 秒没法算，报错。"""
    x = _x(x)
    n = int(round(sec * sr))
    if n < 2:
        raise ValueError("采样率太低，切不出 1 秒的块")
    blocks = x.size // n
    if blocks < 2:
        raise ValueError("片段不足 2 秒，算不出逐秒电平")
    rms = np.sqrt(np.mean(x[: blocks * n].reshape(blocks, n) ** 2, axis=1))
    rms = np.maximum(rms, DBFS_FLOOR)
    return float(np.std(20.0 * np.log10(rms)))


def a_weighting_db(f):
    """IEC 61672 的 A 计权解析式（dB）。1 kHz 处为 0 dB。"""
    f = np.asarray(f, np.float64)
    f2 = f * f
    num = (12194.0 ** 2) * (f2 ** 2)
    den = (f2 + 20.6 ** 2) * np.sqrt((f2 + 107.7 ** 2) * (f2 + 737.9 ** 2)) * (f2 + 12194.0 ** 2)
    safe = den > 0
    ratio = np.zeros_like(f2)
    ratio[safe] = num[safe] / den[safe]
    return 20.0 * np.log10(np.maximum(ratio, 1e-300)) + 2.0


def a_weighted_dbfs(x, sr):
    """A 计权电平（dBFS）：频域乘 A 计权增益，再取整体 RMS。"""
    x = _x(x)
    if x.size < 2:
        raise ValueError("片段太短，算不了 A 计权电平")
    f = np.fft.rfftfreq(x.size, 1.0 / sr)
    gain = 10.0 ** (a_weighting_db(f) / 20.0)
    y = np.fft.irfft(np.fft.rfft(x) * gain, x.size)
    rms = float(np.sqrt(np.mean(y * y)))
    if rms <= 0:
        raise ValueError("片段全零，算不了 A 计权电平")
    return float(20.0 * np.log10(rms))


def ltas_slope_db_per_oct(x, sr):
    """长期平均频谱的斜率（dB/倍频程）：Welch 平均功率谱，在 FIT_HZ 上对 log2(f) 拟合直线。"""
    x = _x(x)
    nseg = min(SEG, x.size)
    if nseg < 32:
        raise ValueError("片段太短，算不了长期平均频谱")
    win = np.hanning(nseg)
    step = max(1, nseg // 2)
    segs = [x[i:i + nseg] * win for i in range(0, x.size - nseg + 1, step)]
    if not segs:
        raise ValueError("片段太短，算不了长期平均频谱")
    power = np.mean([np.abs(np.fft.rfft(s)) ** 2 for s in segs], axis=0)
    f = np.fft.rfftfreq(nseg, 1.0 / sr)
    lo, hi = FIT_HZ
    keep = (f >= lo) & (f <= hi) & (power > 0)
    if np.count_nonzero(keep) < 4:
        raise ValueError("100 Hz … 4 kHz 里没有足够的频点")
    return float(np.polyfit(np.log2(f[keep]), 10.0 * np.log10(power[keep]), 1)[0])


# ---------------------------------------------------------------- 判定

def screen_clip(x, sr, scene, rules, vad=None, tagger=None, rir=None):
    """一条片段过一遍判据。vad / tagger / rir 缺了就跳过对应的检查，理由里记「未查：…」。
    返回 {"verdict": "keep" | "drop", "reasons": [...], "measures": {...}}。"""
    if scene not in rules["scenes"]:
        raise ValueError(f"screen_rules.json 里没有场景 {scene}（有：{'、'.join(sorted(rules['scenes']))}）")
    conf = rules["scenes"][scene]
    sp = rules["speech"]
    x = _x(x)
    reasons, measures, drop = [], {}, False

    # 1. 无人声 —— Silero VAD 的逐帧说话概率
    name = "人声占比（Silero VAD）"
    frac = None
    if vad is None:
        reasons.append(f"未查：{name}（没给 VAD）")
    else:
        try:
            frac = speech_frac(frames_speech_prob(x, sr, vad), sp["vad_prob"])
        except ValueError as e:
            reasons.append(f"未查：{name}（{e}）")
    if frac is not None:
        measures["speech_frac"] = frac
        if frac > sp["max_frac"]:
            drop = True
            reasons.append(f"有人声：说话帧占比 {frac:.3f} 超过 {sp['max_frac']}")

    # 2. 无人声 —— AudioSet 标签里的 Speech，以及像不像目标场景
    name = "场景标签（PANNs CNN14）"
    tags = None
    if tagger is None:
        reasons.append(f"未查：{name}（没给标签模型）")
    else:
        try:
            tags = top_tags(x, sr, tagger, conf["top_k"])
        except ValueError as e:
            reasons.append(f"未查：{name}（{e}）")
    if tags is not None:
        measures["top_tags"] = tags
        labels = [lab for lab, _ in tags]
        sprob = dict(tags).get(sp["tag"], 0.0)
        measures["speech_tag_prob"] = sprob
        if sprob > sp["tag_max"]:
            drop = True
            reasons.append(f"有人声：标签 {sp['tag']} 概率 {sprob:.3f} 超过 {sp['tag_max']}")
        if not set(labels) & set(conf["tags_any"]):
            drop = True
            given = " 或 ".join(conf["tags_any"])
            reasons.append(f"不像 {scene}：前 {conf['top_k']} 个标签（{'、'.join(labels)}）里没有 {given}")

    # 3. 混响时间（只对写明了 rt60_s 的场景）
    if "rt60_s" in conf:
        name = "混响时间（Schroeder）"
        rt = None
        if rir is None:
            reasons.append(f"未查：{name}（没给冲激响应）")
        else:
            try:
                rt = rt60_schroeder(rir, sr)
            except ValueError as e:
                reasons.append(f"未查：{name}（{e}）")
        if rt is not None:
            measures["rt60_s"] = rt
            lo, hi = conf["rt60_s"]
            if not lo <= rt <= hi:
                drop = True
                reasons.append(f"混响时间不符：{rt:.2f} 秒不在 {scene} 的 {lo}–{hi} 秒内")

    # 4. 平稳度
    name = "平稳度（逐秒电平标准差）"
    limit = rules["stationarity"]["max_level_std_db"]
    try:
        std = level_std_db(x, sr)
    except ValueError as e:
        reasons.append(f"未查：{name}（{e}）")
    else:
        measures["level_std_db"] = std
        if std > limit:
            drop = True
            reasons.append(f"不平稳：逐秒电平标准差 {std:.2f} dB 超过 {limit}")

    # 5. 只记录、不判定的声学量（给报告和人工比对用）
    for name, fn in (("A 计权电平", a_weighted_dbfs), ("长期频谱斜率", ltas_slope_db_per_oct)):
        try:
            measures[{"A 计权电平": "a_dbfs", "长期频谱斜率": "ltas_db_per_oct"}[name]] = fn(x, sr)
        except ValueError as e:
            reasons.append(f"未查：{name}（{e}）")

    return {"verdict": "drop" if drop else "keep", "reasons": reasons, "measures": measures}


# ---------------------------------------------------------------- 模型（惰性导入）

def default_vad():
    """Silero VAD。没装就让云上去装，不在导入时炸。"""
    try:
        import torch
        from silero_vad import load_silero_vad
    except ImportError as e:
        raise SystemExit(f"没装 Silero VAD（{e}）：云上请 pip install silero-vad 后再跑，或用 --no-models 空跑")
    model = load_silero_vad()
    step = 512  # Silero 只吃 512 样本（16 kHz 下约 32 毫秒）一块

    def vad(x16k):
        probs, buf = [], _x(x16k).astype(np.float32)
        with torch.no_grad():
            model.reset_states()
            for i in range(0, buf.size - step + 1, step):
                probs.append(float(model(torch.from_numpy(buf[i:i + step]), SR_VAD)))
        return np.asarray(probs, np.float64)

    return vad


def default_tagger(checkpoint=None, device="cpu"):
    """PANNs CNN14 的 AudioSet 标签。没装就让云上去装。"""
    try:
        from panns_inference import AudioTagging, labels
    except ImportError as e:
        raise SystemExit(f"没装 PANNs（{e}）：云上请 pip install panns-inference 后再跑，或用 --no-models 空跑")
    net = AudioTagging(checkpoint_path=checkpoint, device=device)
    # labels 的两种导出形式：0.1.1 直接是名字的 list，旧版是带 .labels 的对象
    names = labels.labels if hasattr(labels, "labels") else labels

    def tagger(x32k):
        clip, _ = net.inference(_x(x32k).astype(np.float32)[None, :])
        if len(names) != clip.shape[-1]:
            raise ValueError(
                f"PANNs 标签数对不上：模型输出 {clip.shape[-1]} 个，标签列表 {len(names)} 个"
                "（装错版本的 panns-inference？）")
        return dict(zip(names, [float(p) for p in clip[0]]))

    return tagger


# ---------------------------------------------------------------- 命令行

def read_list(path):
    out = []
    for line in Path(path).read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if line and not line.startswith("#"):
            out.append(line)
    return out


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def read_first_channel(path):
    x, sr = sf.read(str(path), dtype="float32", always_2d=True)
    return x[:, 0].astype(np.float64), int(sr)


def reason_key(text):
    """汇总用：去掉括号里的细节和数字，把同一条判据的命中归到一起。"""
    return re.sub(r"[-+]?[0-9.]+", "N", text.split("（")[0])


def run(args):
    rules = json.loads(Path(args.rules).read_text(encoding="utf-8"))
    files = read_list(args.list)
    vad = None if args.no_models else default_vad()
    tagger = None if args.no_models else default_tagger()
    root = Path(args.root)
    keep = drop = 0
    counts = {}
    with open(args.out, "w", encoding="utf-8") as out:
        for rel in files:
            path = root / rel
            x, sr = read_first_channel(path)
            res = screen_clip(x, sr, args.scene, rules, vad=vad, tagger=tagger)
            rec = {"path": rel, "sha256": sha256_file(path), "sr": sr,
                   "seconds": round(x.size / sr, 3), **res}
            out.write(json.dumps(rec, ensure_ascii=False) + "\n")
            if res["verdict"] == "keep":
                keep += 1
            else:
                drop += 1
            for r in res["reasons"]:
                key = reason_key(r)
                counts[key] = counts.get(key, 0) + 1
    print(f"共 {len(files)} 条：keep {keep} · drop {drop}")
    for key in sorted(counts):
        print(f"  {key} — {counts[key]} 次")


def main(argv=None):
    ap = argparse.ArgumentParser(description="候选训练噪声的机器筛查（判据在 screen_rules.json）")
    ap.add_argument("--root", required=True, help="片段所在目录；清单里的路径相对它")
    ap.add_argument("--list", required=True, help="候选清单，一行一个相对路径，# 开头是注释")
    ap.add_argument("--scene", required=True, help="目标场景，须是 screen_rules.json 里的键")
    ap.add_argument("--out", required=True, help="输出 jsonl，一条片段一行")
    ap.add_argument("--rules", default=str(RULES), help="判据 json（默认同目录的 screen_rules.json）")
    ap.add_argument("--no-models", action="store_true", help="不装 VAD 与标签模型，只跑声学量（空跑）")
    run(ap.parse_args(argv))


if __name__ == "__main__":
    main()
