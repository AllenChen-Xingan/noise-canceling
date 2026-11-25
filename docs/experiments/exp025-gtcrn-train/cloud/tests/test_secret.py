"""put-secret：把一个密钥送进机器的容器盘，全程不在命令参数、输出、账本、看门狗登记里出现。假接口，不连网。

判红条件（大总管批，2026-11-25）：密钥原文或它的 base64 编码出现在任何 exec 的 command、CLI 输出、账本里。
"""

import base64
import json
import os
import secrets
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))
sys.path.insert(0, HERE)

import secret_ops as S  # noqa: E402
import watchdog as W  # noqa: E402
from test_cloud import Base  # noqa: E402


def leak_forms(value: str, name: str) -> list:
    """密钥会以什么样子漏出去：原文、整行、它们的 base64（标准与 URL 安全两种，去掉补齐）。"""
    forms = [value, f"{name}={value}"]
    for raw in list(forms):
        for enc in (base64.b64encode, base64.urlsafe_b64encode):
            forms.append(enc(raw.encode()).decode().rstrip("="))
    return forms


class TestPutSecret(Base):
    def setUp(self):
        super().setUp()
        self.fake.pods["pod_9"] = {"id": "pod_9", "name": "vdn-t", "status": "RUNNING"}
        self.value = "sk-or-v1-" + secrets.token_hex(32)
        self.env_file = os.path.join(self.tmp.name, "env.local")
        with open(self.env_file, "w") as f:
            f.write(f"RUNPOD_API_KEY=x\nOPENROUTER_API_KEY={self.value}\n")

    def assert_no_leak(self, *texts):
        commands = json.dumps([c[2].get("command") if isinstance(c[2], dict) else c[2]
                               for c in self.fake.calls], ensure_ascii=False)
        with open(self.ledger, encoding="utf-8") as f:
            ledger = f.read()
        watch = json.dumps(W._load())
        for form in leak_forms(self.value, "OPENROUTER_API_KEY"):
            for where, text in (("exec command", commands), ("账本", ledger), ("看门狗登记", watch),
                                *((f"输出{i}", t) for i, t in enumerate(texts))):
                self.assertNotIn(form, text, f"密钥以 {form[:6]}… 的样子出现在{where}")

    def assert_no_frame_leak(self, e):
        """异常的每一帧（不只 secret_ops）局部变量里都没有值；测试自己的帧除外。"""
        tb = e.__traceback__
        while tb is not None:
            if not tb.tb_frame.f_code.co_filename.endswith("test_secret.py"):
                self.assertNotIn(self.value, repr(tb.tb_frame.f_locals), tb.tb_frame.f_code.co_name)
            tb = tb.tb_next

    def run_cli_fail(self, *argv):
        """跑一条会失败的 CLI，返回 (异常, stdout, stderr)；两路输出也要查（第 4 轮 #4）。"""
        import io
        from contextlib import redirect_stderr, redirect_stdout
        import rp
        out, err = io.StringIO(), io.StringIO()
        caught = None
        try:  # 不用 assertRaises：它会把异常的 traceback 清掉，帧检查就成了空检查
            with redirect_stdout(out), redirect_stderr(err):
                rp.main(["--ledger", self.ledger, *argv], client=self.cli)
        except SystemExit as e:
            caught = e
        self.assertIsNotNone(caught, "应当报错")
        self.assertIsNotNone(caught.__traceback__)
        return caught, out.getvalue(), err.getvalue()

    def test_every_failure_exit_is_clean(self):
        # 第 4 轮 #1：校验失败、畸形响应、各种结果码，每个出口的消息、两路输出、帧局部变量都不含值
        orig = self.fake.exec
        cases = [
            ("bad-remote", None, ["--remote", "/workspace/.env.local"], "[REFUSED]"),
            ("weird-response", lambda argv, env=None: ["not", "a", "dict"] if env else orig(argv, env), [], "[UNKNOWN]"),
            ("mvfail", lambda argv, env=None: dict(orig(argv, env), stdout="MVFAIL\n") if env else orig(argv, env),
             [], "[UNKNOWN]"),
            ("echo-value", lambda argv, env=None: dict(orig(argv, env), stdout=env["V"], exitCode=1)
             if env else orig(argv, env), [], "[UNKNOWN]"),
            ("value-as-exitcode", lambda argv, env=None: dict(orig(argv, env), stdout="", exitCode=env["V"])
             if env else orig(argv, env), [], "[UNKNOWN]"),  # 第 5 轮 #1
        ]
        for label, fake_exec, extra, code in cases:
            with self.subTest(label):
                self.fake.exec = fake_exec or orig
                e, out, err = self.run_cli_fail("--env-file", self.env_file, "put-secret", "pod_9",
                                                "OPENROUTER_API_KEY", *extra)
                self.assert_no_leak(str(e), out, err)
                self.assert_no_frame_leak(e)
                self.assertTrue(str(e).startswith(code), str(e)[:60])  # 第 5 轮 #2：结果码也要对
        self.fake.exec = orig

    def test_writes_file_on_container_disk_and_reports_length_only(self):
        out, err = self.run_cli("--env-file", self.env_file, "put-secret", "pod_9", "OPENROUTER_API_KEY")
        self.assertEqual(self.fake.files["/root/.env.local"], f"OPENROUTER_API_KEY={self.value}\n".encode())
        self.assertEqual(self.fake.modes["/root/.env.local"], 0o600)
        self.assertIn(f"{len(self.value)} 位", out + err)
        self.assert_no_leak(out, err)
        sent = [c[2] for c in self.fake.calls if c[0] == "POST" and c[1].endswith("/exec")
                and isinstance(c[2], dict) and c[2].get("env")]
        self.assertEqual(len(sent), 1)  # 值只走 env 字段，且只发一次

    def test_missing_key_refuses_without_sending(self):
        with open(self.env_file, "w") as f:
            f.write("RUNPOD_API_KEY=x\n")
        with self.assertRaises(SystemExit) as cm:
            self.run_cli("--env-file", self.env_file, "put-secret", "pod_9", "OPENROUTER_API_KEY")
        self.assertIn("OPENROUTER_API_KEY", str(cm.exception))
        self.assertFalse([c for c in self.fake.calls if c[1].endswith("/exec")])

    def test_refuses_network_volume_and_odd_paths(self):
        for remote in ("/workspace/.env.local", "/root/../workspace/x", "relative/.env"):
            with self.assertRaises(SystemExit):
                S.put_secret(self.cli, "pod_9", "OPENROUTER_API_KEY", self.env_file, remote=remote)
        self.assertFalse([c for c in self.fake.calls if c[1].endswith("/exec")])

    def test_failure_message_carries_no_remote_text(self):
        # secret-ops 第 1 轮 #1：机器回显值（含跨截断边界、整行 base64），报错里一律看不到
        orig = self.fake.exec
        line_b64 = base64.b64encode(f"OPENROUTER_API_KEY={self.value}".encode()).decode()

        def leaky(argv, env=None):
            if env:
                return {"stdout": "x" * 290 + env["V"], "stderr": line_b64, "exitCode": 1,
                        "timedOut": False, "truncated": False, "durationMs": 1}
            return orig(argv, env)
        self.fake.exec = leaky
        e, out, err = self.run_cli_fail("--env-file", self.env_file, "put-secret", "pod_9", "OPENROUTER_API_KEY")
        self.assert_no_leak(str(e), out, err)  # 第 6 轮 #2：本机两路输出一起查
        for text in (str(e), out, err):
            self.assertNotIn(self.value[:12], text)  # 片段也不行

    def test_exception_chain_holds_no_secret(self):
        # 第 1 轮 #3：传输层抛的异常带着值；新异常的链和格式化后的 traceback 里都不能有
        import traceback

        def boom(method, url, headers, body):
            raise OSError(f"reset while sending {body!r}")
        cli = type(self.cli)("k" * 50, transport=boom, sleep=lambda s: None)
        try:
            S.put_secret(cli, "pod_9", "OPENROUTER_API_KEY", self.env_file)
        except SystemExit as e:
            chain, x = [], e
            while x is not None:
                chain.append(x)
                x = x.__cause__ or x.__context__
            self.assertEqual(len(chain), 1)
            tb = e.__traceback__  # 第 3 轮 #1：traceback 各帧的局部变量里也不能有值
            while tb is not None:
                if tb.tb_frame.f_code.co_filename.endswith("secret_ops.py"):
                    self.assertNotIn(self.value, repr(tb.tb_frame.f_locals))
                tb = tb.tb_next
            text = "".join(traceback.format_exception(e))
            self.assert_no_leak(text, str(e))
            self.assertIn("结果未知", str(e))
        else:
            self.fail("应当报错")

    def test_directory_target_and_failed_check_keep_old_file(self):
        # 第 1 轮 #2：目标是目录，或核对没过，旧文件不动
        self.fake.files["/root/.env.local"] = b"OLD\n"
        self.fake.dirs = {"/root/.env.local"}
        with self.assertRaises(SystemExit) as cm:
            S.put_secret(self.cli, "pod_9", "OPENROUTER_API_KEY", self.env_file)
        self.assertIn("目录", str(cm.exception))
        self.fake.dirs = set()
        orig = self.fake.exec
        self.fake.exec = lambda argv, env=None: orig(argv, dict(env, W="0" * 64) if env else env)
        with self.assertRaises(SystemExit):
            S.put_secret(self.cli, "pod_9", "OPENROUTER_API_KEY", self.env_file)
        self.assertEqual(self.fake.files["/root/.env.local"], b"OLD\n")

    def test_no_completion_marker_is_unknown(self):
        # 第 3 轮 #4：替换完、回 OK 前断了：没有完成标记就按「结果未知」
        orig = self.fake.exec
        self.fake.exec = lambda argv, env=None: dict(orig(argv, env), stdout="") if env else orig(argv, env)
        with self.assertRaises(SystemExit) as cm:
            S.put_secret(self.cli, "pod_9", "OPENROUTER_API_KEY", self.env_file)
        self.assertIn("[UNKNOWN]", str(cm.exception))

    def test_timeout_is_reported_as_unknown(self):
        # 第 1 轮 #5：超时不宣称成功或已清理
        orig = self.fake.exec
        self.fake.exec = lambda argv, env=None: dict(orig(argv, env), timedOut=True) if env else orig(argv, env)
        with self.assertRaises(SystemExit) as cm:
            S.put_secret(self.cli, "pod_9", "OPENROUTER_API_KEY", self.env_file)
        self.assertIn("结果未知", str(cm.exception))

    def test_write_command_is_valid_sh_and_has_no_value(self):
        import subprocess
        r = subprocess.run(["sh", "-n", "-c", S.WRITE_SH], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn('"$V"', S.WRITE_SH)  # 只引用变量（真机上的执行留待下次开机冒烟，见 plan-eval-baseline.md）

    def test_bad_utf8_env_file_does_not_leak(self):
        # 第 2 轮 #1：坏 UTF-8 的解码异常带着原始字节，不能跟着报错出去
        with open(self.env_file, "wb") as f:
            f.write(f"OPENROUTER_API_KEY={self.value}\n".encode() + b"\xff\xfe bad\n")
        try:
            S.put_secret(self.cli, "pod_9", "OPENROUTER_API_KEY", self.env_file)
        except SystemExit as e:
            import traceback
            self.assertIsNone(e.__context__)
            self.assertIn("[ERROR]", str(e))
            self.assert_no_frame_leak(e)
            self.assert_no_leak("".join(traceback.format_exception(e)), str(e))
        else:
            self.fail("应当报错")
        self.assertFalse([c for c in self.fake.calls if c[1].endswith("/exec")])

    def test_cli_entry_with_bad_utf8_env_file_does_not_leak(self):
        # 第 6 轮 #3：不注入 client，走真 CLI 入口（load_key 先读同一个文件），只替换网络传输
        import rp
        import rp_client
        with open(self.env_file, "wb") as f:
            f.write(f"RUNPOD_API_KEY={self.value}\nOPENROUTER_API_KEY={self.value}\n".encode() + b"\xff\xfe\n")
        calls = []
        orig = rp_client.urllib_transport
        rp_client.urllib_transport = lambda *a: calls.append(a) or (200, {}, b"{}")
        caught = None
        try:
            rp.main(["--ledger", self.ledger, "--env-file", self.env_file, "put-secret", "pod_9", "OPENROUTER_API_KEY"])
        except SystemExit as e:
            caught = e
        finally:
            rp_client.urllib_transport = orig
        self.assertIsNotNone(caught)
        self.assertIsNone(caught.__context__)
        self.assert_no_frame_leak(caught)
        import traceback
        self.assert_no_leak(str(caught), "".join(traceback.format_exception(caught)))
        self.assertEqual(calls, [])

    def test_bad_key_name_refused(self):
        with self.assertRaises(SystemExit):
            S.put_secret(self.cli, "pod_9", "bad name;rm", self.env_file)


if __name__ == "__main__":
    import unittest
    unittest.main()
