"""传文件与后台任务的并发、发布、语法边界（评审第 6 轮）。假接口，不连网。"""

import os
import sys
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))
sys.path.insert(0, HERE)

import rp_ops as O  # noqa: E402
from test_cloud import Base  # noqa: E402


class TestTransfer(Base):
    def setUp(self):
        super().setUp()
        self.fake.pods["pod_9"] = {"id": "pod_9", "name": "vdn-t", "status": "RUNNING"}

    def test_put_does_not_clobber_file_that_appears_during_upload(self):
        # 第 6 轮 #1：开始时没有，传的过程中别人放了同名文件：发布失败，对方的文件不动
        orig = self.fake.exec

        def sneaky(argv):
            out = orig(argv)
            if "base64 -d >>" in " ".join(argv):
                self.fake.files.setdefault("/w/f", b"theirs")
            return out
        self.fake.exec = sneaky
        with self.assertRaises(SystemExit):
            O.put_bytes(self.cli, "pod_9", b"mine", "/w/f")
        self.assertEqual(self.fake.files["/w/f"], b"theirs")

    def test_each_upload_uses_its_own_temp_file(self):
        O.put_bytes(self.cli, "pod_9", b"a", "/w/a")
        temps = [c for c in self.fake.calls if c[0] == "POST" and ": >" in str(c[2])]
        self.assertRegex(str(temps[-1][2]), r"/w/a\.part-[0-9a-f]{8}")

    def test_overwrite_backup_names_are_unique(self):
        # 第 6 轮 #2：同一秒覆盖两次，两份备份名不同
        O.put_bytes(self.cli, "pod_9", b"v1", "/w/f")
        O.put_bytes(self.cli, "pod_9", b"v2", "/w/f", overwrite=True)
        O.put_bytes(self.cli, "pod_9", b"v3", "/w/f", overwrite=True)
        pubs = [str(c[2]) for c in self.fake.calls if c[0] == "POST" and "mv -f" in str(c[2])]
        baks = [p.split(" ln ")[1].split()[1] for p in pubs]
        self.assertEqual(len(set(baks)), 2)
        self.assertEqual(self.fake.files["/w/f"], b"v3")

    def test_trailing_comment_in_script_is_safe(self):
        # 第 6 轮 #4：行尾注释不再吞掉包装的右括号
        O.start_job(self.cli, "pod_9", "echo done # completed", "c")
        wrapped = [str(c[2]) for c in self.fake.calls if c[0] == "POST" and "nohup" in str(c[2])][-1]
        self.assertNotIn("( echo done", wrapped)

    def test_job_name_claim_is_atomic(self):
        # 第 6 轮 #3：占位用 mkdir <job>.run；占不到就拒绝
        self.fake.running = {"t"}
        with self.assertRaises(SystemExit):
            O.start_job(self.cli, "pod_9", "true", "t")

    def test_same_name_twice_waiter_gets_its_own_result(self):
        # 大总管裁决（第 7 轮 #2，改结构）：每次运行一个唯一编号，等第一次的拿到第一次的结果
        rid1 = O.start_job(self.cli, "pod_9", "false", "t")
        self.fake.jobs[rid1] = (1, "first\n")
        rid2 = O.start_job(self.cli, "pod_9", "true", "t")
        self.fake.jobs[rid2] = (0, "second\n")
        self.assertNotEqual(rid1, rid2)
        st = O.wait_job(self.cli, "pod_9", rid1, 10, sleep=lambda s: None)
        self.assertEqual((st["rc"], st["log"]), (1, "first\n"))


if __name__ == "__main__":
    unittest.main()
