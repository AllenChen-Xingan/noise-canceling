"""冒烟命令与 ONNX 导出：判据逻辑、复现检查、导出对拍（A · B，含训后非零权重的 B）。"""
from pathlib import Path

import torch

import config as cfgmod
import export
import smoke
import train


def _cfg(tmp_path):
    return cfgmod.from_dict({
        "run": {"out_dir": str(tmp_path), "name": "s"},
        "data": {"kind": "synthetic", "seg_s": 1.0, "batch": 2, "num_workers": 0, "dev_n": 2, "dev_batches": 1},
        "device": "cpu"})


def test_reproduce_official_enh():
    r = smoke.check_reproduce(torch.device("cpu"))
    assert r["pass"] and r["max_abs_diff"] <= 2 * smoke.LSB16


def test_init_check_passes_and_overfit_reports(tmp_path):
    cfg = _cfg(tmp_path)
    batch = train.make_source(cfg).dev_batches()[0]
    init = smoke.check_init(cfg, batch, torch.device("cpu"))
    assert init["pass"] and init["rel_diff_B_vs_A"] <= 1e-5
    ov = smoke.check_overfit(cfg, batch, torch.device("cpu"), steps=3, lr=1e-3)
    assert ov["last"] < ov["first"]  # 3 步只看方向；≥ 50% 的判据由正式冒烟跑满步数时判


def test_overfit_rule_threshold():
    """判据本身：下降 49% 不过、51% 过、变成 NaN 不过（防判据写反）。"""
    assert not smoke.overfit_pass(10.0, 5.1)
    assert smoke.overfit_pass(10.0, 4.9)
    assert not smoke.overfit_pass(10.0, float("nan"))


def test_known_dev_mismatch_fails(tmp_path):
    cfg = _cfg(tmp_path)
    dev = train.make_source(cfg).dev_batches()
    good = smoke.check_known_dev(cfg, dev, torch.device("cpu"), {}, 1e-3)
    bad = smoke.check_known_dev(cfg, dev, torch.device("cpu"), {"hybrid": -999.0}, 1e-3)
    assert good["pass"] and not bad["pass"]


def test_export_official_a(tmp_path):
    m, _ = export.model_from_ckpt(None)
    out = tmp_path / "a.onnx"
    assert export.export(m, out) == ["spec"]
    assert export.compare(m, out)["pass"]


def test_export_trained_b_from_checkpoint(tmp_path):
    """训 2 步得到检查点，再把注意力与条件口子权重改成非零（模拟训后），导出对拍；cond 作为输入生效。"""
    cfg = cfgmod.from_dict({**cfgmod.to_dict(_cfg(tmp_path)),
                            "schedule": {"max_steps": 2, "ckpt_every": 2, "eval_every": 100, "log_every": 1,
                                         "keep_last": 2, }})
    train.train(cfg)
    ck = tmp_path / "s" / "ckpt" / "step-0000002.pt"
    st = torch.load(ck, weights_only=False)
    st["model"]["attn.out.weight"] = torch.randn_like(st["model"]["attn.out.weight"]) * 0.3
    st["model"]["film.fc.weight"] = torch.randn_like(st["model"]["film.fc.weight"])
    torch.save(st, ck)
    m, meta = export.model_from_ckpt(ck)
    assert meta["variant"] == "B" and meta["step"] == 2
    out = tmp_path / "b.onnx"
    assert export.export(m, out) == ["spec", "cond"]
    r = export.compare(m, out)
    assert r["pass"], r
