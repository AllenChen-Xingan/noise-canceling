"""显卡机开机检查：装 git、核提交号、不对就停（--down-on-fail 时删机）。假接口，不连网。"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))
sys.path.insert(0, HERE)

import ledger as L  # noqa: E402
import preflight as F  # noqa: E402
from test_cloud import Base  # noqa: E402

WANT = "7eecbbe85db862dc4666088d6d8a1593c4f39cd8"


class TestPreflight(Base):
    def setUp(self):
        super().setUp()
        self.run_cli("up", "--yes", "--dc", "US-KS-2", "--no-volume")
        self.pod = next(iter(self.fake.pods))

    def _job_log(self, rc, log):
        self.fake.next_job = (rc, log)

    def test_passes_when_commit_matches_and_installs_git_first(self):
        self._job_log(0, WANT + "\n")
        out, _ = self.run_cli("preflight", self.pod, "--repo", "/workspace/r", "--commit", WANT)
        self.assertIn("开机检查过", out)
        started = [str(c[2]) for c in self.fake.calls if c[0] == "POST" and "nohup" in str(c[2])]
        self.assertIn("apt-get install -y -qq git", started[-1])  # 先装 git
        self.assertIn("rev-parse HEAD", started[-1])

    def test_wrong_commit_stops_and_deletes_with_flag(self):
        self._job_log(0, "0" * 40 + "\n")
        with self.assertRaises(SystemExit) as cm:
            self.run_cli("preflight", self.pod, "--repo", "/workspace/r", "--commit", WANT, "--down-on-fail")
        self.assertIn("[WRONG_COMMIT]", str(cm.exception))
        self.assertEqual(self.fake.pods, {})  # 已删机
        self.assertEqual(L.read_rows(self.ledger)[-1]["action"], "pod_delete")

    def test_no_git_fails_without_deleting_by_default(self):
        self._job_log(127, "sh: 1: git: not found\n")
        with self.assertRaises(SystemExit) as cm:
            self.run_cli("preflight", self.pod, "--repo", "/workspace/r", "--commit", WANT)
        self.assertIn("[NO_GIT]", str(cm.exception))
        self.assertIn(self.pod, self.fake.pods)  # 没带 --down-on-fail：不删

    def test_short_commit_refused_before_anything_runs(self):
        with self.assertRaises(SystemExit) as cm:
            F.check_commit(self.cli, self.pod, "/workspace/r", "7eecbbe8")
        self.assertIn("[REFUSED]", str(cm.exception))
        self.assertFalse([c for c in self.fake.calls if "nohup" in str(c[2])])


if __name__ == "__main__":
    import unittest
    unittest.main()
