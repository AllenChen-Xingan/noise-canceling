"""显卡机开机检查：网络盘空间够、装 git、核提交号、不对就停（--down-on-fail 时删机）。假接口，不连网。"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))
sys.path.insert(0, HERE)

import config as C  # noqa: E402
import ledger as L  # noqa: E402
import preflight as F  # noqa: E402
from test_cloud import Base  # noqa: E402

WANT = "7eecbbe85db862dc4666088d6d8a1593c4f39cd8"
DU_OK = (0, "40\t/workspace\n")  # du -s --block-size=1G 的输出形状
GIT_OK = (0, WANT + "\n")


class TestPreflight(Base):
    def setUp(self):
        super().setUp()
        self.run_cli("volume-create", "--yes")
        self.vid = next(iter(self.fake.volumes))
        self.run_cli("up", "--yes", "--dc", "US-KS-2", "--no-volume")
        self.pod = next(iter(self.fake.pods))

    def _set_jobs(self, du=DU_OK, git=GIT_OK):
        """磁盘检查的任务名是 diskcheck，git 检查的任务名是 preflight；两者可以给不同结果。"""
        self.fake.job_logs = {"diskcheck": du, "preflight": git}

    def _volume_size(self, size):
        self.fake.volumes[self.vid]["size"] = size

    def _jobs_started(self):
        return [str(c[2]) for c in self.fake.calls if c[0] == "POST" and "nohup" in str(c[2])]

    # ---- 原有：装 git、核提交号 ----

    def test_passes_when_commit_matches_and_installs_git_first(self):
        self._set_jobs()
        out, _ = self.run_cli("preflight", self.pod, "--repo", "/workspace/r", "--commit", WANT)
        self.assertIn("开机检查过", out)
        started = self._jobs_started()
        self.assertIn("apt-get install -y -qq git", started[-1])  # 先装 git
        self.assertIn("rev-parse HEAD", started[-1])

    def test_wrong_commit_stops_and_deletes_with_flag(self):
        self._set_jobs(git=(0, "0" * 40 + "\n"))
        with self.assertRaises(SystemExit) as cm:
            self.run_cli("preflight", self.pod, "--repo", "/workspace/r", "--commit", WANT, "--down-on-fail")
        self.assertIn("[WRONG_COMMIT]", str(cm.exception))
        self.assertEqual(self.fake.pods, {})  # 已删机
        self.assertEqual(L.read_rows(self.ledger)[-1]["action"], "pod_delete")

    def test_no_git_fails_without_deleting_by_default(self):
        self._set_jobs(git=(127, "sh: 1: git: not found\n"))
        with self.assertRaises(SystemExit) as cm:
            self.run_cli("preflight", self.pod, "--repo", "/workspace/r", "--commit", WANT)
        self.assertIn("[NO_GIT]", str(cm.exception))
        self.assertIn(self.pod, self.fake.pods)  # 没带 --down-on-fail：不删

    def test_short_commit_refused_before_anything_runs(self):
        with self.assertRaises(SystemExit) as cm:
            F.check_commit(self.cli, self.pod, "/workspace/r", "7eecbbe8")
        self.assertIn("[REFUSED]", str(cm.exception))
        self.assertFalse([c for c in self.fake.calls if "nohup" in str(c[2])])

    # ---- 新增：网络盘空间（少于 MIN_FREE_GB 不开训练）----

    def test_free_space_passes_and_prints_numbers(self):
        self._volume_size(200)
        self._set_jobs(du=(0, "40\t/workspace\n"))
        out, _ = self.run_cli("preflight", self.pod, "--skip-git")
        self.assertIn("已用 40 GB", out)
        self.assertIn("配额 200 GB", out)
        self.assertIn("剩 160 GB", out)

    def test_low_disk_stops_without_deleting_by_default(self):
        self._volume_size(150)
        self._set_jobs(du=(0, "130\t/workspace\n"))
        with self.assertRaises(SystemExit) as cm:
            self.run_cli("preflight", self.pod, "--repo", "/workspace/r", "--commit", WANT)
        self.assertIn("[LOW_DISK]", str(cm.exception))
        self.assertIn("20 GB", str(cm.exception))  # 剩 150 - 130
        self.assertIn(self.pod, self.fake.pods)

    def test_low_disk_with_down_on_fail_deletes_pod(self):
        self._volume_size(150)
        self._set_jobs(du=(0, "130\t/workspace\n"))
        with self.assertRaises(SystemExit) as cm:
            self.run_cli("preflight", self.pod, "--skip-git", "--down-on-fail")
        self.assertIn("[LOW_DISK]", str(cm.exception))
        self.assertEqual(self.fake.pods, {})
        self.assertEqual(L.read_rows(self.ledger)[-1]["action"], "pod_delete")

    def test_quota_comes_from_api_not_a_constant(self):
        # 同一个已用量：200 GB 的盘还够、120 GB 的盘不够
        self._set_jobs(du=(0, "100\t/workspace\n"))
        self._volume_size(200)
        out, _ = self.run_cli("preflight", self.pod, "--skip-git")
        self.assertIn("剩 100 GB", out)
        self._volume_size(120)
        with self.assertRaises(SystemExit) as cm:
            self.run_cli("preflight", self.pod, "--skip-git")
        self.assertIn("[LOW_DISK]", str(cm.exception))

    def test_no_volume_is_refused(self):
        self._set_jobs()
        del self.fake.volumes[self.vid]
        with self.assertRaises(SystemExit) as cm:
            self.run_cli("preflight", self.pod, "--skip-git")
        self.assertIn("[NO_VOLUME]", str(cm.exception))

    def test_unusable_du_output_is_no_du(self):
        self._volume_size(200)
        for du in ((0, "du: cannot access '/workspace'\n"), (1, "40\t/workspace\n"), (0, "\n")):
            with self.subTest(du=du), self.assertRaises(SystemExit) as cm:
                self._set_jobs(du=du)
                self.run_cli("preflight", self.pod, "--skip-git")
            self.assertIn("[NO_DU]", str(cm.exception))

    def test_disk_check_runs_before_git_check(self):
        self._volume_size(150)
        self._set_jobs(du=(0, "130\t/workspace\n"), git=GIT_OK)
        with self.assertRaises(SystemExit) as cm:
            self.run_cli("preflight", self.pod, "--repo", "/workspace/r", "--commit", WANT)
        self.assertIn("[LOW_DISK]", str(cm.exception))
        self.assertFalse([s for s in self._jobs_started() if "rev-parse" in s])  # 磁盘不过：git 检查没起

    def test_repo_and_commit_required_together_unless_skip_git(self):
        self._set_jobs()
        with self.assertRaises(SystemExit) as cm:
            self.run_cli("preflight", self.pod, "--repo", "/workspace/r")
        self.assertIn("--skip-git", str(cm.exception))
        self.assertFalse(self._jobs_started())  # 参数不对：一件都没跑


if __name__ == "__main__":
    import unittest
    unittest.main()
