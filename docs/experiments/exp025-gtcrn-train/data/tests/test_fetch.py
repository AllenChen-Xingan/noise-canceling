"""fetch.py：断点续传 · 校验 · 安全解压 · sources.json 字段完整。用本机 127.0.0.1 小服务器，不联网、不下真数据。"""
import hashlib
import http.server
import io
import json
import sys
import tarfile
import tempfile
import threading
import unittest
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import fetch  # noqa: E402

BLOB = bytes(range(256)) * 4000  # 1,024,000 字节，跨多个 CHUNK


class Handler(http.server.BaseHTTPRequestHandler):
    ranges = True
    status_first = []  # 依次先回这些状态码（如 [429, 429]），之后正常发
    retry_after = None
    cut_after = None  # 第一次请求只发这么多字节就断
    calls = []

    def log_message(self, *a):
        pass

    def do_HEAD(self):
        self.send_response(200)
        self.send_header("Content-Length", str(len(BLOB)))
        self.send_header("Accept-Ranges", "bytes")
        self.end_headers()

    def do_GET(self):
        cls = type(self)
        cls.calls.append(self.headers.get("Range"))
        cls.agents.append(self.headers.get("User-Agent"))
        if cls.status_first:
            code = cls.status_first.pop(0)
            self.send_response(code)
            if cls.retry_after is not None:
                self.send_header("Retry-After", str(cls.retry_after))
            self.send_header("Content-Length", "0")
            self.end_headers()
            return
        start = 0
        rng = self.headers.get("Range")
        if rng and cls.ranges:
            start = int(rng.split("=")[1].split("-")[0])
            self.send_response(206)
        else:
            self.send_response(200)
        body = BLOB[start:]
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        if cls.cut_after is not None:
            n, cls.cut_after = cls.cut_after, None
            self.wfile.write(body[:n])
            self.wfile.flush()
            self.close_connection = True
            return
        self.wfile.write(body)


class Server:
    def __init__(self, ranges=True, cut_after=None, status_first=(), retry_after=None):
        Handler.ranges, Handler.cut_after, Handler.calls, Handler.agents = ranges, cut_after, [], []
        Handler.status_first, Handler.retry_after = list(status_first), retry_after
        self.httpd = http.server.HTTPServer(("127.0.0.1", 0), Handler)
        self.url = f"http://127.0.0.1:{self.httpd.server_address[1]}/blob"
        threading.Thread(target=self.httpd.serve_forever, daemon=True).start()

    def close(self):
        self.httpd.shutdown()
        self.httpd.server_close()


class Download(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.out = Path(self.tmp.name) / "x.bin"

    def tearDown(self):
        self.tmp.cleanup()

    def test_full(self):
        s = Server()
        try:
            self.assertEqual(fetch.download(s.url, self.out, len(BLOB)), len(BLOB))
        finally:
            s.close()
        self.assertEqual(self.out.read_bytes(), BLOB)

    def test_resume_from_part(self):
        part = self.out.with_name("x.bin.part")
        part.write_bytes(BLOB[:300_000])
        s = Server(ranges=True)
        try:
            fetch.download(s.url, self.out, len(BLOB))
        finally:
            s.close()
        self.assertEqual(Handler.calls[0], "bytes=300000-")
        self.assertEqual(self.out.read_bytes(), BLOB)

    def test_server_ignores_range_restarts(self):
        self.out.with_name("x.bin.part").write_bytes(b"garbage" * 1000)
        s = Server(ranges=False)
        try:
            fetch.download(s.url, self.out, len(BLOB))
        finally:
            s.close()
        self.assertEqual(self.out.read_bytes(), BLOB)

    def test_cut_midway_then_resume(self):
        s = Server(ranges=True, cut_after=200_000)
        try:
            fetch.download(s.url, self.out, len(BLOB), retries=3)
        finally:
            s.close()
        self.assertEqual(self.out.read_bytes(), BLOB)
        self.assertGreaterEqual(len(Handler.calls), 2)

    def test_wrong_size_keeps_part(self):
        s = Server()
        try:
            with self.assertRaises(SystemExit):
                fetch.download(s.url, self.out, len(BLOB) + 5, retries=1)
        finally:
            s.close()
        self.assertFalse(self.out.exists())


class RateLimit(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.out = Path(self.tmp.name) / "x.bin"
        self.slept = []
        self.orig = fetch._sleep
        fetch._sleep = self.slept.append

    def tearDown(self):
        fetch._sleep = self.orig
        self.tmp.cleanup()

    def test_429_honors_retry_after(self):
        s = Server(status_first=[429, 429], retry_after=7)
        try:
            fetch.download(s.url, self.out, len(BLOB))
        finally:
            s.close()
        self.assertEqual(self.out.read_bytes(), BLOB)
        self.assertEqual(self.slept, [7.0, 7.0])

    def test_429_without_header_backs_off(self):
        s = Server(status_first=[429, 503])
        try:
            fetch.download(s.url, self.out, len(BLOB))
        finally:
            s.close()
        self.assertEqual(self.slept, [5.0, 10.0])

    def test_404_fails_fast(self):
        s = Server(status_first=[404])
        try:
            with self.assertRaises(SystemExit):
                fetch.download(s.url, self.out, len(BLOB))
        finally:
            s.close()
        self.assertEqual(len(Handler.calls), 1)

    def test_retry_after_capped_and_date(self):
        class E:
            headers = {"Retry-After": "99999"}
        self.assertEqual(fetch.retry_after(E(), 0), fetch.MAX_WAIT_S)
        E.headers = {"Retry-After": "Wed, 21 Oct 2015 07:28:00 GMT"}  # 过去的日期 → 退回指数退避
        self.assertEqual(fetch.retry_after(E(), 1), 10.0)

    def test_polite_only_wikimedia(self):
        fetch.polite("https://commons.wikimedia.org/wiki/Special:FilePath/x.ogg")
        fetch.polite("https://upload.wikimedia.org/a.ogg")
        fetch.polite("https://www.openslr.org/resources/12/x.tar.gz")
        fetch.polite("https://notwikimedia.org/x")
        self.assertEqual(self.slept, [fetch.POLITE_GAP_S] * 2)

    def test_user_agent_generic(self):
        s = Server()
        try:
            fetch.download(s.url, self.out, len(BLOB))
        finally:
            s.close()
        self.assertEqual(Handler.agents, [fetch.UA])
        self.assertNotRegex(fetch.UA, r"@|\.edu|\.com|chen|allen")


class Verify(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.p = Path(self.tmp.name) / "a"
        self.p.write_bytes(BLOB)
        self.md5 = hashlib.md5(BLOB).hexdigest()
        self.sha = hashlib.sha256(BLOB).hexdigest()

    def tearDown(self):
        self.tmp.cleanup()

    def test_md5_mismatch_red(self):
        with self.assertRaises(SystemExit):
            fetch.verify(self.p, {"name": "a", "url": "u", "md5": "0" * 32}, {"files": {}})

    def test_first_download_records_sha(self):
        rec = {"files": {}}
        fetch.verify(self.p, {"name": "a", "url": "u", "md5": self.md5}, rec)
        self.assertEqual(rec["files"]["u"]["sha256"], self.sha)

    def test_recorded_sha_mismatch_red(self):
        with self.assertRaises(SystemExit):
            fetch.verify(self.p, {"name": "a", "url": "u"}, {"files": {"u": {"sha256": "f" * 64}}})


class Extract(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.d = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def test_zip_slip_refused(self):
        arc = self.d / "bad.zip"
        with zipfile.ZipFile(arc, "w") as z:
            z.writestr("../evil.txt", "x")
        with self.assertRaises(SystemExit):
            fetch.extract(arc, self.d / "out", "zip")
        self.assertFalse((self.d / "evil.txt").exists())

    def test_tar_link_refused(self):
        arc = self.d / "bad.tar.gz"
        with tarfile.open(arc, "w:gz") as t:
            ti = tarfile.TarInfo("a/link")
            ti.type, ti.linkname = tarfile.SYMTYPE, "/etc/passwd"
            t.addfile(ti)
        with self.assertRaises(SystemExit):
            fetch.extract(arc, self.d / "out", "tar")

    def test_tar_ok(self):
        arc = self.d / "ok.tar.gz"
        with tarfile.open(arc, "w:gz") as t:
            ti = tarfile.TarInfo("LibriSpeech/x.txt")
            ti.size = 3
            t.addfile(ti, io.BytesIO(b"abc"))
        fetch.extract(arc, self.d / "out", "tar")
        self.assertEqual((self.d / "out/LibriSpeech/x.txt").read_bytes(), b"abc")


class Space(unittest.TestCase):
    def test_need_and_check(self):
        g = {"X": {"files": [{"name": "a.zip", "extract": "zip", "dest": "x", "bytes": 10**9}, {"name": "b.ogg", "extract": "none", "dest": "y/b.ogg", "bytes": 10**6}]}}
        with tempfile.TemporaryDirectory() as d:
            self.assertEqual(fetch.need_bytes(g, ["X"], d), int(10**9 * fetch.PEAK_FACTOR + 10**6))
            (Path(d) / "x").mkdir()
            (Path(d) / "x/.done-a.zip").write_text("s")
            self.assertEqual(fetch.need_bytes(g, ["X"], d), 10**6)
            g["X"]["files"][1]["bytes"] = 10**18
            with self.assertRaises(SystemExit):
                fetch.check_space(g, ["X"], d, config_path=Path(d) / "无")

    def fake_config(self, d, mount, gb):
        p = Path(d) / "config.py"
        p.write_text(f'VOLUME_SIZE_GB = {gb}\nVOLUME_MOUNT = "{mount}"\n')
        return p

    def test_quota_minus_du_on_volume(self):
        """挂载点 = 网络盘时，按「配额 − du 已用」判；df 很大也不算数（cloudrun 2026-11-24 实测 df 报 158 TB）。"""
        with tempfile.TemporaryDirectory() as d, tempfile.TemporaryDirectory() as vol:
            (Path(vol) / "old.bin").write_bytes(b"x" * 3_000_000)
            cfg = self.fake_config(d, vol, 0.004)  # 配额 4 MB
            g = {"X": {"files": [{"name": "a.ogg", "extract": "none", "dest": "a.ogg", "bytes": 500_000}]}}
            need, free, mode = fetch.check_space(g, ["X"], vol, config_path=cfg)
            self.assertIn("配额", mode)
            self.assertLessEqual(free, 4_000_000 - 3_000_000)
            g["X"]["files"][0]["bytes"] = 1_500_000  # 已用 3 MB + 1.5 MB > 4 MB
            with self.assertRaises(SystemExit):
                fetch.check_space(g, ["X"], vol, config_path=cfg)

    def test_other_disk_uses_df(self):
        with tempfile.TemporaryDirectory() as d, tempfile.TemporaryDirectory() as other:
            cfg = self.fake_config(d, "/no/such/mount", 0.000001)
            g = {"X": {"files": [{"name": "a.ogg", "extract": "none", "dest": "a.ogg", "bytes": 500_000}]}}
            self.assertEqual(fetch.check_space(g, ["X"], other, config_path=cfg)[2], "df")

    def test_used_bytes_counts_files(self):
        with tempfile.TemporaryDirectory() as d:
            (Path(d) / "s").mkdir()
            (Path(d) / "s/a").write_bytes(b"x" * 100_000)
            self.assertGreaterEqual(fetch.used_bytes(d), 100_000)

    def test_workspace_without_config_refused(self):
        with self.assertRaises(SystemExit):
            fetch.load_cloud_config(Path("/no/such/config.py"))

    def test_real_cloud_config_quota(self):
        """配额正本是 cloud/config.py（cloudrun 的分支，未合进本分支前这条会红）：第一批含解压峰值要放得下。"""
        cfg = fetch.load_cloud_config()
        self.assertEqual(cfg.VOLUME_MOUNT, "/workspace")
        with tempfile.TemporaryDirectory() as d:
            self.assertLess(fetch.need_bytes(fetch.load_sources(), ["A1", "A2", "A3", "A4", "A5", "E2"], d), cfg.VOLUME_SIZE_GB * 1e9)


class Head(unittest.TestCase):
    def test_head_size_match(self):
        s = Server()
        try:
            g = {"X": {"files": [{"name": "b", "url": s.url, "bytes": len(BLOB)}, {"name": "c", "url": s.url, "bytes": 5}]}}
            self.assertEqual(fetch.head(g, ["X"]), ["c"])
        finally:
            s.close()


class Sources(unittest.TestCase):
    def test_every_file_complete(self):
        for gid, g in fetch.load_sources().items():
            for k in ("name", "role", "train_ok", "license", "files"):
                self.assertIn(k, g, gid)
            self.assertIn(g["role"], ("train", "exp013"))
            for f in g["files"]:
                self.assertTrue(f["url"].startswith("https://"), f["name"])
                self.assertGreater(f["bytes"], 0)
                self.assertIn(f["extract"], ("tar", "zip", "none"))
                self.assertFalse(Path(f["dest"]).is_absolute())
                self.assertTrue(f.get("md5") or f.get("sha256") or g["role"] == "train", f"{f['name']} 没有任何校验码")

    def test_first_batch_size_matches_plan(self):
        """027 download-list 说第一批约 16.1 GB；HEAD 实测合计应在 ±10% 内。"""
        g = fetch.load_sources()
        n = sum(f["bytes"] for k in ("A1", "A2", "A3", "A4", "A5") for f in g[k]["files"])
        self.assertLess(abs(n / 1e9 - 16.1) / 16.1, 0.10, n)

    def test_nc_not_trainable(self):
        for g in fetch.load_sources().values():
            if "NC" in g["license"]:
                self.assertFalse(g["train_ok"])

    def test_refuses_on_mac(self):
        if sys.platform != "darwin":
            self.skipTest("只在本机 Mac 上验这条")
        import subprocess

        r = subprocess.run([sys.executable, str(HERE / "fetch.py"), "--root", "/tmp/x", "--only", "A3"], capture_output=True, text=True)
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("D88", r.stderr)


if __name__ == "__main__":
    unittest.main()
