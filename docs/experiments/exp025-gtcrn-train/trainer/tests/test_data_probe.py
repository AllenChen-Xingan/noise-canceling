"""过拟合慢的数据侧诊断（大总管 2026-11-25 ④–⑥）：输入与目标的对齐、目标本身、逐个关配方旋钮。只诊断。"""
import copy
import json

import numpy as np
import soundfile as sf
import torch
import yaml

import config as cfgmod
import data_probe as dp
import smoke
import train


def _cfg_file(tmp_path, batch=4):
    d = {"run": {"out_dir": str(tmp_path), "name": "s"},
         "data": {"kind": "synthetic", "seg_s": 1.0, "batch": batch, "num_workers": 0, "dev_n": batch,
                  "dev_batches": 1, "calib_batches": 1},
         "device": "cpu"}
    p = tmp_path / "c.yaml"
    p.write_text(yaml.safe_dump(d))
    return p


def _batch(tmp_path):
    cfg = cfgmod.load(str(_cfg_file(tmp_path)), {})
    return cfg, train.make_source(cfg).dev_batches()[0]


def test_xcorr_lag_detects_shift_and_zero():
    g = torch.Generator().manual_seed(0)
    c = torch.randn(2, 16000, generator=g)
    shifted = torch.zeros_like(c)
    shifted[:, 37:] = c[:, :-37]
    r = dp.alignment(shifted + 0.01 * torch.randn(2, 16000, generator=g), c)
    assert [s["lag_samples"] for s in r["segments"]] == [37, 37] and not r["all_zero_lag"]
    r0 = dp.alignment(c, c)
    assert r0["all_zero_lag"] and r0["segments"][0]["corr"] > 0.99


def test_synthetic_batch_is_aligned(tmp_path):
    _, b = _batch(tmp_path)
    assert dp.alignment(b["noisy"], b["clean"])["all_zero_lag"]


def test_target_stats_flags_silent_clipped_and_level():
    clean = torch.zeros(3, 16000)
    clean[1] = 0.05 * torch.sin(torch.arange(16000) * 0.05)
    clean[2] = 0.05 * torch.sin(torch.arange(16000) * 0.05)
    clean[2, :500] = 1.0  # 削波
    metas = [{"level_dbfs": -28.0}, {"level_dbfs": -70.0}, {"level_dbfs": -28.0}]
    recipe = {"knobs": {"level": {"lo_dbfs": -60.0, "hi_dbfs": -10.0}}}
    s = dp.target_stats(clean, clean, metas, recipe)["segments"]
    assert s[0]["near_silent"] and not s[1]["near_silent"]
    assert s[2]["clipped_samples"] == 500 and s[1]["clipped_samples"] == 0
    assert s[0]["level_in_recipe_range"] is True and s[1]["level_in_recipe_range"] is False
    assert dp.target_stats(clean, clean, None, None)["segments"][0]["level_in_recipe_range"] is None


def test_ablation_recipes_turn_off_exactly_one_knob():
    base = {"knobs": {k: {"on": True, "src": "x"} for k in ("reverb", "eq", "clip", "n_noises", "level", "snr")}}
    keep = copy.deepcopy(base)
    rs = dp.ablation_recipes(base)
    assert base == keep  # 原配方不被改
    assert list(rs) == ["baseline", "reverb_off", "eq_off", "clip_off", "n_noises_1", "all4_off"]
    for name, knob in (("reverb_off", "reverb"), ("eq_off", "eq"), ("clip_off", "clip"), ("n_noises_1", "n_noises")):
        off = [k for k, v in rs[name]["knobs"].items() if not v["on"]]
        assert off == [knob], name
    assert sorted(k for k, v in rs["all4_off"]["knobs"].items() if not v["on"]) == ["clip", "eq", "n_noises", "reverb"]
    assert all(v["on"] for v in rs["baseline"]["knobs"].values())


def test_write_wavs(tmp_path):
    _, b = _batch(tmp_path)
    files = dp.write_wavs(tmp_path / "wav", b["noisy"], b["clean"])
    assert len(files) == 8
    x, sr = sf.read(files[0]["path"])
    info = sf.info(files[0]["path"])
    assert sr == 16000 and info.subtype == "PCM_16" and len(x) == b["noisy"].shape[1] and len(files[0]["sha256"]) == 64


def test_run_ablations_with_fake_batches(tmp_path):
    cfg, b = _batch(tmp_path)
    seen = []

    def make_batch(recipe, name):
        seen.append(name)
        return b

    import overfit_probe
    out = dp.run_ablations(cfg, torch.device("cpu"), {"knobs": {k: {"on": True, "src": "x"} for k in
                                                                ("reverb", "eq", "clip", "n_noises")}},
                           make_batch, steps=1, curve_fn=overfit_probe.run_curve)
    assert seen == list(dp.ABLATIONS) and set(out) == set(dp.ABLATIONS)
    for v in out.values():
        assert {"oracle_loss", "drop_frac_of_oracle_span", "si_snr_gain_db", "steps"} <= set(v)


def test_probe_cli_has_data_checks_on_synthetic(tmp_path):
    out = tmp_path / "p.json"
    rc = smoke.main(["--config", str(_cfg_file(tmp_path)), "--overfit-probe", str(out),
                     "--probe-steps", "2", "--probe-sweep-steps", "1"])
    assert rc == 0
    r = json.loads(out.read_text())
    d = r["data_checks"]
    assert d["alignment"]["all_zero_lag"] and len(d["targets"]["segments"]) == 4 and len(d["wavs"]) == 8
    assert "skipped" in r["ablation"]  # 合成数据没有配方
    assert np.isfinite(r["oracle_loss"])
