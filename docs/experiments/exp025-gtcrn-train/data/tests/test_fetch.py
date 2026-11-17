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
    cut_after = None  # 第一次请求只发这么多字节就断
    calls = []

    def log_message(self, *a):
        pass

    def do_GET(self):
        cls = type(self)
        cls.calls.append(self.headers.get("Range"))
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
    def __init__(self, ranges=True, cut_after=None):
        Handler.ranges, Handler.cut_after, Handler.calls = ranges, cut_after, []
        self.httpd = http.server.HTTPServer(("127.0.0.1", 0), Handler)
        self.url = f"http://127.0.0.1:{self.httpd.server_address[1]}/blob"
        threading.Thread(target=self.httpd.serve_forever, daemon=True).start()

    def close(self):
        self.httpd.shutdown()


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
