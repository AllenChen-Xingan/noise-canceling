"""训练损失的读数与 Rust verifier 正本对拍（027 3.2 交叉校验第 1 条 · CLAUDE.md 五.5）。

对拍的是**参与反传的那份代码**（`losses.splvl_db` / `losses.onset_excess_db`），不另写不可导版本。
Rust 侧：`parity_rs`（只读调用 vdn-eval 的 `verifiers::eval` 与 `analysis::masks / truth_from_clean`）。
先编：`cargo build --release --manifest-path parity_rs/Cargo.toml`（没编时本文件的测试红，不跳过）。
"""
import json
import subprocess
from pathlib import Path

import numpy as np
import pytest
import torch

import losses

HERE = Path(__file__).resolve().parents[1]
BIN = HERE / "parity_rs" / "target" / "release" / "exp025-parity"
N = 16_000 * 4


def _speechlike(rng, n, n_bursts, min_gap=0.15):
    """一串「有声段」：带包络的噪声爆发 + 停顿；返回 (clean, 区间)。确定性（按 rng）。"""
    x = np.zeros(n, np.float32)
    ivs, t = [], int(0.2 * 16000)
    for _ in range(n_bursts):
        dur = int(rng.uniform(0.25, 0.6) * 16000)
        if t + dur > n:
            break
        env = np.sin(np.linspace(0, np.pi, dur)) ** 0.5
        x[t:t + dur] = (rng.standard_normal(dur) * env * rng.uniform(0.05, 0.3)).astype(np.float32)
        ivs.append((t, t + dur))
        t += dur + int(rng.uniform(min_gap, 0.5) * 16000)
    return x, ivs


def _cases():
    rng = np.random.default_rng(20261116)
    cases = []
    c, iv = _speechlike(rng, N, 6)
    cases.append(("scaled_half", c, c * 0.5, iv, []))
    c, iv = _speechlike(rng, N, 6)
    o = c.copy()
    for a, _ in iv:  # 每个有声段开头 50 ms 压 12 dB，模拟吞字头
        o[a:a + 800] *= 0.25
    o += rng.standard_normal(N).astype(np.float32) * 0.003
    cases.append(("onset_eaten_noisy", c, o, iv, []))
    c, iv = _speechlike(rng, N, 6)
    cases.append(("crushed_floor", c, c * 1e-4, iv, []))  # 触发 pc·1e-4 下限与 40 dB 封顶
    c, iv = _speechlike(rng, N, 6)
    cases.append(("with_interf", c, c * 0.7 + 0.01 * rng.standard_normal(N).astype(np.float32),
                  iv, [(iv[1][0] + 333, iv[2][1] - 77)]))
    c, iv = _speechlike(rng, N, 2)
    cases.append(("two_onsets_only", c, c * 0.8, iv, []))  # V-ONSET 待测
    c = np.zeros(N, np.float32)
    c[4000:4000 + 3000] = rng.standard_normal(3000).astype(np.float32) * 0.1
    cases.append(("short_speech", c, c * 0.9, [(4000, 7000)], []))  # V-SPLVL 不足 50 帧，待测
    c, iv = _speechlike(rng, N, 7, min_gap=0.05)  # 有的停顿 < 100 ms：不算新字头
    iv = [(a, b + 37) for a, b in iv]  # 区间端点不对齐 160
    cases.append(("short_gaps_unaligned", c, np.roll(c, 40) * 0.9, iv, []))
    # 自审补的盲区：段长不是 160 / 320 的整数倍 · 说话一直延伸到段尾 · 字头离段尾不到 50 ms · 干净轨全零
    n = N + 123
    c, iv = _speechlike(rng, n, 6)
    tail = n - 5000
    c[tail:] = (rng.standard_normal(n - tail) * 0.1).astype(np.float32)
    iv.append((tail, n))
    cases.append(("odd_len_speech_to_end", c, c * 0.6 + 0.002, iv, []))
    n = N + 7
    c, iv = _speechlike(rng, n, 6)
    late = n - 500  # 停顿 > 100 ms 后在段尾前 500 样本又开口：窗口越界，不算字头
    c[late - 4000: late] = 0
    c[late:] = (rng.standard_normal(n - late) * 0.1).astype(np.float32)
    cases.append(("onset_window_past_end", c, c * 0.7, iv + [(late, n)], []))
    cases.append(("all_zero_clean", np.zeros(N, np.float32),
                  (rng.standard_normal(N) * 0.01).astype(np.float32), [], []))
    # V-PUMPV（人声忽大忽小）：连续说话 + 2 Hz ±6 dB 增益抖动（同 probes.rs 的探针）· 只有整体增益 · 有旁人的区间
    c = (rng.standard_normal(N) * 0.1).astype(np.float32)
    g = (10 ** (6 * np.sin(2 * np.pi * 2 * np.arange(N) / 16000) / 20)).astype(np.float32)
    cases.append(("pump_2hz_6db", c, c * g + 0.002 * rng.standard_normal(N).astype(np.float32), [(0, N)], []))
    c = (rng.standard_normal(N + 77) * 0.1).astype(np.float32)
    cases.append(("pump_flat_gain", c, c * 0.3, [(500, N + 77)], [(8000, 20000)]))
    # 边界（codex pump-loss 第 1 轮 #2）：有声窗 9 / 10 个 · 一窗 8 / 10 与 7 / 10 帧有声 · 投影为 0 的窗（db10 下限）·
    # 干净轨近零的窗（pc 门槛两侧）
    c = (rng.standard_normal(N) * 0.1).astype(np.float32)
    o = c * (1 + 0.3 * rng.standard_normal(N).astype(np.float32))
    cases.append(("pump_9_windows", c, o, [(0, 9 * 1600)], []))
    cases.append(("pump_10_windows_8of10", c, o, [(0, 9 * 1600 + 8 * 160)], []))
    cases.append(("pump_10th_7of10", c, o, [(0, 9 * 1600 + 7 * 160)], []))
    oz = o.copy()
    oz[3200:4800] = 0.0                                  # 第 3 窗输出全零：投影 0 → db10 的 1e-20 下限
    cases.append(("pump_out_zero_window", c, oz, [(0, N)], []))
    cs = c.copy()
    cs[1600:3200] = 7e-9                                 # 第 2 窗干净轨能量 1600·4.9e-17 ≈ 7.8e-14 ≤ 1e-12：跳过
    cs[4800:6400] = 3e-8                                 # 第 4 窗 ≈ 1.4e-12 > 1e-12：算
    cases.append(("pump_clean_energy_gate", cs, cs * 0.5 + 1e-9, [(0, N)], []))
    return cases


@pytest.fixture(scope="module")
def rust(tmp_path_factory):
    assert BIN.exists(), f"先编对拍驱动：cargo build --release --manifest-path {HERE / 'parity_rs/Cargo.toml'}"
    d = tmp_path_factory.mktemp("parity")
    items = []
    for name, c, o, own, itf in _cases():
        c.astype("<f4").tofile(d / f"{name}.clean.f32")
        o.astype("<f4").tofile(d / f"{name}.out.f32")
        items.append({"name": name, "clean": f"{name}.clean.f32", "out": f"{name}.out.f32",
                      "owner": [list(p) for p in own], "interf": [list(p) for p in itf]})
    (d / "list.json").write_text(json.dumps(items))
    res = subprocess.run([str(BIN), str(d / "list.json")], check=True, capture_output=True, text=True)
    return {r["name"]: r for r in json.loads(res.stdout)}


CASES = _cases()
IDS = [c[0] for c in CASES]


@pytest.mark.parametrize("case", CASES, ids=IDS)
def test_masks_and_truth_match_rust(rust, case):
    name, c, _, own, itf = case
    r = rust[name]
    nf = len(c) // losses.MASK_FRAME
    assert "".join("1" if v else "0" for v in losses.paint(own, nf)) == r["owner_mask"]
    assert "".join("1" if v else "0" for v in losses.paint(itf, nf)) == r["interf_mask"]
    assert [list(p) for p in losses.truth_from_clean(c)] == r["truth_from_clean"]


def _ours(case, dtype):
    _, c, o, own, itf = case
    ct = torch.as_tensor(c, dtype=dtype)[None]
    ot = torch.as_tensor(o, dtype=dtype)[None]
    fr = torch.as_tensor(losses.owner_only_mask(own, itf, len(c)))[None]
    d, vd = losses.splvl_db(ot, ct, fr)
    e, ve, cnt = losses.onset_excess_db(ot, ct)
    return d.item(), bool(vd), e.item(), bool(ve), cnt[0]


@pytest.mark.parametrize("dtype,tol", [(torch.float64, 1e-6), (torch.float32, 2e-3)], ids=["f64", "f32"])
@pytest.mark.parametrize("case", CASES, ids=IDS)
def test_readings_match_rust(rust, case, dtype, tol):
    """V-SPLVL / V-ONSET：有效性一致；有效时读数差 ≤ tol dB（f64 是对拍精度，f32 是训练时精度）。"""
    r = rust[case[0]]
    d, vd, e, ve, cnt = _ours(case, dtype)
    assert vd == (r["splvl"]["value"] is not None)
    if vd:
        assert abs(d - r["splvl"]["value"]) <= tol
    assert ve == (r["onset"]["value"] is not None)
    if ve:
        assert abs(e - r["onset"]["value"]) <= tol
        assert cnt == r["onset"]["extra"]["onsets"]


@pytest.mark.parametrize("dtype,tol", [(torch.float64, 1e-6), (torch.float32, 2e-3)], ids=["f64", "f32"])
@pytest.mark.parametrize("case", CASES, ids=IDS)
def test_pumpv_matches_rust(rust, case, dtype, tol):
    """V-PUMPV（人声忽大忽小）：有效性一致；有效时方差差 ≤ tol dB²，窗数相同。"""
    _, c, o, own, _ = case
    r = rust[case[0]]["pumpv"]
    ct = torch.as_tensor(c, dtype=dtype)[None]
    ot = torch.as_tensor(o, dtype=dtype)[None]
    fr = torch.as_tensor(losses.owner_mask(own, len(c)))[None]
    v, ok, nw = losses.pumpv_var(ot, ct, fr)
    assert bool(ok) == (r["value"] is not None)
    if ok:
        assert abs(v.item() - r["value"]) <= tol * max(1.0, abs(r["value"]))
        assert nw[0] == r["extra"]["windows"]


def test_pumpv_cases_cover_both_branches(rust):
    pv = {k: r["pumpv"]["value"] for k, r in rust.items()}
    assert any(v is None for v in pv.values()) and pv["pump_flat_gain"] == pytest.approx(0.0, abs=1e-6)
    assert pv["pump_2hz_6db"] > 1.0  # 探针抖动明显超过 1 dB² 的 A 档线
    assert pv["pump_9_windows"] is None and pv["pump_10_windows_8of10"] is not None and pv["pump_10th_7of10"] is None
    assert rust["pump_out_zero_window"]["pumpv"]["value"] > 1000  # 下限 −200 dB 的那一窗把方差拉得很大
    assert rust["pump_clean_energy_gate"]["pumpv"]["extra"]["windows"] == 39


def test_cases_cover_both_branches(rust):
    """夹具本身要覆盖：两项都有「判了」和「待测」；字头超额有明显为正的一条（防夹具退化成全 0）。"""
    sp = [r["splvl"]["value"] for r in rust.values()]
    on = [r["onset"]["value"] for r in rust.values()]
    assert any(v is None for v in sp) and any(v is not None for v in sp)
    assert any(v is None for v in on) and any(v is not None for v in on)
    assert rust["onset_eaten_noisy"]["onset"]["value"] > 3.0
    assert abs(rust["scaled_half"]["splvl"]["value"] - 20 * np.log10(0.5)) < 1e-3
    assert abs(rust["crushed_floor"]["splvl"]["value"] + 40.0) < 1e-6
