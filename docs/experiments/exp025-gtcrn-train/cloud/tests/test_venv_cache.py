"""remote/venv_cache.sh：key 稳定 · 首次装包并发布 · 再次解包不装包 · 不覆盖别人的 tar · 最后一行 PY=。
用假的 uv 与假的 tar 在本机跑，不连网，不碰真实云机器。
"""

import os
import stat
import subprocess
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
SCRIPT = os.path.join(os.path.dirname(HERE), "remote", "venv_cache.sh")

# 假 uv：记参数；--version 打版本号；venv 时按最后那个参数建出 <dir>/bin/python
FAKE_UV = r"""#!/bin/sh
echo "uv $*" >> LOGPATH
if [ "$1" = "--version" ]; then echo "uv 0.5.0 (fake)"; exit 0; fi
if [ "$1" = "venv" ]; then
  for a in "$@"; do d="$a"; done
  mkdir -p "$d/bin"
  printf '#!/bin/sh\n' > "$d/bin/python"
  chmod +x "$d/bin/python"
fi
"""

# 假 tar：-cf <part> -C <root> <key> 写占位内容；-xf <tar> -C <root> 建出 <root>/<key>/bin/python。
# FAKE_TAR_RACE=1 时，-cf 的同时抢先发布成品（模拟另一台机器同时发布）。
FAKE_TAR = r"""#!/bin/sh
flag=$1
file=$2
root=$4
case "$flag" in
  -cf)
    key=$5
    printf 'fake-tar\n' > "$file"
    if [ "${FAKE_TAR_RACE:-0}" = "1" ]; then printf 'someone-else\n' > "${file%%.part-*}"; fi
    ;;
  -xf)
    key=$(basename "$file" .tar)
    mkdir -p "$root/$key/bin"
    printf '#!/bin/sh\n' > "$root/$key/bin/python"
    chmod +x "$root/$key/bin/python"
    ;;
esac
"""


def _exe(path: str, body: str, log: str = "") -> None:
    if log:
        body = body.replace("LOGPATH", log)
    with open(path, "w") as f:
        f.write(body)
    os.chmod(path, os.stat(path).st_mode | stat.S_IEXEC)


class _Case(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.t = self._tmp.name
        self.bin = os.path.join(self.t, "bin")
        os.makedirs(self.bin)
        self.log = os.path.join(self.t, "uv.log")
        _exe(os.path.join(self.bin, "uv"), FAKE_UV, self.log)
        _exe(os.path.join(self.bin, "tar"), FAKE_TAR)
        self.store = os.path.join(self.t, "store")
        self.root = os.path.join(self.t, "root")
        self.reqs = []
        for i, body in enumerate(("a==1\n", "b==2\n")):
            p = os.path.join(self.t, f"req{i}.txt")
            with open(p, "w") as f:
                f.write(body)
            self.reqs.append(p)

    def tearDown(self):
        self._tmp.cleanup()

    def env(self, **extra):
        e = dict(os.environ)
        e["PATH"] = self.bin + os.pathsep + e.get("PATH", "")
        e["VENV_STORE"] = extra.pop("store", self.store)
        e["VENV_ROOT"] = extra.pop("root", self.root)
        e["UV_PYTHON_INSTALL_DIR"] = os.path.join(self.t, "uvpy")
        e["UV_CACHE_DIR"] = os.path.join(self.t, "uvcache")
        e.update(extra)
        return e

    def run_script(self, reqs=None, **extra):
        reqs = self.reqs if reqs is None else reqs
        return subprocess.run(["sh", SCRIPT] + reqs, capture_output=True, text=True,
                              env=self.env(**extra))

    def log_lines(self):
        if not os.path.exists(self.log):
            return []
        with open(self.log) as f:
            return [l.rstrip("\n") for l in f]

    def built_with(self, needle="uv venv --relocatable --python 3.12.14 "):
        return any(l.startswith(needle) for l in self.log_lines())

    def key_of(self, store=None):
        store = store or self.store
        names = [n for n in os.listdir(store) if n.endswith(".tar")]
        self.assertEqual(len(names), 1, f"网络盘上应有且只有一个 tar，实有 {names}")
        return names[0][: -len(".tar")]


class TestVenvCache(_Case):
    def test_syntax(self):
        r = subprocess.run(["sh", "-n", SCRIPT], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)

    def test_key_stable_and_content_sensitive(self):
        r = self.run_script()
        self.assertEqual(r.returncode, 0, r.stderr)
        key1 = self.key_of()
        # 同一份内容、换一个路径与另一个网络盘：key 不变
        same = os.path.join(self.t, "req_copy.txt")
        with open(same, "w") as f:
            f.write("a==1\n")
        r = self.run_script([same, self.reqs[1]], store=os.path.join(self.t, "store2"),
                            root=os.path.join(self.t, "root2"))
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(self.key_of(os.path.join(self.t, "store2")), key1)
        # 改一个字：key 变
        changed = os.path.join(self.t, "req_changed.txt")
        with open(changed, "w") as f:
            f.write("a==1.0\n")
        r = self.run_script([changed, self.reqs[1]], store=os.path.join(self.t, "store3"),
                            root=os.path.join(self.t, "root3"))
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertNotEqual(self.key_of(os.path.join(self.t, "store3")), key1)

    def test_first_call_builds_and_publishes(self):
        r = self.run_script()
        self.assertEqual(r.returncode, 0, r.stderr)
        log = self.log_lines()
        self.assertTrue(self.built_with(), "要 uv venv --relocatable")
        installs = [l for l in log if l.startswith("uv pip install")]
        self.assertEqual(len(installs), len(self.reqs), "每份 requirements 一次 uv pip install")
        self.assertFalse([l for l in log if " -q" in l], "不带 -q，要保留 uv 自己的计时段")
        steps = [l.split()[1] for l in r.stdout.splitlines() if l.startswith("=== ")]
        self.assertEqual(steps[0], "venv")
        self.assertEqual(steps[1:3], ["pip-1", "pip-2"])
        self.assertIn("pack", steps)
        self.assertTrue(all("seconds=" in l for l in r.stdout.splitlines() if l.startswith("=== ")))
        key = self.key_of()
        with open(os.path.join(self.store, key + ".tar")) as f:
            self.assertEqual(f.read(), "fake-tar\n", "tar 进了网络盘")
        self.assertTrue(os.path.exists(os.path.join(self.store, key + ".tar.sha256")), "要写校验文件")
        self.assertEqual([n for n in os.listdir(self.store) if ".part-" in n], [], "不该留 .part")
        self.assertEqual(r.stdout.splitlines()[-1], f"PY={self.root}/{key}/bin/python")
        self.assertTrue(os.path.exists(r.stdout.splitlines()[-1][3:]))

    def test_second_call_restores_without_install(self):
        self.assertEqual(self.run_script().returncode, 0)
        key = self.key_of()
        open(self.log, "w").close()
        root2 = os.path.join(self.t, "root_fresh")
        os.makedirs(root2)
        r = self.run_script(root=root2)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertFalse([l for l in self.log_lines() if "pip install" in l], "解包路径不许装包")
        self.assertFalse([l for l in self.log_lines() if l.startswith("uv venv")])
        self.assertIn("=== restore seconds=", r.stdout)
        self.assertEqual(r.stdout.splitlines()[-1], f"PY={root2}/{key}/bin/python")
        self.assertTrue(os.path.exists(os.path.join(root2, key, "bin", "python")))

    def test_leftover_part_file_means_not_published(self):
        self.assertEqual(self.run_script().returncode, 0)
        key = self.key_of()
        store2 = os.path.join(self.t, "store_part")
        os.makedirs(store2)
        with open(os.path.join(store2, key + ".tar.part-xyz"), "w") as f:
            f.write("half\n")
        open(self.log, "w").close()
        r = self.run_script(store=store2, root=os.path.join(self.t, "root_part"))
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertTrue(self.built_with(), "只有 .part 就重新装")
        self.assertTrue(os.path.exists(os.path.join(store2, key + ".tar")), "这次要发布成品")

    def test_existing_tar_is_never_overwritten(self):
        race = os.path.join(self.t, "store_race")
        r = self.run_script(store=race, root=os.path.join(self.t, "root_race"), FAKE_TAR_RACE="1")
        self.assertEqual(r.returncode, 0, r.stderr)
        key = self.key_of(race)
        with open(os.path.join(race, key + ".tar")) as f:
            self.assertEqual(f.read(), "someone-else\n", "别人的成品不许被覆盖")
        self.assertEqual([n for n in os.listdir(race) if ".part-" in n], [], "自己的那份要删掉")


if __name__ == "__main__":
    unittest.main()
