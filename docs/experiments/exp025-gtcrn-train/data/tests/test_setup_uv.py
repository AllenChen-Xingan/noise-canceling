"""setup_uv.sh：云上装 uv（2026-11-25：rust:1.99.1-trixie 镜像里没有 pip，旧的 `pip install uv` 失败）。

不联网：假 uv 包用本地 file:// 地址。核：哈希不对就拒绝、已有 uv 不下载、默认路径是 $VDN_ROOT/bin，
cloud_steps.sh setup 把这个目录放进 PATH 再调 uv。官方 sha256 的联网核对要设 VDN_NET=1 才跑。
"""
import hashlib
import io
import os
import re
import subprocess
import tarfile
import tempfile
import unittest
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
SCRIPT = HERE / "setup_uv.sh"
STEPS = HERE / "cloud_steps.sh"
SYS_PATH = "/usr/bin:/bin:/usr/sbin:/sbin"  # 不含 homebrew 等装了真 uv 的目录


def consts():
    return dict(re.findall(r"^(UV_[A-Z0-9_]+)=(\S+)$", SCRIPT.read_text(), re.M))


def fake_tarball(path, target="x86_64-unknown-linux-gnu"):
    """仿官方包的结构：uv-<目标>/uv、uv-<目标>/uvx。"""
    with tarfile.open(path, "w:gz") as tf:
        for name in ("uv", "uvx"):
            body = b"#!/bin/sh\necho fake-" + name.encode() + b"\n"
            info = tarfile.TarInfo(f"uv-{target}/{name}")
            info.size, info.mode = len(body), 0o755
            tf.addfile(info, io.BytesIO(body))
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def fake_uv(bin_dir, log):
    """假 uv：只把参数记进 log。"""
    bin_dir.mkdir(parents=True, exist_ok=True)
    p = bin_dir / "uv"
    p.write_text(f'#!/bin/sh\necho "$@" >> "{log}"\n')
    p.chmod(0o755)


class SetupUv(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.d = Path(self.tmp.name)
        self.tgz = self.d / "uv.tar.gz"
        self.sha = fake_tarball(self.tgz)

    def tearDown(self):
        self.tmp.cleanup()

    def run_sh(self, args, **env):
        e = {"PATH": SYS_PATH, "HOME": str(self.d), **env}
        return subprocess.run(["bash", *args], env=e, capture_output=True, text=True)

    def test_wrong_sha_refused(self):
        """包的 sha256 与正本常量不同：退出码 3，目标目录里没有 uv。"""
        bin_dir = self.d / "bin"
        r = self.run_sh([str(SCRIPT)], VDN_BIN=str(bin_dir), VDN_UV_ARCH="x86_64",
                        VDN_UV_URL=self.tgz.as_uri())
        self.assertEqual(r.returncode, 3, r.stderr)
        self.assertIn("sha256 不对", r.stderr)
        self.assertFalse((bin_dir / "uv").exists())

    def test_right_sha_installs_both_binaries(self):
        bin_dir = self.d / "bin"
        cmd = f'VDN_UV_LIB=1 source "{SCRIPT}"; install_uv "{self.tgz.as_uri()}" {self.sha} "{bin_dir}"'
        r = self.run_sh(["-c", cmd])
        self.assertEqual(r.returncode, 0, r.stderr)
        for name in ("uv", "uvx"):
            p = bin_dir / name
            self.assertTrue(os.access(p, os.X_OK), name)
            self.assertEqual(subprocess.run([p], capture_output=True, text=True).stdout.strip(), f"fake-{name}")

    def test_wrong_sha_via_install_uv_leaves_no_temp(self):
        tmpdir = self.d / "t"
        tmpdir.mkdir()
        cmd = f'VDN_UV_LIB=1 source "{SCRIPT}"; install_uv "{self.tgz.as_uri()}" {"0" * 64} "{self.d / "bin"}"'
        r = self.run_sh(["-c", cmd], TMPDIR=str(tmpdir))
        self.assertEqual(r.returncode, 3)
        self.assertEqual(list(tmpdir.iterdir()), [])

    def test_existing_uv_not_downloaded(self):
        """PATH 上已有 uv：不下载（下载地址指向不存在的文件，下载就会失败）。"""
        fake_uv(self.d / "sys", self.d / "log")
        r = self.run_sh([str(SCRIPT)], PATH=f"{self.d / 'sys'}:{SYS_PATH}", VDN_BIN=str(self.d / "bin"),
                        VDN_UV_URL=(self.d / "missing.tar.gz").as_uri())
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("已有 uv", r.stdout)

    def test_default_bin_is_root_bin(self):
        """没设 VDN_BIN 时，uv 放在 $VDN_ROOT/bin（云上即 /workspace/bin）。"""
        fake_uv(self.d / "bin", self.d / "log")
        r = self.run_sh([str(SCRIPT)], VDN_ROOT=str(self.d), VDN_UV_URL=(self.d / "missing.tar.gz").as_uri())
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn(f"已有 uv：{self.d / 'bin' / 'uv'}", r.stdout)
        self.assertIn("${VDN_ROOT:-/workspace}/bin", SCRIPT.read_text())

    def test_unknown_arch_refused(self):
        r = self.run_sh([str(SCRIPT)], VDN_BIN=str(self.d / "bin"), VDN_UV_ARCH="mips")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("不支持的架构", r.stderr)

    def test_cloud_steps_setup_uses_root_bin(self):
        """cloud_steps.sh setup：$VDN_ROOT/bin 进 PATH，调 setup_uv.sh，再用 uv 建 venv、装依赖；不再用 pip。"""
        log = self.d / "log"
        fake_uv(self.d / "bin", log)
        r = self.run_sh([str(STEPS), "setup"], VDN_ROOT=str(self.d), VDN_UV_URL=(self.d / "missing.tar.gz").as_uri())
        self.assertEqual(r.returncode, 0, r.stderr)
        calls = log.read_text().splitlines()
        self.assertEqual(len(calls), 2, calls)
        self.assertTrue(calls[0].startswith("venv "), calls)
        self.assertTrue(calls[1].startswith("pip install "), calls)
        self.assertIn(str(HERE / "requirements.txt"), calls[1])
        self.assertNotIn("pip install -q uv", STEPS.read_text())


class Constants(unittest.TestCase):
    def test_pins_well_formed(self):
        c = consts()
        self.assertRegex(c["UV_VERSION"], r"^\d+\.\d+\.\d+$")
        for k in ("UV_SHA256_X86_64", "UV_SHA256_AARCH64"):
            self.assertRegex(c[k], r"^[0-9a-f]{64}$", k)
        self.assertNotEqual(c["UV_SHA256_X86_64"], c["UV_SHA256_AARCH64"])

    def test_url_is_official_release(self):
        cmd = f'VDN_UV_LIB=1 source "{SCRIPT}"; uv_url "$(uv_target x86_64)"; uv_url "$(uv_target aarch64)"'
        out = subprocess.run(["bash", "-c", cmd], capture_output=True, text=True, check=True).stdout.split()
        v = consts()["UV_VERSION"]
        base = f"https://github.com/astral-sh/uv/releases/download/{v}/uv-"
        self.assertEqual(out, [base + "x86_64-unknown-linux-gnu.tar.gz", base + "aarch64-unknown-linux-gnu.tar.gz"])

    @unittest.skipUnless(os.environ.get("VDN_NET") == "1", "要联网，设 VDN_NET=1 人工跑")
    def test_pins_match_official_sha256_files(self):
        c = consts()
        for target, k in (("x86_64-unknown-linux-gnu", "UV_SHA256_X86_64"), ("aarch64-unknown-linux-gnu", "UV_SHA256_AARCH64")):
            url = f"https://github.com/astral-sh/uv/releases/download/{c['UV_VERSION']}/uv-{target}.tar.gz.sha256"
            req = urllib.request.Request(url, headers={"User-Agent": "generic-client"})
            with urllib.request.urlopen(req, timeout=30) as r:
                self.assertEqual(r.read().decode().split()[0], c[k], target)


if __name__ == "__main__":
    unittest.main()
