"""检查 022 设备键表：键名格式、不重复、non-laptop 子键带前缀。

devices.md 是设备键的正本；mtgdata 的 check_cards.py 读同一列（行首 | `键` |）。
用法：python3 -I check_devices.py [devices.md 路径]；有问题退出码 1。
"""
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
DEFAULT = os.path.join(HERE, "..", "devices.md")
ROW = re.compile(r"^\| `([^`]+)` \|")
KEY = re.compile(r"^[a-z][a-z0-9]*(-[a-z0-9]+)*$")
PARENT = "non-laptop"


def read_keys(text):
    return [m.group(1) for m in (ROW.match(l) for l in text.splitlines()) if m]


def key_section(text):
    """第一节（键表）的正文：从「## 一、」到下一个「## 」之前。"""
    out, inside = [], False
    for line in text.splitlines():
        if line.startswith("## "):
            inside = line.startswith("## 一、")
            continue
        if inside:
            out.append(line)
    return "\n".join(out)


def stray_rows(text):
    """第一节以外、却会被下游（mtgdata check_cards.py）读成设备键的行。"""
    inside = read_keys(key_section(text))
    return [k for k in read_keys(text) if k not in inside]


def problems(keys):
    out = []
    seen = set()
    for k in keys:
        if not KEY.match(k):
            out.append("键名格式不对（小写英文、数字、连字符）：%s" % k)
        if k in seen:
            out.append("键重复：%s" % k)
        seen.add(k)
    if not keys:
        out.append("没读到任何键（表格式变了？行首应是 | `键` |）")
    if PARENT in seen and not any(k.startswith(PARENT + "-") for k in seen):
        out.append("有父类 non-laptop 却没有子键")
    return out


def main(argv):
    path = argv[1] if len(argv) > 1 else DEFAULT
    with open(path, encoding="utf-8") as fh:
        text = fh.read()
    keys = read_keys(key_section(text))
    errs = problems(keys)
    errs += ["第一节以外的行会被读成设备键：%s" % k for k in stray_rows(text)]
    for e in errs:
        print(e)
    print("设备键 %d 个，问题 %d 处" % (len(keys), len(errs)))
    return 1 if errs else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
