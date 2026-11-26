"""BUT Speech@FIT Reverb 库的夹具（tests/test_but.py 与 tests/test_mix_loader.py 共用）：临时目录现造极短的 16 kHz wav。

布局照真库（上游 read_me：`<房间>/MicID01/<扬声器布置>/<麦克风>/silence/…`）：
两个房间（会议室 C236 · 酒店房间 Room112）× 各几次扬声器布置 × 两个麦克风。
C236 的第 3 次布置里第 2 个扬声器放收音机，那一次必须排除；它的 `spk_meta.txt` 写的是上游真文件
（`tests/fixtures/but_spk_meta/VUT_FIT_L207…spk_meta.txt`，里面有 `$EnvSpk2…` 行），干净布置写的是上游真文件
`VUT_FIT_C236…spk_meta.txt`（只有 1 个扬声器）——不是照 read_me 编的假文件。
"""
import json
from pathlib import Path

import numpy as np
import soundfile as sf

SR = 16000
SEC = 0.5
ROOT = "train-raw/but-reverbdb"
C236 = "VUT_FIT_C236"
ROOM112 = "Hotel_SkalskyDvur_Room112"
MICS = ("01", "02")
# 房间 → [(扬声器布置, 是否放收音机), …]
SETUPS = {
    C236: [("SpkID01_20190503_S", False), ("SpkID02_20190503_S", False), ("SpkID03_20190503_S", True)],
    ROOM112: [("SpkID01_20171223_S", False), ("SpkID02_20171223_S", False)],
}
# 上游真 spk_meta.txt 的副本（正本：上游 2019 RIR-Only 包里那两个文件）
FIX = Path(__file__).resolve().parent / "fixtures" / "but_spk_meta"
C236_META = "VUT_FIT_C236__MicID01__SpkID01_20190503_S__spk_meta.txt"
L207_META = "VUT_FIT_L207__MicID01__SpkID02_20171227_S__spk_meta.txt"


def wav_rel(room, setup, mic, ver="v00"):
    return f"{ROOT}/{room}/MicID01/{setup}/{mic}/silence/silence_16kHz_60sec.{ver}.wav"


def all_files():
    return [wav_rel(room, setup, mic) for room, setups in SETUPS.items() for setup, _r in setups for mic in MICS]


def radio_files():
    """放收音机的那次布置的全部 wav（必须被 `exclude_if_meta` 排除）。"""
    return [wav_rel(room, setup, mic) for room, setups in SETUPS.items() for setup, r in setups if r for mic in MICS]


def spk_meta_src(radio):
    """这次布置用的上游真 spk_meta.txt（在仓里的副本；内容不在这里手写）。"""
    return FIX / (L207_META if radio else C236_META)


def spk_meta(radio):
    """真文件的内容，按 utf-8 读回文本（文件里是非 ASCII 字符，如 «¼»）。"""
    return spk_meta_src(bool(radio)).read_text(encoding="utf-8")


def spk_meta_bytes(radio):
    """真文件的原始字节：写出去逐字节相同，不经过解码再编码。"""
    return spk_meta_src(bool(radio)).read_bytes()


def meta_path(root, rel, file="../../spk_meta.txt"):
    """规则里 meta 文件相对 wav 所在目录解析（与 split.exclude_if_meta 同一条算法）。"""
    return Path((Path(root) / rel).parent / file).resolve()


def make_tree(root, radio_meta=True):
    """造夹具树；返回全部 wav 的相对路径。radio_meta=False 时不写放收音机那次布置的 spk_meta.txt。"""
    root = Path(root)
    rng = np.random.default_rng(0)
    for room, setups in SETUPS.items():
        for setup, r in setups:
            if radio_meta or not r:
                p = meta_path(root, wav_rel(room, setup, MICS[0]))
                p.parent.mkdir(parents=True, exist_ok=True)
                p.write_bytes(spk_meta_bytes(r))
            for mic in MICS:
                f = root / wav_rel(room, setup, mic)
                f.parent.mkdir(parents=True, exist_ok=True)
                sf.write(str(f), 0.01 * rng.standard_normal(int(SEC * SR)), SR)
    return all_files()


def write_screen(path, rels, drop=()):
    """写一份 screen.py 形态的 jsonl（path 相对数据根 · verdict）。drop 里的记 drop。"""
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    drop = set(drop)
    rows = [{"path": r, "verdict": "drop" if r in drop else "keep", "reasons": [] if r not in drop else ["夹具：人声过多"]} for r in rels]
    path.write_text("".join(json.dumps(r, ensure_ascii=False) + "\n" for r in rows))
    return path
