#!/usr/bin/env python3
"""检查 023 数据卡字段是否齐全、合规。

正本：字段名与顺序读 card-template.md；场景键读 021 `catalog.md`、设备键读 022 `devices.md`
（两表第一列反引号里的键）。本脚本和模板都不另抄键。
合并进 main 前，021 / 022 在旁边的 worktree 里，脚本会去那里找；两处都找不到就报错退出。
用法：python3 check_cards.py [卡片目录]，默认同目录下 cards/。有错退出码 1。
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
TEMPLATE = HERE / "card-template.md"

FIELD_RE = re.compile(r"^- \*\*(?P<name>[^*]+)\*\*：(?P<value>.*)$")
LICENSE_VERDICTS = ("训练可", "只评测", "不可", "未查到")
ROW_KEY_RE = re.compile(r"^\|\s*`([a-z0-9-]+)`\s*\|", re.M)
RESEARCH = HERE.parent
KEY_SOURCES = {  # 名称 → (仓内路径, 合并前所在的 worktree 名)
    "场景键": ("021_meeting-scenes/catalog.md", "noise_canceling-scenes"),
    "设备键": ("022_mac-mic/devices.md", "noise_canceling-macmic"),
}


def parse_fields(text: str) -> list[str]:
    """从模板「## 卡片格式」一节读字段顺序。"""
    block = text.split("## 卡片格式", 1)[-1] if "## 卡片格式" in text else ""
    fields = [m.group("name") for m in map(FIELD_RE.match, block.splitlines()) if m]
    if not fields:
        raise ValueError("card-template.md 缺「卡片格式」字段表")
    return fields


def parse_keys(text: str) -> set[str]:
    """读正本表格第一列反引号里的键。"""
    keys = set(ROW_KEY_RE.findall(text))
    if not keys:
        raise ValueError("键表里没读到任何 | `键` | 行")
    return keys


def find_key_source(name: str) -> Path:
    """先找本仓 docs/research/ 下的正本；合并前退到旁边的 worktree。"""
    rel, worktree = KEY_SOURCES[name]
    repo = RESEARCH.parent.parent
    for cand in (RESEARCH / rel, repo.parent / worktree / "docs" / "research" / rel):
        if cand.exists():
            return cand
    raise FileNotFoundError(f"找不到{name}正本 {rel}")


def load_schema() -> tuple[list[str], set[str], set[str]]:
    fields = parse_fields(TEMPLATE.read_text(encoding="utf-8"))
    scenes = parse_keys(find_key_source("场景键").read_text(encoding="utf-8"))
    devices = parse_keys(find_key_source("设备键").read_text(encoding="utf-8"))
    return fields, scenes, devices


def split_keys(value: str) -> list[str]:
    """「a, b（说明（嵌套））【来源】」→ ['a', 'b']；括号与来源标签里的文字不算键。"""
    prev = None
    while prev != value:  # 由内向外剥括号，处理嵌套
        prev, value = value, re.sub(r"[（(][^（）()]*[）)]", "", value)
    value = re.sub(r"【[^】]*】", "", value)
    return [k.strip() for k in re.split(r"[,，、;；]", value) if k.strip()]


def check_card(text: str, fields: list[str], scenes: set[str], devices: set[str]) -> list[str]:
    """返回这张卡的问题清单；空表示合格。只看第一个 `## ` 小节之前的字段区。"""
    errs: list[str] = []
    head = text.split("\n## ", 1)[0]
    if not head.lstrip().startswith("# "):
        errs.append("缺一级标题")
    found = [(m.group("name"), m.group("value").strip())
             for m in map(FIELD_RE.match, head.splitlines()) if m]
    names = [n for n, _ in found]
    for f in fields:
        if f not in names:
            errs.append(f"缺字段「{f}」")
    dup = {n for n in names if names.count(n) > 1}
    for n in sorted(dup):
        errs.append(f"字段重复「{n}」")
    extra = [n for n in names if n not in fields]
    for n in extra:
        errs.append(f"多出未知字段「{n}」")
    known = [n for n in names if n in fields and n not in dup]
    if not errs and known != fields:
        errs.append("字段顺序与模板不一致")
    values = dict(found)
    for n, v in found:
        if not v:
            errs.append(f"字段「{n}」为空（查不到请写「未查到」）")
    lic = values.get("许可", "")
    if lic and not any(f"结论：{v}" in lic for v in LICENSE_VERDICTS):
        errs.append("「许可」缺「结论：训练可 / 只评测 / 不可 / 未查到」")
    for name, allowed in (("场景键", scenes), ("设备键", devices)):
        v = values.get(name, "")
        if not v or v == "未查到":
            continue
        for k in split_keys(v):
            if k not in allowed:
                errs.append(f"「{name}」用了模板外的键 `{k}`")
    src = values.get("出处", "")
    if src and src != "未查到" and "【" not in src:
        errs.append("「出处」缺来源标签【…】")
    return errs


def check_addendum(text: str, scenes: set[str], devices: set[str]) -> list[str]:
    """补字段文件里每行「场景键 / 设备键」也只许用模板里的键。"""
    errs: list[str] = []
    for i, line in enumerate(text.splitlines(), 1):
        m = FIELD_RE.match(line)
        if not m or m.group("name") not in ("场景键", "设备键"):
            continue
        allowed = scenes if m.group("name") == "场景键" else devices
        for k in split_keys(m.group("value")):
            if k != "未查到" and k not in allowed:
                errs.append(f"第 {i} 行「{m.group('name')}」用了模板外的键 `{k}`")
    return errs


def main(argv: list[str]) -> int:
    card_dir = Path(argv[1]) if len(argv) > 1 else HERE / "cards"
    fields, scenes, devices = load_schema()
    cards = sorted(p for p in card_dir.glob("*.md") if p.name != "README.md")
    bad = 0
    for p in cards:
        errs = check_card(p.read_text(encoding="utf-8"), fields, scenes, devices)
        if errs:
            bad += 1
            for e in errs:
                print(f"{p.name}: {e}")
    addendum = HERE / "cards-014-addendum.md"
    if card_dir == HERE / "cards" and addendum.exists():
        for e in check_addendum(addendum.read_text(encoding="utf-8"), scenes, devices):
            bad += 1
            print(f"{addendum.name}: {e}")
    print(f"检查 {len(cards)} 张卡，{bad} 处问题")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
