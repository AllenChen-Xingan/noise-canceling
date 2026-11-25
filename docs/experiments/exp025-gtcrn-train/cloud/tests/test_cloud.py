"""cloud/ 脚本的测试：全用假 API（fake_runpod），不连网、不开机、不花钱。

跑法：python3 -m unittest discover -s docs/experiments/exp025-gtcrn-train/cloud/tests
"""

import datetime as dt
import io
import json
import os
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from unittest import mock

HERE = os.path.dirname(os.path.abspath(__file__))
CLOUD = os.path.dirname(HERE)
sys.path.insert(0, CLOUD)
sys.path.insert(0, HERE)

import config as C  # noqa: E402
import ledger as L  # noqa: E402
import rp  # noqa: E402
import rp_ops as O  # noqa: E402
import watchdog as W  # noqa: E402
from fake_runpod import FakeRunPod  # noqa: E402
from rp_client import ApiError, Client, load_key, parse_env_file  # noqa: E402

FAKE_KEY = "rpa_FAKE0123456789"


class Base(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.ledger = os.path.join(self.tmp.name, "costs.tsv")
        self.fake = FakeRunPod()
        self.cli = Client(FAKE_KEY, transport=self.fake, sleep=lambda s: None)
        # 看门狗状态写临时目录；不真起后台进程
        self._saved = (W.DEADLINES, W.PIDFILE, rp.SPAWN)
        W.DEADLINES = os.path.join(self.tmp.name, "deadlines.json")
        W.PIDFILE = os.path.join(self.tmp.name, "watchdog.pid")
        self.spawned = []
        rp.SPAWN = lambda argv, **kw: self.spawned.append(argv) or type("P", (), {"pid": 999999999})()

    def tearDown(self):
        W.DEADLINES, W.PIDFILE, rp.SPAWN = self._saved
        self.tmp.cleanup()

    def run_cli(self, *argv):
        out, err = io.StringIO(), io.StringIO()
        with redirect_stdout(out), redirect_stderr(err):
            rp.main(["--ledger", self.ledger, *argv], client=self.cli)
        return out.getvalue(), err.getvalue()


class TestClient(Base):
    def test_bearer_header_and_repr_hides_key(self):
        self.cli.get("/v2/catalog/gpus")
        self.assertEqual(self.fake.headers_seen[0]["Authorization"], f"Bearer {FAKE_KEY}")
        self.assertNotIn(FAKE_KEY, repr(self.cli))

    def test_error_raises_with_status(self):
        with self.assertRaises(ApiError) as cm:
            self.cli.get("/v2/pods/nope")
        self.assertEqual(cm.exception.status, 404)
        self.assertNotIn(FAKE_KEY, str(cm.exception))

    def test_echoed_key_is_redacted_in_errors(self):
        # 评审第 1 轮 #10：服务把密钥回显进错误详情，异常里也看不到
        cli = Client(FAKE_KEY, transport=lambda *a: (401, {}, json.dumps({"detail": f"bad key {FAKE_KEY}"}).encode()))
        with self.assertRaises(ApiError) as cm:
            cli.get("/x")
        self.assertNotIn(FAKE_KEY, str(cm.exception))
        self.assertIn("<key>", str(cm.exception))

    def test_redaction_happens_before_truncation(self):
        # 评审第 2 轮 #6：密钥跨在 300 字截断处，也不留残片
        raw = json.dumps({"detail": "x" * 280 + FAKE_KEY}).encode()
        cli = Client(FAKE_KEY, transport=lambda *a: (400, {}, raw))
        with self.assertRaises(ApiError) as cm:
            cli.get("/x")
        self.assertNotIn(FAKE_KEY[:15], str(cm.exception))

    def test_exec_not_resent_on_503(self):
        # 评审第 2 轮 #2：exec 回 503 可能已执行，不重发
        n = []
        cli = Client(FAKE_KEY, transport=lambda *a: n.append(1) or (503, {"Retry-After": "1"}, b"{}"),
                     sleep=lambda s: None)
        with self.assertRaises(ApiError):
            O.exec_cmd(cli, "pod_x", ["true"])
        self.assertEqual(len(n), 1)

    def test_retry_after_then_success(self):
        seq = [(409, {"Retry-After": "1"}, b"{}"), (200, {}, b'{"ok": 1}')]
        slept = []
        cli = Client(FAKE_KEY, transport=lambda *a: seq.pop(0), sleep=slept.append)
        self.assertEqual(cli.get("/x"), {"ok": 1})
        self.assertEqual(slept, [1.0])

    def test_env_file_parsing_and_precedence(self):
        p = os.path.join(self.tmp.name, ".env.local")
        with open(p, "w") as f:
            f.write("# c\nOTHER=1\nexport RUNPOD_API_KEY='abc'\n")
        self.assertEqual(load_key(env={}, env_file=p), "abc")
        self.assertEqual(load_key(env={"RUNPOD_API_KEY": "zz"}, env_file=p), "zz")
        self.assertEqual(parse_env_file('A="b=c"')["A"], "b=c")
        with self.assertRaises(SystemExit):
            load_key(env={}, env_file=os.path.join(self.tmp.name, "missing"))


class TestLedger(Base):
    def test_header_matches_config_and_repo_tsv(self):
        L.append_row({"action": "x"}, self.ledger)
        with open(self.ledger) as f:
            self.assertEqual(tuple(f.readline().rstrip("\n").split("\t")), C.COSTS_TSV_COLUMNS)
        repo_tsv = os.path.join(CLOUD, "costs.tsv")
        L.read_rows(repo_tsv)  # 仓里那份表头必须和正本一致，否则抛错

    def test_unknown_column_rejected(self):
        with self.assertRaises(ValueError):
            L.append_row({"bogus": 1}, self.ledger)

    def test_budget_counts_pods_volume_and_projection(self):
        t0 = dt.datetime(2026, 11, 1, tzinfo=dt.timezone.utc)
        now = t0 + dt.timedelta(days=30)
        rows = [
            {"action": "volume_create", "resource_id": "v", "usd_per_hr": f"{L.volume_hourly(150):.6f}",
             "started_utc": L.iso(t0), "ended_utc": "", "billed_usd": "", "est_usd": ""},
            {"action": "pod_delete", "resource_id": "p", "est_usd": "3.0", "billed_usd": "", "usd_per_hr": "0.5"},
            {"action": "pod_delete", "resource_id": "q", "est_usd": "9.0", "billed_usd": "2.0", "usd_per_hr": "0.5"},
        ]
        spent = L.spent_so_far(rows, now)
        self.assertAlmostEqual(spent, 10.5 + 3.0 + 2.0, places=2)  # 盘一个月 10.5 + 账单优先
        self.assertAlmostEqual(L.check_budget(rows, 0.5, 4, now=now), spent + L.volume_reserve(rows) + 0.5 * (4 + L.GRACE_H), places=2)
        self.assertAlmostEqual(L.volume_reserve(rows), L.volume_hourly(150) * C.VOLUME_RESERVE_HOURS, places=2)  # 盘没删：预留一个月
        with self.assertRaises(SystemExit):
            L.check_budget(rows, 0.5, 10, cap=20.0, now=now)  # 15.5 + 5 > 20（显式上限，不随 config 变）

    def test_deleted_volume_has_no_reserve(self):
        rows = [{"action": "volume_create", "resource_id": "v", "usd_per_hr": "0.01"},
                {"action": "volume_delete", "resource_id": "v"}]
        self.assertEqual(L.volume_reserve(rows), 0.0)

    def test_open_pod_counts_toward_budget(self):
        t0 = dt.datetime(2026, 11, 16, tzinfo=dt.timezone.utc)
        rows = [{"action": "pod_create", "resource_id": "p", "usd_per_hr": "0.5", "started_utc": L.iso(t0)}]
        self.assertAlmostEqual(L.live_pod_cost(rows, t0 + dt.timedelta(hours=2)), 1.0)

    def test_open_pods_reserve_their_full_max_hours(self):
        # 评审第 1 轮 #2：两台各开 4 小时，第二台开机时要算上第一台跑满的费用，不只算已跑的
        t0 = dt.datetime(2026, 11, 16, tzinfo=dt.timezone.utc)
        rows = [{"action": "pod_create", "resource_id": "p", "usd_per_hr": "5.0",
                 "started_utc": L.iso(t0), "note": "max_hours=4.0"}]
        now = t0 + dt.timedelta(minutes=6)
        self.assertAlmostEqual(L.open_pods_worst(rows, now), 5.0 * (4 + L.GRACE_H))
        with self.assertRaises(SystemExit):
            L.check_budget(rows, 5.0, 4, cap=40.0, now=now)  # 旧算法 0.5 + 20.4 < 40 会放行
        late = t0 + dt.timedelta(hours=6)  # 已超时还没删：按实际已跑算
        self.assertAlmostEqual(L.open_pods_worst(rows, late), 30.0)


class TestOps(Base):
    def test_datacenters_need_stock_and_volume_support(self):
        ids = [d["id"] for d in O.candidate_datacenters(self.cli)]
        self.assertEqual(ids, ["US-KS-2", "EU-RO-1"])  # 库存高的在前；无盘支持、无货的剔掉

    def test_pod_body_matches_spec_fields(self):
        with open(os.path.join(HERE, "api_fields.json")) as f:
            spec = json.load(f)
        self.assertEqual(spec["sha256_16"], C.SPEC_SHA256_16)
        b = O.pod_body("vdn-x", "US-KS-2", "vol_1", 1.5)
        self.assertLessEqual(set(b), set(spec["schemas"]["CreatePodRequest"]))
        self.assertLessEqual(set(b["gpu"]), set(spec["schemas"]["CreateGpuConfig"]))
        self.assertLessEqual(set(b["mounts"]["network"][0]), set(spec["schemas"]["NetworkMount"]))
        self.assertEqual(b["gpu"]["minVcpuCountPerGpu"], C.MIN_VCPU_PER_GPU)
        self.assertEqual(b["cloud"], "SECURE")
        self.assertEqual(b["cmd"][:2], ["sh", "-c"])
        self.assertRegex(b["cmd"][2], r"^d=\d+; s=")  # 绝对截止时刻，见 test_watchdog
        self.assertIn("VDN_SELF_DELETE_PY", b["env"])
        self.assertEqual(spec["exec_timeout_max"], C.EXEC_TIMEOUT_MAX_S)
        self.assertEqual(spec["exec_arg_max"], C.EXEC_ARG_MAX_CHARS)
        self.assertEqual(spec["exec_args_max"], C.EXEC_ARGS_MAX)
        self.assertLess(C.PUT_ARGS_PER_CALL + 4, C.EXEC_ARGS_MAX)
        self.assertLess(C.FETCH_CHUNK_BYTES * 4 // 3 + 4, C.EXEC_STDOUT_CAP)

    def test_volume_create_logs_and_refuses_duplicate(self):
        O.create_volume(self.cli, "US-KS-2", ledger_path=self.ledger)
        rows = L.read_rows(self.ledger)
        self.assertEqual(rows[0]["action"], "volume_create")
        self.assertAlmostEqual(float(rows[0]["usd_per_hr"]) * 720, 10.5, places=3)
        with self.assertRaises(SystemExit):
            O.create_volume(self.cli, "US-KS-2", ledger_path=self.ledger)

    def test_volume_ledger_failure_deletes_new_volume(self):
        # 评审第 2 轮 #3：建盘后写账失败，删掉刚建的空盘
        with mock.patch.object(L, "append_row", side_effect=OSError("disk full")), self.assertRaises(OSError):
            O.create_volume(self.cli, "US-KS-2", ledger_path=self.ledger)
        self.assertEqual(self.fake.volumes, {})

    def test_volume_delete_stops_accrual(self):
        v = O.create_volume(self.cli, C.DATA_CENTER, ledger_path=self.ledger)
        O.delete_volume(self.cli, v["id"], self.ledger)
        self.assertEqual(self.fake.volumes, {})
        rows = L.read_rows(self.ledger)
        self.assertEqual([r["action"] for r in rows], ["volume_create", "volume_delete"])
        later = L.utcnow() + dt.timedelta(days=30)
        self.assertLess(L.volume_spent(rows, later), 0.01)  # 删后不再摊

    def test_over_budget_never_posts(self):
        with self.assertRaises(SystemExit):
            O.create_pod(self.cli, "US-KS-2", None, max_hours=C.SPEND_CAP_USD / 0.5 + 1, usd_per_hr_hint=0.5, ledger_path=self.ledger)
        self.assertFalse([c for c in self.fake.calls if c[:2] == ("POST", "/v2/pods")])

    def test_pod_name_prefix_enforced(self):
        with self.assertRaises(SystemExit):
            O.create_pod(self.cli, "US-KS-2", None, 1, 0.5, name="other", ledger_path=self.ledger)

    def test_queued_pod_is_cancelled(self):
        self.fake.queue = True
        with self.assertRaises(SystemExit):
            O.create_pod(self.cli, "US-KS-2", None, 1, C.USD_PER_HR_EXPECTED, ledger_path=self.ledger)
        self.assertIn(("DELETE", "/v2/queued-pods/q_1", None), self.fake.calls)
        self.assertEqual(L.open_pods(L.read_rows(self.ledger)), {})

    def test_price_far_above_hint_deletes(self):
        self.fake.price = C.USD_PER_HR_EXPECTED * 3
        with self.assertRaises(SystemExit):
            O.create_pod(self.cli, "US-KS-2", None, 1, C.USD_PER_HR_EXPECTED, ledger_path=self.ledger)
        self.assertEqual(self.fake.pods, {})
        self.assertEqual(L.open_pods(L.read_rows(self.ledger)), {})

    def test_price_moderately_above_tolerance_deletes(self):
        # 评审第 1 轮 #3：实际价超过容差（旧代码要超 1.5 倍才删）也要删
        self.fake.price = C.USD_PER_HR_EXPECTED * (C.PRICE_TOLERANCE + 0.1)
        with self.assertRaises(SystemExit):
            O.create_pod(self.cli, "US-KS-2", None, 1, C.USD_PER_HR_EXPECTED, ledger_path=self.ledger)
        self.assertEqual(self.fake.pods, {})

    def test_actual_price_rechecks_budget(self):
        # 实际价在容差内、但按实际价算会越过上限：删机
        self.fake.price = C.USD_PER_HR_EXPECTED * 1.1
        hours = C.SPEND_CAP_USD / (C.USD_PER_HR_EXPECTED * 1.1) - 0.05  # 按预估价能过、按实际价过不了
        with self.assertRaises(SystemExit):
            O.create_pod(self.cli, "US-KS-2", None, hours, C.USD_PER_HR_EXPECTED, ledger_path=self.ledger)
        self.assertEqual(self.fake.pods, {})

    def test_create_post_is_not_retried(self):
        # 评审第 1 轮 #4：建机回 503 + Retry-After 不重发，免得多开一台
        orig = self.fake.__call__
        n = {"post": 0}

        def flaky(method, url, headers, body):
            if method == "POST" and url.endswith("/v2/pods"):
                n["post"] += 1
                return 503, {"Retry-After": "1"}, b'{"detail":"busy"}'
            return orig(method, url, headers, body)
        cli = Client("k" * 50, transport=flaky, sleep=lambda s: None)
        with self.assertRaises(ApiError):
            O.create_pod(cli, "US-KS-2", None, 1, C.USD_PER_HR_EXPECTED, ledger_path=self.ledger)
        self.assertEqual(n["post"], 1)

    def test_failure_after_create_deletes_pod(self):
        # 评审第 1 轮 #5：机器建好后写账本出错，也要先删机
        real = L.append_row
        def boom(row, path):
            if row.get("action") == "pod_create":
                raise OSError("disk full")
            return real(row, path)
        with mock.patch.object(L, "append_row", boom), self.assertRaises(OSError):
            O.create_pod(self.cli, "US-KS-2", None, 1, C.USD_PER_HR_EXPECTED, ledger_path=self.ledger)
        self.assertEqual(self.fake.pods, {})

    def test_create_then_delete_uses_delete_not_stop(self):
        p = O.create_pod(self.cli, "US-KS-2", None, 1, C.USD_PER_HR_EXPECTED, ledger_path=self.ledger)
        O.delete_pod(self.cli, p["id"], self.ledger)
        self.assertIn(("DELETE", f"/v2/pods/{p['id']}", None), self.fake.calls)
        self.assertFalse([c for c in self.fake.calls if c[1].endswith("/action")])
        rows = L.read_rows(self.ledger)
        self.assertEqual([r["action"] for r in rows], ["pod_create", "pod_delete"])
        self.assertEqual(rows[1]["started_utc"], rows[0]["started_utc"])

    def test_delete_missing_pod_still_logs(self):
        O.delete_pod(self.cli, "pod_gone", self.ledger)
        self.assertIn("已不存在", L.read_rows(self.ledger)[0]["note"])

    def test_put_and_fetch_roundtrip_large(self):
        self.fake.pods["pod_9"] = {"id": "pod_9", "name": "vdn-t", "status": "RUNNING"}
        data = os.urandom(3 * C.FETCH_CHUNK_BYTES + 123)  # 跨多批上传、多块取回
        O.put_bytes(self.cli, "pod_9", data, "/root/x/a b.bin")
        self.assertEqual(self.fake.files["/root/x/a b.bin"], data)
        self.assertEqual(O.fetch_bytes(self.cli, "pod_9", "/root/x/a b.bin"), data)

    def test_fetch_detects_corruption(self):
        self.fake.pods["pod_9"] = {"id": "pod_9", "name": "vdn-t", "status": "RUNNING"}
        O.put_bytes(self.cli, "pod_9", b"hello", "/f")
        orig = self.fake.exec

        def bad(argv):
            r = orig(argv)
            if argv[2].startswith("dd if="):
                r["stdout"] = "aGVsbG8h"  # "hello!"
            return r
        self.fake.exec = bad
        with self.assertRaises(SystemExit):
            O.fetch_bytes(self.cli, "pod_9", "/f")

    def test_job_status_and_failure(self):
        self.fake.pods["pod_9"] = {"id": "pod_9", "name": "vdn-t", "status": "RUNNING"}
        self.assertFalse(O.job_status(self.cli, "pod_9", "j")["done"])
        O.start_job(self.cli, "pod_9", "echo hi", "j")
        st = O.wait_job(self.cli, "pod_9", "j", 10, sleep=lambda s: None)
        self.assertEqual((st["done"], st["rc"]), (True, 0))
        self.fake.fail_exec = "nohup"
        with self.assertRaises(SystemExit):
            O.start_job(self.cli, "pod_9", "echo hi", "k")

    def test_wait_exec_ready_retries_409(self):
        self.fake.pods["pod_9"] = {"id": "pod_9", "name": "vdn-t", "status": "RUNNING"}
        orig, n = self.fake.route, [0]

        def starting(m, p, q, d):
            if p.endswith("/exec") and n[0] < 2:
                n[0] += 1
                return 409, {"detail": "Pod container is starting, retry shortly"}
            return orig(m, p, q, d)
        self.fake.route = starting
        slept = []
        O.wait_exec_ready(self.cli, "pod_9", sleep=slept.append)
        self.assertEqual((n[0], slept), (2, [10, 10]))

    def test_wait_running_times_out(self):
        self.fake.pods["pod_9"] = {"id": "pod_9", "name": "vdn-t", "status": "STARTING"}
        ticks = iter(range(0, 10000, 100))
        with self.assertRaises(SystemExit):
            O.wait_running(self.cli, "pod_9", timeout_s=300, sleep=lambda s: None, clock=lambda: next(ticks))


class TestCli(Base):
    def test_money_commands_need_yes(self):
        for argv in (["up", "--dc", "US-KS-2", "--no-volume"], ["probe", "--dc", "US-KS-2", "--no-volume"],
                     ["volume-create", "--dc", "US-KS-2"], ["volume-delete", "--id", "vol_1"]):
            with self.assertRaises(SystemExit):
                self.run_cli(*argv)
        self.assertFalse([c for c in self.fake.calls if c[0] in ("POST", "DELETE")])

    def test_up_mounts_volume_in_its_dc(self):
        self.run_cli("volume-create", "--dc", "EU-RO-1", "--yes")
        out, _ = self.run_cli("up", "--yes", "--max-hours", "0.5")
        body = [c[2] for c in self.fake.calls if c[:2] == ("POST", "/v2/pods")][0]
        self.assertEqual(body["dataCenterIds"], ["EU-RO-1"])
        self.assertEqual(body["mounts"]["network"][0]["path"], C.VOLUME_MOUNT)
        with self.assertRaises(SystemExit):
            self.run_cli("up", "--yes", "--dc", "US-KS-2")  # 盘在别的机房

    def test_low_vcpu_pod_is_deleted(self):
        self.fake.vcpu = 6
        with self.assertRaises(SystemExit):
            self.run_cli("up", "--yes", "--dc", "US-KS-2", "--no-volume")
        self.assertEqual(self.fake.pods, {})

    def test_probe_always_deletes(self):
        self.fake.fail_exec = "nohup"  # 测速启动失败
        with self.assertRaises(SystemExit):
            self.run_cli("probe", "--yes", "--dc", "US-KS-2", "--no-volume", "--max-hours", "0.25")
        self.assertEqual(self.fake.pods, {})
        self.assertEqual(L.open_pods(L.read_rows(self.ledger)), {})

    def test_probe_writes_result_and_deletes(self):
        orig = rp.HERE
        rp.HERE = self.tmp.name
        try:
            for n in ("probe_remote.py", "probe_urls.txt"):
                with open(os.path.join(CLOUD, n), "rb") as s, open(os.path.join(self.tmp.name, n), "wb") as d:
                    d.write(s.read())
            out, _ = self.run_cli("probe", "--yes", "--dc", "US-KS-2", "--no-volume", "--max-hours", "0.25")
        finally:
            rp.HERE = orig
        with open(out.strip()) as f:
            self.assertEqual(json.load(f)["nproc"], 16)
        self.assertEqual(self.fake.pods, {})

    def test_catalog_price_above_tolerance_refuses(self):
        self.fake.price = C.USD_PER_HR_EXPECTED * C.PRICE_TOLERANCE + 0.01
        with self.assertRaises(SystemExit):
            self.run_cli("up", "--yes", "--dc", C.DATA_CENTER, "--no-volume")
        self.assertFalse([c for c in self.fake.calls if c[:2] == ("POST", "/v2/pods")])

    def test_volume_create_defaults_to_config_dc(self):
        self.run_cli("volume-create", "--yes")
        body = [c[2] for c in self.fake.calls if c[:2] == ("POST", "/v2/network-volumes")][0]
        self.assertEqual(body["dataCenter"], C.DATA_CENTER)

    def test_cpu_pod_body_and_ledger(self):
        self.run_cli("volume-create", "--yes")
        self.run_cli("up", "--yes", "--cpu", "--max-hours", "1")
        body = [c[2] for c in self.fake.calls if c[:2] == ("POST", "/v2/pods")][0]
        self.assertNotIn("gpu", body)
        self.assertEqual(body["cpu"], {"id": C.CPU_FLAVOR_ID, "vcpuCount": C.CPU_VCPU})
        self.assertEqual(C.CPU_VCPU & (C.CPU_VCPU - 1), 0)  # openapi：必须是 2 的幂
        self.assertEqual(body["mounts"]["network"][0]["path"], C.VOLUME_MOUNT)  # 挂同一块盘
        row = L.read_rows(self.ledger)[-1]
        self.assertEqual(row["gpu"], f"{C.CPU_FLAVOR_ID} x{C.CPU_VCPU}")
        self.assertAlmostEqual(float(row["usd_per_hr"]), C.CPU_USD_PER_VCPU_HR_EXPECTED * C.CPU_VCPU, places=4)

    def test_cpu_price_above_tolerance_refuses(self):
        self.fake.cpu_price = C.CPU_USD_PER_VCPU_HR_EXPECTED * 2
        with self.assertRaises(SystemExit):
            self.run_cli("up", "--yes", "--cpu", "--dc", C.DATA_CENTER, "--no-volume")
        self.assertFalse([c for c in self.fake.calls if c[:2] == ("POST", "/v2/pods")])

    def test_reap_skips_ledger_pod_renamed_away_from_prefix(self):
        # 评审第 1 轮 #1：账上未关、云上还在但名字不是 vdn- 开头的，不删
        self.run_cli("up", "--yes", "--dc", "US-KS-2", "--no-volume")
        pid = next(iter(self.fake.pods))
        self.fake.pods[pid]["name"] = "someone-else"
        out, _ = self.run_cli("reap")
        self.assertIn(pid, self.fake.pods)
        self.assertIn("跳过", out)

    def test_job_with_empty_rc_is_not_done(self):
        # 评审第 1 轮 #11：.rc 空着或乱码不算完
        self.run_cli("up", "--yes", "--dc", "US-KS-2", "--no-volume")
        self.fake.jobs["j"] = ("", "partial\n")
        st = O.job_status(self.cli, next(iter(self.fake.pods)), "j")
        self.assertEqual((st["done"], st["rc"]), (False, None))

    def test_reap_and_spend(self):
        self.run_cli("up", "--yes", "--dc", "US-KS-2", "--no-volume")
        self.run_cli("reap")
        self.assertEqual(self.fake.pods, {})
        out, _ = self.run_cli("spend")
        self.assertEqual(json.loads(out)["open_pods"], [])


class TestDocsCrossCheck(unittest.TestCase):
    def test_readme_numbers_match_config(self):
        with open(os.path.join(CLOUD, "README.md"), encoding="utf-8") as f:
            text = f.read()
        for needle in (f"{C.SPEND_CAP_USD:g} 美元", f"{C.VOLUME_SIZE_GB} GB", C.GPU_TYPE_ID, C.IMAGE, C.DATA_CENTER,
                       f"{C.USD_PER_HR_EXPECTED} 美元",
                       f"vCPU ≥ {C.MIN_VCPU_PER_GPU}"):
            self.assertIn(needle, text)


if __name__ == "__main__":
    unittest.main()
