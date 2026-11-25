"""rebuild_exp013.py 与 exp013_inputs.py 的本机可测部分：哈希比对逻辑、版本核对、原料清单和 pools 正本一致。
真正的重建在云上跑（D97）：人工步骤见 data/README.md 第六节。"""
import json
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import exp013_inputs  # noqa: E402
import rebuild_exp013 as rb  # noqa: E402

L6323 = rb.EXP013 / "builds" / "20261124165427-6323"  # 本机用 main 现行代码出的 build


def upload_log(build=L6323):
    """仿 `sha256sum -c hashes.txt` 的输出：每个文件一行「路径: OK」。"""
    return "".join(f"{l.split('  ', 1)[1]}: OK\n" for l in (build / "hashes.txt").read_text().splitlines() if l.strip())


class Compare(unittest.TestCase):
    def write(self, d, name, rows):
        p = Path(d) / name
        p.write_text("".join(f"{h}  _tmp/exp013/{rb.BUILD}/{k}\n" for k, h in rows.items()))
        return p

    def test_same_extra_differ(self):
        with tempfile.TemporaryDirectory() as d:
            old = self.write(d, "a", {"c1/mix.wav": "1" * 64, "c1/owner.wav": "2" * 64})
            self.assertTrue(rb.compare(old, self.write(d, "b", {"c1/mix.wav": "1" * 64, "c1/owner.wav": "2" * 64, "c1/owner_early.wav": "3" * 64}))["ok"])
            r = rb.compare(old, self.write(d, "c", {"c1/mix.wav": "1" * 64, "c1/owner.wav": "f" * 64}))
            self.assertFalse(r["ok"])
            self.assertEqual(r["differ_first"], ["c1/owner.wav"])
            self.assertFalse(rb.compare(old, self.write(d, "e", {"c1/mix.wav": "1" * 64}))["ok"])

    def test_old_build_has_4540(self):
        self.assertEqual(len(rb.read_hashes(rb.OLD / "hashes.txt")), 4540)

    def test_version_diff(self):
        self.assertEqual(rb.version_diff(rb.versions()), {})
        self.assertIn("numpy", rb.version_diff(dict(rb.versions(), numpy="0.0")))

    def test_code_drift_covered_by_report(self):
        """exp013 代码和旧 build 记的不同（evalcmp 合进 early_rir 后如此）：仓里必须有一份 ok 的核对报告，且报告里新 build 用的代码就是现在的代码。"""
        if rb.code_drift():
            self.assertTrue(rb.drift_covered(), f"代码变了 {rb.code_drift()}，rebuild/ 里没有对着现行代码的 ok 报告")

    def test_drift_covered_needs_ok_report_on_current_code(self):
        """ok 的报告才算数；报告里记的代码对不上现行的不算。"""
        from unittest import mock

        with tempfile.TemporaryDirectory() as d, mock.patch.object(rb, "REPORTS", Path(d)):
            self.assertFalse(rb.drift_covered())
            r = rb.check_cloud(L6323, L6323, upload_log(), "原料全对\n")
            self.assertTrue(r["ok"], r)
            (Path(d) / "exp013-x.json").write_text(json.dumps(dict(r, ok=False)))
            self.assertFalse(rb.drift_covered())
            stale = dict(r, new_code_sha256=dict(r["new_code_sha256"], **{"rooms.py": "0" * 64}))
            (Path(d) / "exp013-y.json").write_text(json.dumps(stale))
            self.assertFalse(rb.drift_covered())
            (Path(d) / "exp013-z.json").write_text(json.dumps(r))
            self.assertTrue(rb.drift_covered())

    def test_any_build_id_prefix_stripped(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "h"
            p.write_text("ab  _tmp/exp013/20261124165427-6323/c1/mix.wav\ncd  _tmp/exp013/%s/c1/owner.wav\n" % rb.BUILD)
            self.assertEqual(rb.read_hashes(p), {"c1/mix.wav": "ab", "c1/owner.wav": "cd"})

    def test_check_cloud(self):
        """2026-11-25 大总管定的判据：跨平台做不到逐位相同（x86 对 arm64 差约 4e-13），不再要求云上重建逐位同 17f5。
        要：本机 6323 旧 4540 个逐个同 17f5 · 云上 6323 音频按 hashes.txt 全部对上 · 云上 build 的 exclude 与版本同 17f5、代码同 6323 · 原料全对。
        「官方源可复现」这次没验证（原料从本机上传），报告里照实写。"""
        ok = rb.check_cloud(L6323, L6323, upload_log(), "原料全对\n")
        self.assertTrue(ok["ok"], ok)
        self.assertEqual(ok["local_vs_old"]["same"], 4540)
        self.assertEqual(ok["upload"]["ok"], 5020)
        self.assertTrue(ok["official_source"].startswith("未验证"))
        self.assertFalse(rb.check_cloud(L6323, L6323, upload_log(), "1 处不对")["ok"])
        lines = upload_log().splitlines()
        self.assertFalse(rb.check_cloud(L6323, L6323, "\n".join(lines[1:]), "原料全对")["ok"])  # 少一个
        bad = [lines[0].replace(": OK", ": FAILED")] + lines[1:]
        r = rb.check_cloud(L6323, L6323, "\n".join(bad), "原料全对")
        self.assertFalse(r["ok"])
        self.assertEqual(len(r["upload"]["failed"]), 1)
        with tempfile.TemporaryDirectory() as d:
            c = Path(d) / "20261125022300-aa9f"
            c.mkdir()
            for f in ("manifest.json", "exclude.json"):
                (c / f).write_bytes((L6323 / f).read_bytes())
            self.assertTrue(rb.check_cloud(L6323, c, upload_log(), "原料全对")["ok"])
            m = json.loads((c / "manifest.json").read_text())
            m["versions"]["numpy"] = "0.0"
            (c / "manifest.json").write_text(json.dumps(m))
            self.assertFalse(rb.check_cloud(L6323, c, upload_log(), "原料全对")["ok"])
            m["versions"] = json.loads((L6323 / "manifest.json").read_text())["versions"]
            m["code_sha256"] = dict(m["code_sha256"], **{"rooms.py": "0" * 64})
            (c / "manifest.json").write_text(json.dumps(m))
            self.assertFalse(rb.check_cloud(L6323, c, upload_log(), "原料全对")["ok"])
            (c / "manifest.json").write_bytes((L6323 / "manifest.json").read_bytes())
            (c / "exclude.json").write_text("{}")
            self.assertFalse(rb.check_cloud(L6323, c, upload_log(), "原料全对")["ok"])

    def test_upload_log_of_cloud_built_audio_red(self):
        """自审 P1：云上自产的 aa9f 和 6323 文件名全同、2758 个哈希不同。拿 aa9f 自己的 hashes.txt 跑 sha256sum -c，
        也是 5020 行 OK——判据 2 必须认出这不是 6323 的音频（日志里的路径带的是 aa9f 的 build id）。"""
        aa9f = rb.EXP013 / "builds" / "20261125022300-aa9f"
        r = rb.check_cloud(L6323, aa9f, upload_log(aa9f), "原料全对")
        self.assertFalse(r["upload"]["all_ok"], r["upload"])
        self.assertFalse(r["ok"])
        self.assertTrue(rb.check_cloud(L6323, aa9f, upload_log(), "原料全对")["ok"])

    def test_local_must_match_old(self):
        """本机那份要是改过一个旧文件，就不算。"""
        with tempfile.TemporaryDirectory() as d:
            loc = Path(d) / L6323.name
            loc.mkdir()
            for f in ("manifest.json", "exclude.json"):
                (loc / f).write_bytes((L6323 / f).read_bytes())
            h = (L6323 / "hashes.txt").read_text().splitlines()
            old_key = next(iter(rb.read_hashes(rb.OLD / "hashes.txt")))
            h = [("0" * 64 + l[64:]) if l.endswith("/" + old_key) else l for l in h]
            (loc / "hashes.txt").write_text("\n".join(h) + "\n")
            self.assertFalse(rb.check_cloud(loc, loc, upload_log(loc), "原料全对")["ok"])


class Inputs(unittest.TestCase):
    def test_record_well_formed(self):
        rec = json.loads(exp013_inputs.OUT.read_text())
        self.assertEqual(rec["build"], rb.BUILD)
        self.assertEqual(len(rec["files"]), 1994)
        man = json.loads((rb.OLD / "manifest.json").read_text())["material_index"]
        self.assertEqual(exp013_inputs.index_of(rec["files"]), man["sha256"])
        self.assertTrue(all(len(r["sha256"]) == 64 for r in rec["files"]))

    def test_verify_against_local_if_present(self):
        pools = exp013_inputs.load_pools()
        if not pools.LIBRI.exists():
            self.skipTest("本机无 ~/vdn-data（云上由 rebuild_exp013.py 跑 --verify）")
        rec = json.loads(exp013_inputs.OUT.read_text())
        rec["files"] = rec["files"][:20]  # 抽前 20 个，单测不读满 2 GB
        self.assertEqual(exp013_inputs.verify(pools, rec), [])
        rec["files"][0] = dict(rec["files"][0], sha256="0" * 64)
        self.assertEqual(len(exp013_inputs.verify(pools, rec)), 1)


if __name__ == "__main__":
    unittest.main()
