"""remote/setup_env_timed.sh：语法、步骤顺序、每步都有计时行。用假的 uv 与 setup_uv.sh 在本机跑，不连网。"""

import os
import stat
import subprocess
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
SCRIPT = os.path.join(os.path.dirname(HERE), "remote", "setup_env_timed.sh")


def _exe(path: str, body: str) -> None:
    with open(path, "w") as f:
        f.write("#!/bin/sh\n" + body)
    os.chmod(path, os.stat(path).st_mode | stat.S_IEXEC)


class TestSetupEnvTimed(unittest.TestCase):
    def test_syntax(self):
        r = subprocess.run(["sh", "-n", SCRIPT], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)

    def test_steps_run_in_order_and_each_is_timed(self):
        with tempfile.TemporaryDirectory() as t:
            repo = os.path.join(t, "repo")
            for sub in ("data", "trainer"):
                d = os.path.join(repo, "docs/experiments/exp025-gtcrn-train", sub)
                os.makedirs(d)
                open(os.path.join(d, "requirements.txt"), "w").close()
            calls = os.path.join(t, "calls.txt")
            bindir = os.path.join(t, "bin")
            os.makedirs(bindir)
            _exe(os.path.join(bindir, "uv"), f'echo "uv $*" >> {calls}\n')  # 假 uv：只记参数
            setup_uv = os.path.join(t, "setup_uv.sh")
            _exe(setup_uv, f'echo "setup_uv VDN_BIN=$VDN_BIN" >> {calls}\n')
            env = dict(os.environ, VDN_BIN=bindir, UV_CACHE_DIR=os.path.join(t, "cache"),
                       UV_PYTHON_INSTALL_DIR=os.path.join(t, "py"))
            r = subprocess.run(["sh", SCRIPT, repo, setup_uv], capture_output=True, text=True, env=env)
            self.assertEqual(r.returncode, 0, r.stderr)
            steps = [line.split()[1] for line in r.stdout.splitlines() if line.startswith("=== ")]
            self.assertEqual(steps, ["uv-install", "venv", "pip-data", "pip-both"])
            self.assertTrue(all("seconds=" in line for line in r.stdout.splitlines() if line.startswith("=== ")))
            with open(calls) as f:
                log = f.read()
            self.assertIn(f"setup_uv VDN_BIN={bindir}", log)
            self.assertIn("uv venv --python 3.12.14", log)
            self.assertNotIn(" -q", log)  # 不带 -q：要 uv 自己的「Prepared / Installed … in」两段时间
            self.assertEqual(log.count("uv pip install"), 2)


if __name__ == "__main__":
    unittest.main()
