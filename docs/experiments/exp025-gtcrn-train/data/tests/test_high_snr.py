"""配方旋钮 high_snr：高信噪比 / 干净输入（2026-11-25 大总管交代，人同意 D107）。

用途：教模型「没有噪声时原样放过」。成绩单的 V-PUMPV 只输入干净人声时量；配方 SNR 最高 15 dB，模型几乎没见过干净输入。
- 每段先照旧抽全部随机数；旋钮开时最后再抽一次：clean_frac 的段不加噪声（meta.clean_input），
  frac 的段 SNR 改从 U(lo_db, hi_db) 抽（meta.high_snr），其余照旧
- 和 near_silent（人声为 0）正好相反：同一段两者都抽中时按 near_silent 算，clean_input 记 False
- 旋钮关（默认）：不多抽随机数，混音逐样本和没有这个旋钮时相同
"""
import copy
import json
import sys
import unittest
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE / "tests"))
import loader  # noqa: E402
import mix  # noqa: E402
from test_gpu_mix import N, plans, torch  # noqa: E402

RECIPE = json.loads((HERE / "recipe.json").read_text())


def knobs(**over):
    k = copy.deepcopy(RECIPE["knobs"])
    for name, v in over.items():
        k[name].update(v)
    return k


def without_knob():
    k = knobs()
    k.pop("high_snr")
    return k


def on(**kw):
    return knobs(high_snr={"on": True, **kw})


class Recipe(unittest.TestCase):
    def test_default_off_with_values_and_source(self):
        k = RECIPE["knobs"]["high_snr"]
        self.assertFalse(k["on"])
        self.assertIn("暂定", k["src"])
        self.assertLess(k["frac"] + k["clean_frac"], 1.0)
        self.assertGreater(k["lo_db"], RECIPE["knobs"]["snr"]["hi_db"])
        self.assertLess(k["lo_db"], k["hi_db"])

    def test_bad_values_refused(self):
        import tempfile
        for bad in ({"frac": 0.7, "clean_frac": 0.4}, {"lo_db": 40, "hi_db": 20}, {"frac": -0.1}):
            r = copy.deepcopy(RECIPE)
            r["knobs"]["high_snr"].update(bad)
            with tempfile.TemporaryDirectory() as d:
                p = Path(d) / "r.json"
                p.write_text(json.dumps(r, ensure_ascii=False))
                with self.assertRaises(ValueError, msg=repr(bad)):
                    loader.load_recipe(p)


class Off(unittest.TestCase):
    def test_off_same_as_no_knob(self):
        """旋钮关：plan 的每个字段、render 的输出、之后的随机数都和没有这个旋钮时相同。"""
        a, b = plans(knobs(), range(24)), plans(without_knob(), range(24))
        for pa, pb in zip(a, b):
            self.assertEqual(pa.snr_db, pb.snr_db)
            np.testing.assert_array_equal(pa.rel_gain, pb.rel_gain)
            na, ca, ma = mix.render(pa, N)
            nb, cb, mb = mix.render(pb, N)
            np.testing.assert_array_equal(na, nb)
            np.testing.assert_array_equal(ca, cb)
            self.assertFalse(ma["high_snr"] or ma["clean_input"])
            mb.update(high_snr=False, clean_input=False)
            self.assertEqual(ma, mb)

    def test_on_keeps_other_draws(self):
        """开着时也是最后才抽：电平、削波、噪声段数等其余参数和关着时相同。"""
        for pa, pb in zip(plans(on(), range(24)), plans(knobs(), range(24))):
            self.assertEqual((pa.level_dbfs, pa.clip_u, pa.k, pa.silent), (pb.level_dbfs, pb.clip_u, pb.k, pb.silent))


class On(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        rng = np.random.default_rng(0)
        cls.K = on(frac=0.2, clean_frac=0.1, lo_db=20.0, hi_db=40.0)
        sp = rng.standard_normal(16000) * 0.1
        ns = [(rng.standard_normal(16000) * 0.05, False, "n")]
        cls.ps = [mix.plan(np.random.default_rng([1, i]), sp, ns, None, cls.K, 16000) for i in range(6000)]

    def test_fractions_within_tolerance(self):
        m = [p.meta for p in self.ps]
        self.assertAlmostEqual(np.mean([x["clean_input"] for x in m]), 0.1, delta=0.015)
        self.assertAlmostEqual(np.mean([x["high_snr"] for x in m]), 0.2, delta=0.02)
        self.assertFalse(any(x["clean_input"] and x["high_snr"] for x in m))

    def test_snr_distribution(self):
        hi = np.array([p.snr_db for p in self.ps if p.meta["high_snr"]])
        self.assertTrue(np.all((hi >= 20) & (hi <= 40)))
        self.assertAlmostEqual(float(np.mean(hi)), 30.0, delta=0.6)
        self.assertAlmostEqual(float(np.std(hi)), 20 / np.sqrt(12), delta=0.4)
        rest = np.array([p.snr_db for p in self.ps if not (p.meta["high_snr"] or p.meta["clean_input"])])
        s = self.K["snr"]
        self.assertTrue(np.all((rest >= s["lo_db"]) & (rest <= s["hi_db"])))
        self.assertEqual([p.meta["snr_db"] for p in self.ps[:50] if p.meta["high_snr"]],
                         [round(p.snr_db, 2) for p in self.ps[:50] if p.meta["high_snr"]])

    def test_clean_input_has_no_noise(self):
        """干净段：噪声为 0。关掉混响、均衡、削波、电平随机后，noisy 与 clean 逐点相同。"""
        K = knobs(high_snr={"on": True, "clean_frac": 1.0, "frac": 0.0}, reverb={"on": False}, eq={"on": False},
                  clip={"on": False}, level={"on": False})
        for p in plans(K, range(6)):
            self.assertTrue(p.meta["clean_input"])
            self.assertIsNone(p.meta["snr_db"])
            noisy, clean, meta = mix.render(p, N)
            np.testing.assert_array_equal(noisy, clean)
            self.assertGreater(len(meta["owner_ivs"]), 0)

    def test_near_silent_wins(self):
        K = knobs(high_snr={"on": True, "clean_frac": 1.0, "frac": 0.0}, near_silent={"on": True, "frac": 1.0})
        for p in plans(K, range(4)):
            self.assertTrue(p.meta["near_silent"])
            self.assertFalse(p.meta["clean_input"])
            self.assertTrue(np.any(p.rel_gain > 0))


@unittest.skipIf(torch is None, "没装 torch")
class GpuParity(unittest.TestCase):
    def test_batch_matches_cpu(self):
        import gpu_mix
        K = on(frac=0.4, clean_frac=0.4)
        ps = plans(K, range(16))
        self.assertTrue(any(p.meta["clean_input"] for p in ps) and any(p.meta["high_snr"] for p in ps))
        dev = "cuda" if torch.cuda.is_available() else "cpu"
        out = gpu_mix.render_batch(gpu_mix.collate_plans(ps), device=dev)
        for i, p in enumerate(ps):
            noisy, clean, _ = mix.render(p, N)
            self.assertLessEqual(float(np.max(np.abs(out["noisy"][i].cpu().numpy() - noisy))), 1e-4)
            self.assertLessEqual(float(np.max(np.abs(out["clean"][i].cpu().numpy() - clean))), 1e-4)


if __name__ == "__main__":
    unittest.main()
