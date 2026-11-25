"""RunPod API v2 的薄客户端：读密钥、发请求。只用标准库。

密钥从不打印、不进日志、不进命令行参数：先看环境变量，再读主仓库根的 .env.local。
测试用 `transport` 参数注入假响应，不连网。
"""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import time
import urllib.error
import urllib.parse
import urllib.request
from typing import Any, Callable, Optional

import config as C

# transport(method, url, headers, body_bytes) -> (status, headers_dict, body_bytes)
Transport = Callable[[str, str, dict, Optional[bytes]], "tuple[int, dict, bytes]"]


class ApiError(RuntimeError):
    def __init__(self, status: int, method: str, path: str, detail: str):
        super().__init__(f"{method} {path} -> {status}: {detail}")
        self.status = status


def main_repo_root(start: Optional[str] = None) -> str:
    """worktree 里也返回主仓库根（git common dir 的上一级）。"""
    here = start or os.path.dirname(os.path.abspath(__file__))
    out = subprocess.run(
        ["git", "-C", here, "rev-parse", "--path-format=absolute", "--git-common-dir"],
        capture_output=True, text=True, check=True,
    ).stdout.strip()
    return os.path.dirname(out)


def parse_env_file(text: str) -> dict:
    vals = {}
    for line in text.splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        k, v = line.split("=", 1)
        k = k.strip()
        if k.startswith("export "):
            k = k[len("export "):].strip()
        v = v.strip()
        if len(v) >= 2 and v[0] == v[-1] and v[0] in "\"'":
            v = v[1:-1]
        vals[k] = v
    return vals


def load_key(env: Optional[dict] = None, env_file: Optional[str] = None) -> str:
    env = os.environ if env is None else env
    if env.get(C.KEY_NAME):
        return env[C.KEY_NAME]
    path = env_file or os.path.join(main_repo_root(), ".env.local")
    try:
        with open(path, encoding="utf-8") as f:
            key = parse_env_file(f.read()).get(C.KEY_NAME, "")
    except FileNotFoundError:
        key = ""
    if not key:
        raise SystemExit(f"{C.KEY_NAME} 未设置：环境变量和 .env.local 里都没有")
    return key


def urllib_transport(method, url, headers, body):
    req = urllib.request.Request(url, data=body, method=method, headers=headers)
    try:
        with urllib.request.urlopen(req, timeout=60) as r:
            return r.status, dict(r.headers), r.read()
    except urllib.error.HTTPError as e:
        return e.code, dict(e.headers or {}), e.read() or b""


class Client:
    def __init__(self, key: str, transport: Transport = urllib_transport,
                 sleep: Callable[[float], None] = time.sleep, base: str = C.API_BASE):
        self._key = key
        self._t = transport
        self._sleep = sleep
        self._base = base.rstrip("/")

    def __repr__(self):  # 防止密钥随对象打印出去
        return f"Client(base={self._base!r}, key=<{len(self._key)} chars>)"

    def fingerprint(self) -> str:
        """密钥的短指纹（sha256 前 12 位），用来认「是不是同一个账号」，不可逆推出密钥。"""
        return hashlib.sha256(self._key.encode()).hexdigest()[:12]

    def request(self, method: str, path: str, body: Any = None, query: Optional[dict] = None,
                retries: int = 5, retry_on: tuple = (409, 429, 503)) -> Any:
        url = self._base + path
        if query:
            url += "?" + urllib.parse.urlencode(query, doseq=True)
        headers = {"Authorization": f"Bearer {self._key}", "Accept": "application/json"}
        data = None
        if body is not None:
            data = json.dumps(body).encode()
            headers["Content-Type"] = "application/json"
        for attempt in range(retries + 1):
            status, rh, raw = self._t(method, url, headers, data)
            if status in retry_on and attempt < retries and _retry_after(rh) is not None:
                self._sleep(_retry_after(rh))
                continue
            break
        if status >= 400:
            raise ApiError(status, method, path, _detail(raw, self._key))
        if not raw:
            return None
        return json.loads(raw)

    def get(self, path, **q):
        return self.request("GET", path, query=q or None)

    def post(self, path, body, retries: int = 5, retry_on: tuple = (409, 429, 503)):
        return self.request("POST", path, body=body, retries=retries, retry_on=retry_on)

    def delete(self, path):
        return self.request("DELETE", path)


def _retry_after(headers: dict) -> Optional[float]:
    for k, v in headers.items():
        if k.lower() == "retry-after":
            try:
                return min(float(v), 30.0)
            except ValueError:
                return 5.0
    return None


def _detail(raw: bytes, key: str = "") -> str:
    """错误详情：先把密钥换成 <key>，再截到 300 字（先截后换会留下密钥残片，评审第 2 轮 #6）。"""
    try:
        j = json.loads(raw)
        text = str(j.get("detail") or j.get("title") or j)
    except Exception:
        text = raw.decode("utf-8", "replace")
    if key:
        text = text.replace(key, "<key>")
    return text[:300]