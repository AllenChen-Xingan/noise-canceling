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
                S.put_secret(self.cli, "pod_9", "OPENROUTER_API_KEY", self.value, remote=remote)
        self.assertFalse([c for c in self.fake.calls if c[1].endswith("/exec")])

    def test_failure_message_is_redacted(self):
        # 机器里出错、而且错误信息把值回显出来：报错里也看不到值
        orig = self.fake.exec

        def leaky(argv, env=None):
            if env:
                return {"stdout": "", "stderr": f"boom {env.get('V')}", "exitCode": 1, "timedOut": False,
                        "truncated": False, "durationMs": 1}
            return orig(argv, env)
        self.fake.exec = leaky
        with self.assertRaises(SystemExit) as cm:
            self.run_cli("--env-file", self.env_file, "put-secret", "pod_9", "OPENROUTER_API_KEY")
        self.assert_no_leak(str(cm.exception))
        self.assertIn("<secret>", str(cm.exception))

    def test_bad_key_name_refused(self):
        with self.assertRaises(SystemExit):
            S.put_secret(self.cli, "pod_9", "bad name;rm", self.value)


if __name__ == "__main__":
    import unittest
    unittest.main()
