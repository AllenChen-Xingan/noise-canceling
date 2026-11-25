"""防忘删机两层兜底的测试：假接口、假时钟、不起真进程、不连网。"""

import datetime as dt
import os
import subprocess
import sys
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))
sys.path.insert(0, HERE)

import ledger as L  # noqa: E402
import rp  # noqa: E402
import watchdog as W  # noqa: E402
from test_cloud import Base  # noqa: E402

T0 = dt.datetime(2026, 11, 24, 12, 0, tzinfo=dt.timezone.utc)


class TestLocalWatchdog(Base):
    def test_register_deadline_includes_grace(self):
        when = W.register("pod_a", 0.5, now=T0)
        self.assertEqual(L.parse_iso(when), T0 + dt.timedelta(seconds=1800 + W.GRACE_S))

    def test_tick_deletes_only_overdue(self):
        W.register("pod_a", 0.1, now=T0)
        W.register("pod_b", 2.0, now=T0)
        deleted = []
        done = W.tick(deleted.append, now=T0 + dt.timedelta(hours=1))
        self.assertEqual((done, deleted), (["pod_a"], ["pod_a"]))
        self.assertEqual(list(W._load()), ["pod_b"])

    def test_failed_delete_is_retried_next_tick(self):
        W.register("pod_a", 0.0, now=T0)

        def boom(pid):
            raise OSError("offline")
        self.assertEqual(W.tick(boom, now=T0 + dt.timedelta(hours=1)), [])
        self.assertIn("pod_a", W._load())
        self.assertEqual(W.tick(lambda pid: None, now=T0 + dt.timedelta(hours=1)), ["pod_a"])

    def test_run_loop_exits_when_empty(self):
        W.register("pod_a", 0.0, now=T0)
        clock = iter([T0, T0 + dt.timedelta(hours=1)] * 3)
        slept = []
        W.run_loop(lambda pid: None, sleep=slept.append, now_fn=lambda: next(clock))
        self.assertEqual(W._load(), {})
        self.assertEqual(slept, [30])  # 第一轮没到点睡一次，第二轮删完退出

    def test_up_registers_and_spawns_once(self):
        self.run_cli("up", "--yes", "--dc", "US-MO-1", "--no-volume", "--max-hours", "0.5")
        pid = next(iter(W._load()))
        self.assertIn(pid, self.fake.pods)
        self.assertEqual(len(self.spawned), 1)
        self.assertEqual(self.spawned[0][-1], "watchdog")
        with open(W.PIDFILE, "w") as f:  # 模拟看门狗已在跑：不再起第二个
            f.write(str(os.getpid()))
        self.run_cli("up", "--yes", "--dc", "US-MO-1", "--no-volume", "--max-hours", "0.5")
        self.assertEqual(len(self.spawned), 1)

    def test_down_and_reap_unregister(self):
        self.run_cli("up", "--yes", "--dc", "US-MO-1", "--no-volume")
        pid = next(iter(W._load()))
        self.run_cli("down", pid)
        self.assertEqual(W._load(), {})
        self.run_cli("up", "--yes", "--dc", "US-MO-1", "--no-volume")
        self.run_cli("reap")
        self.assertEqual(W._load(), {})

    def test_watchdog_command_deletes_overdue_pod_and_logs(self):
        self.run_cli("up", "--yes", "--dc", "US-MO-1", "--no-volume", "--max-hours", "0.01")
        pid = next(iter(W._load()))
        W._save({pid: L.iso(T0)})  # 改成早已过点
        self.run_cli("watchdog")
        self.assertEqual(self.fake.pods, {})
        rows = L.read_rows(self.ledger)
        self.assertEqual(rows[-1]["action"], "pod_delete")
        self.assertIn("watchdog", rows[-1]["note"])


class TestInPodSelfDelete(unittest.TestCase):
    def test_script_targets_only_own_pod_id(self):
        s = W.SELF_DELETE_PY
        self.assertIn("os.environ.get('RUNPOD_POD_ID')", s)
        self.assertIn("/v2/pods/'+pid", s)
        self.assertNotIn("/v2/pods'", s)  # 不列机器、不删别的
        compile(s, "<self-delete>", "exec")
        W.assert_spec_safe()

    def test_cmd_sleeps_then_retries(self):
        cmd = W.pod_cmd(0.25, now=1_000_000)
        self.assertEqual(cmd[:2], ["sh", "-c"])
        self.assertTrue(cmd[2].startswith("d=1000900;"))
        self.assertIn('python3 -c "$VDN_SELF_DELETE_PY"', cmd[2])
        self.assertIn("sleep 60", cmd[2])

    def test_script_exits_2_without_platform_env(self):
        env = {k: v for k, v in os.environ.items() if k not in ("RUNPOD_POD_ID", "RUNPOD_API_KEY")}
        r = subprocess.run([sys.executable, "-I", "-c", W.SELF_DELETE_PY], env=env,
                           capture_output=True, text=True, timeout=30)
        self.assertEqual(r.returncode, 2)  # 没有平台注入的变量就不发请求

    def test_restart_only_sleeps_the_remainder(self):
        # 评审第 1 轮 #8：容器被重启时重跑同一条命令，只睡到原定截止时刻，不重新睡满
        import time
        head = W.pod_cmd(1.0, now=time.time() - 3000)[2].split("[")[0]  # 只取「算剩余秒数」那段
        r = subprocess.run(["sh", "-c", head + 'echo "$s"'], capture_output=True, text=True)
        self.assertTrue(590 <= int(r.stdout) <= 600, r.stdout)
        past = W.pod_cmd(1.0, now=time.time() - 7200)[2].split("while")[0]
        t = time.time()
        subprocess.run(["sh", "-c", past], check=True)  # 截止已过：不睡，直接进自删
        self.assertLess(time.time() - t, 5)

    def test_shell_wrapper_runs(self):
        # 用 0 秒睡眠跑一次外壳：脚本缺变量退出 2 → 进入 sleep 60；这里只核外壳语法
        r = subprocess.run(["sh", "-n", "-c", W.pod_cmd(0)[2]], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)


if __name__ == "__main__":
    unittest.main()
