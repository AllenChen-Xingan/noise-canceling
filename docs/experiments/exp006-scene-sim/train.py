"""exp006 第二步：用 Rust 算出的特征（scene_offline 的输出）训练细场景分类器，生成 src/scene/src/weights.rs。

- 特征只在 Rust 里实现一份（src/scene/src/features.rs），这里只读它吐出的 JSONL，不在 Python 里重写（铁律 7 唯一编码）
- 训练只用 train 组（说话人、噪音时段、种子都与 test 不重叠）；取 t ≥ 1 s 的判断（缓冲满 1 s 起就交给滞回，所以 1–2 s 的半满缓冲也要训练；第一版只用 ≥ 2 s，开机头几秒偏向「交通」）
- 模型：标准化 → 24 个 tanh 隐单元 → softmax 7 类；numpy 全批 Adam，类别按频次反比加权，固定种子
- 设备阈值：窄带蓝牙看特征 8（hf_p90），削波看特征 21（log10 平顶占比），各取 train 上平衡准确率最高的切点
- 滞回与后验平滑的数值不在这里定，由 eval.py 扫参后写回（--hyst 参数）

用法（worktree 根下）：<主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp006-scene-sim/train.py <build> [--hyst confirm,minconf,dwell,ema]
输出：src/scene/src/weights.rs；docs/experiments/exp006-scene-sim/builds/<build>/train_report.json
"""
import argparse
import json
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
CLASSES = list(json.loads((HERE / "recipes.json").read_text())["fine"])  # 顺序 = Rust NoiseClass 顺序
NF, NH = 35, 24
DEVICES = ["none", "bt_narrow", "clip"]


def load(build, split):
    truth = {}
    for line in (HERE / "builds" / build / "truth.jsonl").read_text().splitlines():
        t = json.loads(line)
        truth[t["id"]] = t
    rows = [json.loads(l) for l in (ROOT / "_tmp/20261031-exp006" / build / f"feat_{split}.jsonl").read_text().splitlines()]
    return truth, rows


def xy(truth, rows, t_min=1.0):
    X, y, dev, ids = [], [], [], []
    for r in rows:
        if r["t"] < t_min:
            continue
        t = truth[r["id"]]
        X.append(r["feat"])
        y.append(CLASSES.index(t["fine"]))
        dev.append(DEVICES.index(t["device"]))
        ids.append(r["id"])
    return np.array(X, np.float64), np.array(y), np.array(dev), ids


def train_mlp(X, y, seed=20261031, epochs=1500, lr=0.01, l2=1e-3):
    rng = np.random.default_rng(seed)
    n, d = X.shape
    C = len(CLASSES)
    W1 = rng.normal(0, 1 / np.sqrt(d), (NH, d))
    b1 = np.zeros(NH)
    W2 = rng.normal(0, 1 / np.sqrt(NH), (C, NH))
    b2 = np.zeros(C)
    cnt = np.bincount(y, minlength=C)
    w = (n / (C * np.maximum(cnt, 1)))[y]
    Y = np.eye(C)[y]
    params = [W1, b1, W2, b2]
    m = [np.zeros_like(p) for p in params]
    v = [np.zeros_like(p) for p in params]
    for ep in range(1, epochs + 1):
        H = np.tanh(X @ W1.T + b1)
        Z = H @ W2.T + b2
        Z -= Z.max(axis=1, keepdims=True)
        P = np.exp(Z)
        P /= P.sum(axis=1, keepdims=True)
        G = (P - Y) * w[:, None] / w.sum()
        gW2 = G.T @ H + l2 * W2
        gb2 = G.sum(0)
        GH = (G @ W2) * (1 - H ** 2)
        gW1 = GH.T @ X + l2 * W1
        gb1 = GH.sum(0)
        for i, g in enumerate([gW1, gb1, gW2, gb2]):
            m[i] = 0.9 * m[i] + 0.1 * g
            v[i] = 0.999 * v[i] + 0.001 * g * g
            params[i] -= lr * (m[i] / (1 - 0.9 ** ep)) / (np.sqrt(v[i] / (1 - 0.999 ** ep)) + 1e-8)
    return params


def predict(params, X):
    W1, b1, W2, b2 = params
    Z = np.tanh(X @ W1.T + b1) @ W2.T + b2
    return Z.argmax(1)


def best_threshold(x, pos, below):
    """在 x 上找切点，使「below=True 时 x<thr 判正」的平衡准确率最高。"""
    best = (0, 0.0)
    for thr in np.unique(np.round(x, 2)):
        pred = x < thr if below else x > thr
        tpr = np.mean(pred[pos]) if pos.any() else 0
        tnr = np.mean(~pred[~pos]) if (~pos).any() else 0
        ba = (tpr + tnr) / 2
        if ba > best[1]:
            best = (float(thr), float(ba))
    return best


def rs_arr(a):
    return "[" + ", ".join(f"{v:.6e}" for v in a) + "]"


def write_weights(build, mean, std, params, bt, clip, hyst):
    W1, b1, W2, b2 = params
    confirm, minconf, dwell, ema = hyst
    lines = [
        "//! 分类器权重与阈值。**由 `docs/experiments/exp006-scene-sim/train.py` 生成，别手改**；重训 = 重跑脚本 = 新提交。",
        f"//! 训练数据：exp006 build `{build}` 的 train 组；评测见该 build 的 `eval_report.json`。",
        "#![allow(clippy::excessive_precision, clippy::approx_constant)]",
        "use crate::features::NF;",
        "use vdn_types::Hysteresis;",
        "",
        f"pub const NC: usize = {len(CLASSES)};",
        f"pub const NH: usize = {NH};",
        f'pub const TRAINED_ON: &str = "exp006/{build}";',
        f"pub static MEAN: [f32; NF] = {rs_arr(mean)};",
        f"pub static STD: [f32; NF] = {rs_arr(std)};",
        "pub static W1: [[f32; NF]; NH] = [" + ", ".join(rs_arr(r) for r in W1) + "];",
        f"pub static B1: [f32; NH] = {rs_arr(b1)};",
        "pub static W2: [[f32; NH]; NC] = [" + ", ".join(rs_arr(r) for r in W2) + "];",
        f"pub static B2: [f32; NC] = {rs_arr(b2)};",
        "/// 特征 8（hf_p90）低于它判为窄带蓝牙",
        f"pub const BT_HF_P90_MAX: f32 = {bt:.3};",
        "/// 特征 21（log10 平顶占比）高于它判为削波",
        f"pub const CLIP_MIN: f32 = {clip:.3};",
        "/// 后验平滑时间常数（秒）",
        f"pub const EMA_S: f32 = {ema};",
        # 行尾放行注释：tidy R-PARAM 会把等于描述表默认值的字面量当成第二处表示；这里的值由 --hyst 产出，
        # 与默认值一致由 src/scene/tests/behavior.rs 比对（#71）
        f"pub const HYSTERESIS: Hysteresis = Hysteresis {{ confirm_s: {confirm}, min_confidence: {minconf}, min_dwell_s: {dwell} }}; // tidy: 预设值，不随默认变：exp006 train.py --hyst 产出（文件头：别手改）；与描述表默认值一致由 src/scene/tests/behavior.rs 比对",
        "",
    ]
    (ROOT / "src/scene/src/weights.rs").write_text("\n".join(lines))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("build")
    ap.add_argument("--hyst", default="2.0,0.6,5.0,1.5", help="confirm_s,min_confidence,min_dwell_s,ema_s")
    a = ap.parse_args()
    hyst = [float(v) for v in a.hyst.split(",")]
    truth, rows = load(a.build, "train")
    X, y, dev, _ = xy(truth, rows)
    mean, std = X.mean(0), X.std(0) + 1e-6
    params = train_mlp((X - mean) / std, y)
    acc_train = float(np.mean(predict(params, (X - mean) / std) == y))
    bt, bt_ba = best_threshold(X[:, 8], dev == 1, below=True)
    clip, clip_ba = best_threshold(X[:, 21], dev == 2, below=False)
    # test 组上的逐窗准确率（未平滑，仅供参考；正式结果看 eval.py 跑 Rust 的输出）
    t_truth, t_rows = load(a.build, "test")
    Xt, yt, dt, _ = xy(t_truth, t_rows)
    acc_test = float(np.mean(predict(params, (Xt - mean) / std) == yt))
    write_weights(a.build, mean, std, params, bt, clip, hyst)
    rep = {"build": a.build, "n_train_windows": int(len(y)), "train_acc_raw": round(acc_train, 4), "test_acc_raw_python": round(acc_test, 4),
           "bt_threshold": bt, "bt_balanced_acc_train": round(bt_ba, 4), "clip_threshold": clip, "clip_balanced_acc_train": round(clip_ba, 4),
           "hysteresis": dict(zip(["confirm_s", "min_confidence", "min_dwell_s", "ema_s"], hyst)),
           "class_counts_train": {c: int(np.sum(y == i)) for i, c in enumerate(CLASSES)}}
    (HERE / "builds" / a.build / "train_report.json").write_text(json.dumps(rep, indent=1, ensure_ascii=False))
    print(json.dumps(rep, ensure_ascii=False))


if __name__ == "__main__":
    main()
