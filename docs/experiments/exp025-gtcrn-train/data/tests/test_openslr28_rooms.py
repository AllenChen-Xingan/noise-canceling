"""OpenSLR 28 实测 RIR 的房间怎么认（2026-11-25 云上 split 失败的回归测试）。

夹具行逐字取自上游 RIRS_NOISES/real_rirs_isotropic_noises/rir_list（Apache-2.0，只取几行元数据，不含音频）。
上游的坑：
- RWCP type4 的 38 条，--room-id 写的是文件名（如 `RWCP_000l.wav`），不是房间名
- AIR 同一间房按位置 / 角度 / 有无假人头拆成多个 room-id（aula_carolina_1_1_90 · 1_3_0 …；lecture_0 / lecture_1）
"""
import hashlib
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import split  # noqa: E402

REAL = "train-raw/openslr28/RIRS_NOISES/real_rirs_isotropic_noises"
LINES = [
    "--rir-id 00001 --room-id RVB2014_largeroom1 RIRS_NOISES/real_rirs_isotropic_noises/RVB2014_type1_rir_largeroom1_far_angla.wav",
    "--rir-id 00002 --room-id RVB2014_largeroom1 RIRS_NOISES/real_rirs_isotropic_noises/RVB2014_type1_rir_largeroom1_far_anglb.wav",
    "--rir-id 00017 --room-id RVB2014_smallroom1 RIRS_NOISES/real_rirs_isotropic_noises/RVB2014_type1_rir_smallroom1_far_angla.wav",
    "--rir-id 00037 --room-id RWCP_circle_ane RIRS_NOISES/real_rirs_isotropic_noises/RWCP_type1_rir_circle_ane_imp000.wav",
    "--rir-id 00181 --room-id RWCP_000l.wav RIRS_NOISES/real_rirs_isotropic_noises/RWCP_type4_rir_000l.wav",
    "--rir-id 00182 --room-id RWCP_000r.wav RIRS_NOISES/real_rirs_isotropic_noises/RWCP_type4_rir_000r.wav",
]
AIR = [
    ("air_binaural_aula_carolina_1_1_90", "air_binaural_aula_carolina_1_1_90_3.wav"),
    ("air_binaural_aula_carolina_1_3_0", "air_binaural_aula_carolina_1_3_0_3.wav"),
    ("air_binaural_lecture_0", "air_binaural_lecture_0_1.wav"),
    ("air_phone_lecture", "air_phone_lecture_hhp.wav"),
]


def rule():
    r = dict(split.load_rules()["sources"]["openslr28-rir"])
    r.pop("expect", None)
    r.pop("room_lists_sha256", None)  # 夹具的 rir_list 不是上游原文件
    return r


class Rooms(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        lines = LINES + [f"--rir-id 9{i:04d} --room-id {rid} RIRS_NOISES/real_rirs_isotropic_noises/{f}" for i, (rid, f) in enumerate(AIR)]
        d = self.root / REAL
        d.mkdir(parents=True)
        (d / "rir_list").write_text("\n".join(lines) + "\n")
        self.m = split.room_map(self.root, rule())

    def tearDown(self):
        self.tmp.cleanup()

    def unit(self, name):
        return split.unit_of(f"{REAL}/{name}", rule(), self.m)

    def test_room_id_that_ends_in_wav(self):
        """云上失败原文：RWCP_type4_rir_000l.wav 不在任何 rir_list 里。它在，只是 room-id 也以 .wav 结尾。"""
        self.assertIn("RWCP_type4_rir_000l.wav", self.m)
        self.assertNotIn("RWCP_000l.wav", self.m)

    def test_rwcp_one_unit(self):
        """RWCP 的 room-id 是阵列 / 设置代码（circle · cirline · type4 的文件名），认不出房间：整库归一个单位。"""
        u = {self.unit(n) for n in ("RWCP_type1_rir_circle_ane_imp000.wav", "RWCP_type4_rir_000l.wav", "RWCP_type4_rir_000r.wav")}
        self.assertEqual(len(u), 1)

    def test_air_one_unit(self):
        """AIR 同一间房被拆成多个 room-id（位置 · 角度 · 假人头 · 手机）：整库归一个单位。"""
        self.assertEqual(len({self.unit(f) for _, f in AIR}), 1)

    def test_rvb2014_rooms_kept(self):
        """REVERB 2014 的 room-id 是房间名（每间 4 条 = 2 距离 × 2 角度），照用。"""
        a = self.unit("RVB2014_type1_rir_largeroom1_far_angla.wav")
        self.assertEqual(a, self.unit("RVB2014_type1_rir_largeroom1_far_anglb.wav"))
        self.assertNotEqual(a, self.unit("RVB2014_type1_rir_smallroom1_far_angla.wav"))
        self.assertNotEqual(a, self.unit("RWCP_type4_rir_000l.wav"))

    def test_same_basename_in_three_room_sets(self):
        """模拟 RIR 小中大三套都有 Room001/Room001-00001.wav：按文件名查会互相覆盖（600 间房并成 200 间）。按完整路径查。"""
        sim = self.root / "train-raw/openslr28/RIRS_NOISES/simulated_rirs"
        for size in ("small", "medium", "large"):
            d = sim / f"{size}room"
            d.mkdir(parents=True)
            (d / "rir_list").write_text(f"--rir-id {size}-Room001-00001 --room-id {size}-Room001 RIRS_NOISES/simulated_rirs/{size}room/Room001/Room001-00001.wav\n")
        m = split.room_map(self.root, rule())
        units = {split.unit_of(f"train-raw/openslr28/RIRS_NOISES/simulated_rirs/{z}room/Room001/Room001-00001.wav", rule(), m) for z in ("small", "medium", "large")}
        self.assertEqual(len(units), 3)

    def test_line_without_single_path_red(self):
        (self.root / REAL / "rir_list").write_text("--rir-id 1 --room-id R1 a.wav b.wav\n")
        with self.assertRaises(SystemExit):
            split.room_map(self.root, rule())

    def test_rir_list_hash_pinned(self):
        """规则是对着上游这几份 rir_list 定的：内容变了就停，重新核。"""
        r = rule()
        r["room_lists_sha256"] = {f"{REAL}/rir_list": "0" * 64}
        with self.assertRaises(SystemExit):
            split.room_map(self.root, r)
        r["room_lists_sha256"] = {f"{REAL}/rir_list": hashlib.sha256((self.root / REAL / "rir_list").read_bytes()).hexdigest()}
        split.room_map(self.root, r)

    def test_real_rules_pin_all_four_lists(self):
        pins = split.load_rules()["sources"]["openslr28-rir"]["room_lists_sha256"]
        self.assertEqual(len(pins), 4)
        self.assertTrue(all(len(v) == 64 for v in pins.values()))


if __name__ == "__main__":
    unittest.main()
