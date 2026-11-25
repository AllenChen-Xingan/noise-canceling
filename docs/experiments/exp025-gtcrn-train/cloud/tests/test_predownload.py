"""predownload.py 的本机可测部分：镜像展开、命令拼装、完整性命令选择、跳过已解压。不连网、不调 aria2c。"""

import io
import json
import os
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from unittest import mock

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import predownload as P  # noqa: E402


class TestPredownload(unittest.TestCase):
    def test_openslr_expands_to_three_mirrors_same_path(self):
        urls = P.mirrors("https://www.openslr.org/resources/12/dev-clean.tar.gz")
        self.assertEqual(len(urls), 3)
        self.assertTrue(all(u.endswith("/resources/12/dev-clean.tar.gz") for u in urls))
        self.assertEqual(P.mirrors("https://openslr.elda.org/resources/28/x.zip")[0],
                         "https://www.openslr.org/resources/28/x.zip")

    def test_other_hosts_unchanged(self):
        u = "https://zenodo.org/records/1/files/a.zip?download=1"
        self.assertEqual(P.mirrors(u), [u])

    def test_integrity_cmd(self):
        self.assertEqual(P.integrity_cmd("/a/x.zip", "zip"), ["unzip", "-tq", "/a/x.zip"])
        self.assertEqual(P.integrity_cmd("/a/x.tar.gz", "tar"), ["tar", "-tzf", "/a/x.tar.gz"])
        self.assertEqual(P.integrity_cmd("/a/x.tar", "tar"), ["tar", "-tf", "/a/x.tar"])

    def test_aria2_cmd_names_output_and_no_overwrite(self):
        c = P.aria2_cmd(["u1", "u2"], "/w/archives", "f.zip")
        self.assertEqual(c[0], "aria2c")
        self.assertIn("--allow-overwrite=false", c)
        self.assertEqual(c[c.index("-o") + 1], "f.zip")
        self.assertEqual(c[-2:], ["u1", "u2"])

    def test_skips_extracted_and_stops_over_disk_cap(self):
        with tempfile.TemporaryDirectory() as root:
            src = os.path.join(root, "sources.json")
            files = [{"name": "a.zip", "bytes": 5, "extract": "zip", "dest": "d", "url": "https://x/a.zip"},
                     {"name": "b.zip", "bytes": 5, "extract": "zip", "dest": "d", "url": "https://x/b.zip"}]
            with open(src, "w") as fh:
                json.dump({"groups": {"G": {"files": files}}}, fh)
            os.makedirs(os.path.join(root, "d"))
            open(os.path.join(root, "d", ".done-a.zip"), "w").close()
            out = io.StringIO()
            with mock.patch.object(P, "du_gb", return_value=200.0), redirect_stdout(out), \
                    self.assertRaises(SystemExit) as cm:
                P.main(["x", src, root, "G", "--max-gb", "120"])
            self.assertEqual(cm.exception.code, 3)
            lines = [json.loads(x) for x in out.getvalue().splitlines()]
            self.assertEqual(lines[0]["status"], "已解压，跳过")
            self.assertIn("120", lines[1]["stop"])

    def test_stops_before_a_file_would_cross_the_cap(self):
        # 评审第 1 轮 #9：下载前把本包大小算进去
        with tempfile.TemporaryDirectory() as root:
            src = os.path.join(root, "sources.json")
            f = {"name": "big.zip", "bytes": int(30e9), "extract": "zip", "dest": "d", "url": "https://x/big.zip"}
            with open(src, "w") as fh:
                json.dump({"groups": {"G": {"files": [f]}}}, fh)
            out = io.StringIO()
            with mock.patch.object(P, "du_gb", return_value=100.0), mock.patch.object(P.subprocess, "run") as run, \
                    redirect_stdout(out), self.assertRaises(SystemExit) as cm:
                P.main(["x", src, root, "G", "--max-gb", "120"])
            self.assertEqual(cm.exception.code, 3)
            run.assert_not_called()  # 没开始下

    def test_du_failure_stops(self):
        failed = mock.Mock(returncode=1, stdout="", stderr="du: cannot read")
        with mock.patch.object(P.subprocess, "run", return_value=failed), redirect_stdout(io.StringIO()), \
                self.assertRaises(SystemExit) as cm:
            P.du_gb("/nowhere")
        self.assertEqual(cm.exception.code, 6)


if __name__ == "__main__":
    unittest.main()
