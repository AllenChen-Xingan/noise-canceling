"""假的 RunPod API：在内存里记机器、网络盘、文件，供测试注入 Client(transport=...)。不连网。"""

from __future__ import annotations

import base64
import hashlib
import json
import re
import shlex
import subprocess
import urllib.parse

import config as C  # noqa: E402

GPU = C.GPU_TYPE_ID


class FakeRunPod:
    def __init__(self, price=C.USD_PER_HR_EXPECTED, vcpu=16, queue=False):
        self.calls = []          # (method, path, body)
        self.headers_seen = []
        self.pods = {}
        self.volumes = {}
        self.files = {}          # 机器里的文件：{path: bytes}
        self.jobs = {}           # {job: (rc, log)}，start_job 后立即「跑完」
        self.price, self.vcpu, self.queue = price, vcpu, queue
        self.n = 0
        self.cpu_price = C.CPU_USD_PER_VCPU_HR_EXPECTED
        self.fail_exec = None    # 设成字符串时，含该子串的 exec 返回非零

    # transport 接口
    def __call__(self, method, url, headers, body):
        self.headers_seen.append(headers)
        u = urllib.parse.urlparse(url)
        q = urllib.parse.parse_qs(u.query)
        data = json.loads(body) if body else None
        self.calls.append((method, u.path, data))
        status, out = self.route(method, u.path, q, data)
        raw = b"" if out is None else json.dumps(out).encode()
        return status, {}, raw

    def route(self, m, p, q, d):
        if p == "/v2/catalog/gpus":
            return 200, {"gpus": [{"id": GPU, "price": {"secure": self.price, "community": 0.22},
                                   "minVcpuCount": self.vcpu, "availability": "HIGH"}]}
        if p == "/v2/catalog/cpus":
            return 200, {"cpus": [{"id": C.CPU_FLAVOR_ID, "price": {"securePerVcpu": self.cpu_price}},
                                  {"id": "cpu5c", "price": {"securePerVcpu": 0.035}}]}
        if p == "/v2/catalog/datacenters":
            return 200, {"dataCenters": [
                {"id": "EU-RO-1", "name": "RO", "region": "EUROPE", "networkVolumeTypes": ["STANDARD"],
                 "gpuAvailability": [{"id": GPU, "name": "gpu", "availability": "LOW"}]},
                {"id": "US-KS-2", "name": "KS", "region": "NA", "networkVolumeTypes": ["STANDARD"],
                 "gpuAvailability": [{"id": GPU, "name": "gpu", "availability": "HIGH"}]},
                {"id": "US-XX-9", "name": "no vol", "region": "NA", "networkVolumeTypes": [],
                 "gpuAvailability": [{"id": GPU, "name": "gpu", "availability": "HIGH"}]},
                {"id": "EU-SE-1", "name": "none", "region": "EUROPE", "networkVolumeTypes": ["STANDARD"],
                 "gpuAvailability": [{"id": GPU, "name": "gpu", "availability": "NONE"}]},
            ]}
        if p == "/v2/network-volumes" and m == "GET":
            return 200, {"networkVolumes": list(self.volumes.values()), "pagination": {"hasNextPage": False}}
        if p == "/v2/network-volumes" and m == "POST":
            self.n += 1
            v = {"id": f"vol_{self.n}", "name": d["name"], "size": d["size"], "dataCenter": d["dataCenter"]}
            self.volumes[v["id"]] = v
            return 201, v
        if p.startswith("/v2/network-volumes/") and m == "DELETE":
            vid = p.rsplit("/", 1)[1]
            if vid not in self.volumes:
                return 404, {"detail": "not found"}
            del self.volumes[vid]
            return 204, None
        if p == "/v2/pods" and m == "GET":
            return 200, {"pods": list(self.pods.values()), "pagination": {"hasNextPage": False}}
        if p == "/v2/pods" and m == "POST":
            self.n += 1
            if self.queue:
                return 202, {"id": f"q_{self.n}", "status": "QUEUED", "podId": None,
                             "gpu": {"id": GPU, "count": 1}, "failureReason": None}
            pod = {"id": f"pod_{self.n}", "name": d["name"], "status": "RUNNING", "cost": self.price,
                   "dataCenterId": d["dataCenterIds"][0],
                   "gpu": {"id": GPU, "count": 1, "vcpuCount": self.vcpu, "memory": 125}}
            if "cpu" in d:  # 纯 CPU 机器：没有 gpu 字段，价 = 每核价 × 核数
                assert "gpu" not in d, "gpu 和 cpu 只能二选一"
                pod.pop("gpu")
                pod["cpu"] = dict(d["cpu"])
                pod["cost"] = self.cpu_price * d["cpu"]["vcpuCount"]
            self.pods[pod["id"]] = pod
            return 201, pod
        m_pod = re.fullmatch(r"/v2/pods/([^/]+)(/exec)?", p)
        if m_pod:
            pid = m_pod.group(1)
            if pid not in self.pods:
                return 404, {"detail": "not found"}
            if m == "GET":
                return 200, self.pods[pid]
            if m == "DELETE":
                del self.pods[pid]
                return 204, None
            return 200, self.exec(d["command"])
        if p.startswith("/v2/queued-pods/") and m == "DELETE":
            return 204, None
        if p == "/v2/billing/pods":
            return 200, {"records": [], "metadata": {"totals": {"totalAmount": 0.01}}}
        return 404, {"detail": f"fake: no route {m} {p}"}

    # 只认 rp_ops 发出的几种命令形状
    def exec(self, argv):
        ok = {"stdout": "", "stderr": "", "exitCode": 0, "timedOut": False, "truncated": False, "durationMs": 1}
        assert len(argv) <= 256 and all(len(a) <= 4096 for a in argv), "超出 exec 参数上限"
        script = argv[2] if argv[:2] == ["sh", "-c"] else " ".join(argv)
        if self.fail_exec and self.fail_exec in script:
            return dict(ok, exitCode=1, stderr="boom")
        if script.startswith("sh -n -c "):
            body = shlex.split(script)[3]
            r = subprocess.run(["sh", "-n", "-c", body], capture_output=True, text=True)
            return dict(ok, exitCode=r.returncode, stderr=r.stderr)
        if script.startswith("test ! -e "):
            return dict(ok, exitCode=1 if shlex.split(script)[3] in self.files else 0)
        if script.startswith("sha256sum "):
            path = shlex.split(script)[1]
            return dict(ok, stdout=f"{hashlib.sha256(self.files[path]).hexdigest()}  {path}\n")
        if script.startswith("ln ") and "&& rm -f" in script:  # 不覆盖发布
            src, dst = shlex.split(script.split("&&")[0])[1:3]
            if dst in self.files:
                return dict(ok, exitCode=1, stderr="File exists")
            self.files[dst] = self.files.pop(src)
            return ok
        if script.startswith("if [ -e ") and "mv -f " in script:  # 覆盖发布：先留备份
            src, dst = shlex.split(script.split("mv -f ")[1])[:2]
            if dst in self.files:
                self.files[dst + ".bak"] = self.files[dst]
            self.files[dst] = self.files.pop(src)
            return ok
        if script.startswith("for i in 1 2 3 4 5"):
            return ok
        if "base64 -d >>" in script:
            path = shlex.split(script.split(">>", 1)[1])[0]
            self.files[path] = self.files.get(path, b"") + base64.b64decode("".join(argv[4:]))
            if getattr(self, "corrupt_put", False):  # 模拟传输损坏
                self.files[path] += b"!"
            return ok
        if ": >" in script:
            path = shlex.split(script.split(": >", 1)[1])[0]
            self.files[path] = b""
            return ok
        if script.startswith("mv "):
            src, dst = shlex.split(script.split("&&")[0])[1:3]
            self.files[dst] = self.files.pop(src)
            return dict(ok, stdout=f"{hashlib.sha256(self.files[dst]).hexdigest()}  {dst}\n")
        if script.startswith("stat -c"):
            path = shlex.split(script)[3]
            b = self.files[path]
            return dict(ok, stdout=f"{len(b)}\n{hashlib.sha256(b).hexdigest()}  {path}\n")
        if script.startswith("dd if="):
            parts = dict(x.split("=", 1) for x in shlex.split(script.split("|")[0])[1:] if "=" in x)
            bs, skip = int(parts["bs"]), int(parts["skip"])
            chunk = self.files[parts["if"]][skip * bs:(skip + 1) * bs]
            return dict(ok, stdout=base64.b64encode(chunk).decode())
        if "nohup sh -c" in script:
            job = re.search(r"/([\w.-]+)\.rc", script).group(1)
            self.jobs[job] = self.jobs.get(job, (0, '{"nproc": 16}\n'))
            return ok
        if script.startswith("cat ") and ".rc" in script:
            job = re.search(r"/([\w.-]+)\.rc", script).group(1)
            if job not in self.jobs:
                return dict(ok, stdout="RUNNING\n---\n")
            rc, log = self.jobs[job]
            return dict(ok, stdout=f"{rc}\n---\n{log}")
        return dict(ok, stdout=f"ran: {script}")
