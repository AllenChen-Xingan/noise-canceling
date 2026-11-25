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

        def sneaky(argv, env=None):
            out = orig(argv, env)
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

    def test_same_name_twice_waiter_gets_its_own_result(self):
        # 大总管裁决（第 7 轮 #2，改结构）：每次运行一个唯一编号，等第一次的拿到第一次的结果
        rid1 = O.start_job(self.cli, "pod_9", "false", "t")
        self.fake.jobs[rid1] = (1, "first\n")
        rid2 = O.start_job(self.cli, "pod_9", "true", "t")
        self.fake.jobs[rid2] = (0, "second\n")
        self.assertNotEqual(rid1, rid2)
        st = O.wait_job(self.cli, "pod_9", rid1, 10, sleep=lambda s: None)
        self.assertEqual((st["rc"], st["log"]), (1, "first\n"))

    def test_lost_start_response_reports_run_id(self):
        # 评审第 8 轮 #1：启动请求发出后断网，报错里带编号、说「结果未知」，CLI 也先打出编号
        orig = self.fake.exec

        def lost(argv, env=None):
            if "nohup" in " ".join(argv):
                orig(argv, env)  # 其实已经启动了
                raise OSError("connection reset")
            return orig(argv, env)
        self.fake.exec = lost
        with self.assertRaises(SystemExit) as cm:
            self.run_cli("job-start", "pod_9", "train", "true")
        msg = str(cm.exception)
        self.assertIn("结果未知", msg)
        rid = msg.split("运行编号 ")[1].split("。")[0]
        self.assertTrue(rid.startswith("train-"))
        self.assertIn(rid, self.fake.jobs)  # 按这个编号能查回来
        st = O.wait_job(self.cli, "pod_9", rid, 10, sleep=lambda s: None)
        self.assertEqual(st["rc"], 0)

    def test_probe_fails_on_nonzero_rc_or_empty_log(self):
        # 评审第 8 轮 #2：测速退出码非 0 或日志空，不写结果文件、非零退出，机器照删
        import rp
        orig = rp.HERE
        rp.HERE = self.tmp.name
        try:
            for n in ("probe_remote.py", "probe_urls.txt"):
                with open(os.path.join(os.path.dirname(HERE), n), "rb") as s, \
                        open(os.path.join(self.tmp.name, n), "wb") as d:
                    d.write(s.read())
            del self.fake.pods["pod_9"]  # 这个测试要真开机：去掉不在账上的假机器
            self.fake.next_job = (137, "")
            with self.assertRaises(SystemExit) as cm:
                self.run_cli("probe", "--yes", "--dc", "US-KS-2", "--no-volume", "--max-hours", "0.25")
        finally:
            rp.HERE = orig
        self.assertIn("137", str(cm.exception))
        self.assertFalse(os.path.exists(os.path.join(self.tmp.name, "probes")))
        self.assertEqual(self.fake.pods, {})  # 失败也照删

    def test_wait_keeps_polling_until_log_appears(self):
        # 评审第 10 轮 #2：日志还没出现时 job_status 不报错，等它出现后拿到结果
        rid = "late-20261125000000-abcdef"
        st = O.job_status(self.cli, "pod_9", rid)
        self.assertEqual((st["done"], st["log_seen"]), (False, False))
        polls = []

        def appear(s):
            polls.append(s)
            self.fake.jobs[rid] = (0, "ok\n")
        st = O.wait_job(self.cli, "pod_9", rid, 60, sleep=appear)
        self.assertEqual((st["rc"], st["log"]), (0, "ok\n"))
        self.assertEqual(len(polls), 1)

    def test_unreadable_log_is_an_error_even_if_rc_is_zero(self):
        # 评审第 11 轮 #1：日志在但读不出，不能当成「没日志、已完成」
        rid = "r-20261125000000-abcdef"
        self.fake.jobs[rid] = (0, "x\n")
        self.fake.unreadable_logs = {rid}
        with self.assertRaises(SystemExit):
            O.job_status(self.cli, "pod_9", rid)

    def test_status_reader_runs_in_real_python(self):
        # 改结构后的读取器：在本机真 Python 里跑三种情况——不存在、正常、目录当文件（读出错）
        import json, subprocess, sys, tempfile
        with tempfile.TemporaryDirectory() as t:
            d = os.path.join(t, "j")
            run = lambda: json.loads(subprocess.run([sys.executable, "-c", O.JOB_STATUS_PY, d, "20"],
                                                    capture_output=True, text=True).stdout)
            self.assertEqual(run(), {"rc_raw": None, "log_exists": False})
            with open(d + ".log", "w") as f:
                f.write("NOLOG\n")  # 正文恰好是 NOLOG：照样当日志（第 12 轮 #2）
            with open(d + ".rc", "w") as f:
                f.write("0\n")
            st = run()
            self.assertEqual((st["rc_raw"], st["log_exists"], st["log"]), ("0", True, "NOLOG\n"))
            with open(d + ".log", "w") as f:
                f.write("x" * 300001)  # 单行超过 256 KB：标出按字节裁过（第 13 轮 #1）
            st = run()
            self.assertTrue(st["log_bytes_cut"])
            os.remove(d + ".log")
            os.mkdir(d + ".log")  # 读出错但不是「不存在」（第 12 轮 #1）
            self.assertIn("error", run())

    def test_truncated_status_output_is_an_error(self):
        # 第 12 轮 #3：exec 输出被截断，不判完成
        rid = "t-20261125000000-abcdef"
        self.fake.jobs[rid] = (0, "x\n")
        self.fake.truncate_status = True
        with self.assertRaises(SystemExit):
            O.job_status(self.cli, "pod_9", rid)

    def test_manual_delete_needs_yes(self):
        # 评审第 10 轮 #1：down / reap 不带 --yes 只列出要删的，不删
        with self.assertRaises(SystemExit) as cm:
            self.run_cli("down", "pod_9")
        self.assertIn("--yes", str(cm.exception))
        with self.assertRaises(SystemExit):
            self.run_cli("reap")
        self.assertIn("pod_9", self.fake.pods)
        self.assertFalse([c for c in self.fake.calls if c[0] == "DELETE"])


if __name__ == "__main__":
    unittest.main()
