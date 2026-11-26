"""remote/fetch_archives.sh：边下边记账 · md5 不符立刻停 · 已有的不重下 · 盘满了不开工。
上游没发 md5 的档案按字节数判（清单第四列）。
用假的 aria2c 与假的 du 在本机跑，不连网；md5 与 sha256 用真实现。
"""

import hashlib
import os
import stat
import subprocess
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
SCRIPT = os.path.join(os.path.dirname(HERE), "remote", "fetch_archives.sh")

# 假 aria2c：记参数，把已知内容写进 -d/-o，返回成功
FAKE_ARIA2C = r"""#!/bin/sh
echo "aria2c $*" >> LOGPATH
d=.
o=out.bin
while [ $# -gt 0 ]; do
  case "$1" in
    -d) d=$2; shift 2 ;;
    -o) o=$2; shift 2 ;;
    *) shift ;;
  esac
done
printf 'payload-%s\n' "$o" > "$d/$o"
cat > /dev/null   # 真 aria2c 会读标准输入；脚本没接 /dev/null 的话会吞掉清单剩下的行
"""

# 假 du：用量由环境变量 FAKE_DU_GB 决定
FAKE_DU = r"""#!/bin/sh
printf '%s\t%s\n' "${FAKE_DU_GB:-0}" "$*"
"""


def _exe(path: str, body: str, log: str = "") -> None:
    if log:
        body = body.replace("LOGPATH", log)
    with open(path, "w") as f:
        f.write(body)
    os.chmod(path, os.stat(path).st_mode | stat.S_IEXEC)


def payload(name: str) -> bytes:
    return f"payload-{name}\n".encode()


def md5(data: bytes) -> str:
    return hashlib.md5(data).hexdigest()


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def is_hex64(s: str) -> bool:
    return len(s) == 64 and all(c in "0123456789abcdef" for c in s.lower())


class _Case(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.t = self._tmp.name
        self.bin = os.path.join(self.t, "bin")
        os.makedirs(self.bin)
        self.log = os.path.join(self.t, "aria2c.log")
        _exe(os.path.join(self.bin, "aria2c"), FAKE_ARIA2C, self.log)
        _exe(os.path.join(self.bin, "du"), FAKE_DU)
        self.dest = os.path.join(self.t, "dest")
        self.list_path = os.path.join(self.t, "list.txt")

    def tearDown(self):
        self._tmp.cleanup()

    def write_list(self, lines):
        with open(self.list_path, "w") as f:
            f.write("# 名字 md5 地址\n\n" + "\n".join(lines) + "\n")

    def good(self, name, url="http://example.invalid/x"):
        return f"{name} {md5(payload(name))} {url}"

    def unknown_md5(self, name, url="http://example.invalid/x", nbytes=None):
        """上游没发 md5：第二列写 -，第四列给字节数。"""
        if nbytes is None:
            nbytes = len(payload(name))
        return f"{name} - {url} {nbytes}"

    def run_script(self, **extra):
        e = dict(os.environ)
        e["PATH"] = self.bin + os.pathsep + e.get("PATH", "")
        e.setdefault("FAKE_DU_GB", "1")
        e.update(extra)
        return subprocess.run(["sh", SCRIPT, self.list_path, self.dest],
                              capture_output=True, text=True, env=e)

    def record(self):
        p = os.path.join(self.dest, "checksums.txt")
        if not os.path.exists(p):
            return []
        with open(p) as f:
            return [l.split() for l in f.read().splitlines() if l.strip()]

    def aria2c_calls(self):
        if not os.path.exists(self.log):
            return []
        with open(self.log) as f:
            return [l for l in f.read().splitlines() if l.strip()]


class TestFetchArchives(_Case):
    def test_syntax(self):
        r = subprocess.run(["sh", "-n", SCRIPT], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)

    def test_good_md5_records_ok(self):
        self.write_list([self.good("a.bin"), self.good("b.bin")])
        r = self.run_script()
        self.assertEqual(r.returncode, 0, r.stderr)
        rec = self.record()
        self.assertEqual([(x[0], x[4]) for x in rec], [("a.bin", "ok"), ("b.bin", "ok")])
        for name in ("a.bin", "b.bin"):
            row = [x for x in rec if x[0] == name][0]
            data = payload(name)
            self.assertEqual(row[1], md5(data), "账上记的是实算的 md5")
            self.assertEqual(row[2], sha256(data), "账上记实算的 sha256")
            self.assertEqual(int(row[3]), len(data), "账上记字节数")
            self.assertEqual(row[4], "ok")
            self.assertTrue(row[5].endswith("Z"), "账上带 UTC 时间")
            self.assertEqual(row[6], "md5", "上游给了 md5 就按 md5 判")
        summary = [l for l in r.stdout.splitlines() if l.startswith("=== ")]
        self.assertEqual(len(summary), 2)
        for line, name in zip(summary, ("a.bin", "b.bin")):
            self.assertIn(f"=== {name} bytes={len(payload(name))}", line)
            self.assertIn("seconds=", line)
            self.assertIn(f"md5={md5(payload(name))}", line)
            self.assertIn(f"want={md5(payload(name))}", line)
            self.assertIn(f"sha256={sha256(payload(name))}", line)
            self.assertIn("mode=md5", line)
        self.assertEqual(len(self.aria2c_calls()), 2)

    def test_bad_md5_stops_and_keeps_earlier_records(self):
        self.write_list([self.good("a.bin"), "b.bin " + "0" * 32 + " http://example.invalid/b"])
        r = self.run_script()
        self.assertEqual(r.returncode, 5, r.stdout + r.stderr)
        self.assertIn("STOP md5 不符 b.bin", r.stdout)
        self.assertEqual([(x[0], x[4]) for x in self.record()], [("a.bin", "ok"), ("b.bin", "BAD")])
        self.assertTrue(os.path.exists(os.path.join(self.dest, "b.bin")), "不删文件，删是人的事")

    def test_existing_complete_file_not_downloaded_again(self):
        os.makedirs(self.dest)
        with open(os.path.join(self.dest, "a.bin"), "wb") as f:
            f.write(payload("a.bin"))
        self.write_list([self.good("a.bin")])
        r = self.run_script()
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(self.aria2c_calls(), [], "已有完整文件就不再下载")
        self.assertEqual([(x[0], x[4]) for x in self.record()], [("a.bin", "ok")])

    def test_disk_over_limit_stops_before_download(self):
        self.write_list([self.good("a.bin")])
        r = self.run_script(FAKE_DU_GB="186")
        self.assertEqual(r.returncode, 3, r.stdout + r.stderr)
        self.assertIn("STOP 盘用量", r.stdout)
        self.assertEqual(self.aria2c_calls(), [], "越过停止线不开工")
        self.assertEqual(self.record(), [], "没下东西就不记账")

    def test_disk_limit_is_env_driven(self):
        self.write_list([self.good("a.bin")])
        r = self.run_script(FAKE_DU_GB="100", STOP_GB="101")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(len(self.aria2c_calls()), 1)

    # ---- 上游没发 md5 的档案：只按字节数判 ----
    def test_unknown_md5_with_correct_bytes_is_ok(self):
        self.write_list([self.unknown_md5("a.bin")])
        r = self.run_script()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        rec = self.record()
        self.assertEqual(len(rec), 1)
        name, got_md5, got_sha, nbytes, status, when, mode = rec[0]
        self.assertEqual((name, status, mode), ("a.bin", "ok", "bytes-only"))
        data = payload("a.bin")
        self.assertEqual(got_md5, md5(data), "没上游 md5 也要记实算 md5")
        self.assertEqual(got_sha, sha256(data))
        self.assertTrue(is_hex64(got_sha), "sha256 是 64 位十六进制")
        self.assertEqual(int(nbytes), len(data))
        self.assertTrue(when.endswith("Z"))
        summary = [l for l in r.stdout.splitlines() if l.startswith("=== ")]
        self.assertEqual(len(summary), 1)
        self.assertIn(f"=== a.bin bytes={len(data)}", summary[0])
        self.assertIn(f"sha256={sha256(data)}", summary[0])
        self.assertIn("mode=bytes-only", summary[0])
        self.assertIn("want=-", summary[0])

    def test_unknown_md5_with_wrong_bytes_records_bad_and_keeps_file(self):
        self.write_list([self.unknown_md5("a.bin", nbytes=len(payload("a.bin")) + 1)])
        r = self.run_script()
        self.assertEqual(r.returncode, 5, r.stdout + r.stderr)
        self.assertIn("STOP 字节数不符 a.bin", r.stdout)
        rec = self.record()
        self.assertEqual([(x[0], x[4], x[6]) for x in rec], [("a.bin", "BAD", "bytes-only")])
        self.assertEqual(rec[0][2], sha256(payload("a.bin")), "判 BAD 也要记 sha256")
        self.assertTrue(os.path.exists(os.path.join(self.dest, "a.bin")), "不删文件，删是人的事")

    def test_unknown_md5_without_bytes_column_stops_before_download(self):
        self.write_list([
            self.good("a.bin"),
            "b.bin - http://example.invalid/b",
        ])
        r = self.run_script()
        self.assertEqual(r.returncode, 2, r.stdout + r.stderr)
        self.assertIn("STOP 清单行：md5 未知时必须给字节数：", r.stderr)
        self.assertIn("b.bin - http://example.invalid/b", r.stderr)
        self.assertEqual(self.aria2c_calls(), [], "清单没通过就一个字节都不下")
        self.assertEqual(self.record(), [], "没下东西就不记账")

    def test_known_md5_with_mismatching_bytes_column_is_bad(self):
        self.write_list([
            f"a.bin {md5(payload('a.bin'))} http://example.invalid/a "
            f"{len(payload('a.bin')) + 1}",
        ])
        r = self.run_script()
        self.assertEqual(r.returncode, 5, r.stdout + r.stderr)
        self.assertIn("STOP 字节数不符 a.bin", r.stdout)
        rec = self.record()
        self.assertEqual([(x[0], x[4], x[6]) for x in rec], [("a.bin", "BAD", "md5")])
        self.assertTrue(os.path.exists(os.path.join(self.dest, "a.bin")), "不删文件，删是人的事")

    def test_known_md5_with_matching_bytes_column_is_ok(self):
        self.write_list([
            f"a.bin {md5(payload('a.bin'))} http://example.invalid/a "
            f"{len(payload('a.bin'))}",
        ])
        r = self.run_script()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertEqual([(x[0], x[4], x[6]) for x in self.record()], [("a.bin", "ok", "md5")])


if __name__ == "__main__":
    unittest.main()
