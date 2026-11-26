"""cloud_steps.sh：分片目录必须显式给（2026-11-27 大总管定：不再默认到第一批）。

shard · register · pack 三步读分片目录；没设 VDN_SHARDS 就停下、退出码 2，绝不落到旧的第一批目录里。
其余步骤（setup · test · head · fetch · fetch-e3 · inventory · split · rebuild）照旧，不需要它。
不联网：脚本在调用 python 之前就被拦下，所以不需要真环境。
"""
import os
import re
import subprocess
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
SCRIPT = HERE / "cloud_steps.sh"
SYS_PATH = "/usr/bin:/bin"
NEED_SHARDS = ("shard", "register", "pack")
NO_SHARDS = ("setup", "test", "head", "fetch", "fetch-e3", "inventory", "split", "rebuild")


def branch_bodies():
    """把 case 里每个步骤的正文按步骤名切开：{步骤: 正文}。"""
    text = SCRIPT.read_text()
    starts = [(m.start(), m.group(1)) for m in re.finditer(r"^  ([a-z0-9-]+)\)", text, re.M)]
    out, tail = {}, text.rfind("esac")
    for i, (pos, name) in enumerate(starts):
        end = starts[i + 1][0] if i + 1 < len(starts) else tail
        out[name] = text[pos:end]
    return out


class Guard(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.d = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def run_step(self, *args, **env):
        """干净环境：只有 PATH · HOME · VDN_ROOT，没有 VDN_SHARDS。"""
        e = {"PATH": SYS_PATH, "HOME": str(self.d), "VDN_ROOT": str(self.d), **env}
        return subprocess.run(["bash", str(SCRIPT), *args], env=e, capture_output=True, text=True)

    def test_steps_needing_shards_stop_without_it(self):
        """三步都不落下 python：退出码 2，stderr 提到 VDN_SHARDS，$VDN_ROOT/shards 下什么都没建。"""
        for step in NEED_SHARDS:
            with self.subTest(step=step):
                r = self.run_step(step)
                self.assertEqual(r.returncode, 2, (step, r.stdout, r.stderr))
                self.assertIn("VDN_SHARDS", r.stderr, step)
                self.assertEqual(sorted(os.listdir(self.d / "shards")) if (self.d / "shards").exists() else [],
                                 [], step)

    def test_empty_value_also_stops(self):
        """设了但是空串：同样拒绝（不能当成「用默认值」）。"""
        for step in NEED_SHARDS:
            with self.subTest(step=step):
                r = self.run_step(step, VDN_SHARDS="")
                self.assertEqual(r.returncode, 2, (step, r.stderr))
                self.assertIn("VDN_SHARDS", r.stderr, step)

    def test_no_step_prints_usage(self):
        """不给步骤：打印用法行，退出码 2（照旧）。"""
        r = self.run_step()
        self.assertEqual(r.returncode, 2, r.stderr)
        self.assertIn("用法", r.stdout)


class ScriptText(unittest.TestCase):
    def test_only_guard_reads_shards_dir(self):
        """正文里只有一处读 VDN_SHARDS（守卫里那行赋值），消息文字不算。"""
        reads = [ln.strip() for ln in SCRIPT.read_text().splitlines()
                 if re.search(r"\$\{?VDN_SHARDS", ln) and "echo" not in ln]
        self.assertEqual(len(reads), 1, reads)
        self.assertTrue(reads[0].startswith("SHARDS=${VDN_SHARDS"), reads)

    def test_guard_used_by_three_steps_only(self):
        """need_shards 只被 shard · register · pack 调，其余步骤一行都不碰。"""
        bodies = branch_bodies()
        for step in NEED_SHARDS:
            self.assertIn("need_shards", bodies[step], step)
        for step in NO_SHARDS:
            self.assertNotIn("need_shards", bodies[step], step)
            self.assertNotIn("VDN_SHARDS", bodies[step], step)

    def test_no_default_shards_dir(self):
        """脚本里不再提到第一批的路径（"first-batch"），也没有旧默认值写法。"""
        text = SCRIPT.read_text()
        self.assertNotIn("first-batch", text)
        self.assertNotIn("${VDN_SHARDS:-$ROOT", text)


if __name__ == "__main__":
    unittest.main()
