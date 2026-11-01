"""人耳偏好数据集：把各实验的盲听答卷揭盲、统一成一种格式。

只读仓里已提交的答卷与对照表，不跑任何模型、不读音频（音频哈希取自各 build 的 files.json / clips.json）。
确定性：同样的输入 → 逐字节相同的输出（键排序、无时间戳）。

输出（本目录）：
  trials.jsonl   每屏一条（含检验题）：揭盲后的全部答案、人原话备注、位置、时间、音频哈希
  pairs.jsonl    从每屏推出的两两比较（谁胜 / 平），按维度一行；给 verifier 定标（012 §2.1 L1）与虚拟听众（D48）用
  summary.json   计数、位置偏好、每个会话的有效性
字段说明见 schema.md。

用法：python3 docs/experiments/human-prefs/build_dataset.py [--check]
  --check  只重算并与仓里现有输出比对，不一致则退出码 1（给 CI 用）

接新答卷：
  exp003 / exp006 答卷放进各自的 results/，exp006 无需改代码（自动发现 results/*.json，跳过 *.unblinded.json）；
  新实验写一个适配器函数（返回同格式的 trial 记录），在 collect() 里加一行调用。
"""
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
EXP = HERE.parent
REPO = EXP.parent.parent

GRADE = {"ok": 2, "marginal": 1, "no": 0}

# 维度：键 → 中文名。best-of 问题的「赢家」= 被选中的那份
DIMS = {
    "clearer": "清楚",
    "natural": "自然",
    "meeting": "开会可用",
    "eaten": "吃字（没被勾的胜）",
    "quietest": "最安静（旁人声音最小）",
    "defects": "缺陷标签（没被标的胜，标签可选、证据弱）",
}

# 场景标签：fine 与 exp006 recipes.json 的 7 个噪音大类同名；user 与 recipes.json user_scene 同名
EXP003_SCENE = {
    "c01": ("babble", "cafe", "赌场人声 + 机器声 5 dB"),
    "c02": ("babble", "cafe", "赌场人声 + 机器声 0 dB"),
    "c03": ("babble", "cafe", "赌场人声 + 机器声 −5 dB"),
    "c04": ("neighbor", "cafe", "邻座 1 人 + 咖啡馆露台底噪 5 dB"),
    "c05": ("neighbor", "cafe", "邻座 1 人 + 咖啡馆露台底噪 0 dB"),
    "c06": ("babble", "cafe", "邻桌 3 人 + 食堂底噪 0 dB"),
    "c07": ("babble", "cafe", "邻桌 6 人 + 露台底噪 −5 dB"),
    "c08": ("babble", "cafe", "邻桌 4 人 + 食堂底噪 10 dB"),
    "c09": ("clatter", "cafe", "厨房杯碟水声 0 dB，无旁人说话"),
    "c10": ("music", "music", "背景钢琴 5 dB，无旁人说话"),
    "c11": ("babble", "cafe", "中文目标 · 英文邻桌 3 人 + 食堂底噪 0 dB"),
    "c12": ("neighbor", "cafe", "中文目标 · 英文邻座 1 人 + 露台底噪 5 dB"),
}


def jload(p):
    return json.loads(Path(p).read_text())


def rel(p):
    return str(Path(p).resolve().relative_to(REPO))


def scene_exp005(c):
    p = c["params"]
    bed = "无底噪" if p.get("snr") is None else f"食堂底噪 PCAFETER SNR {p['snr']} dB"
    if c["kind"] == "owner_only":
        return ("babble", "cafe", f"只有主人 + {bed}（底噪自带远处人声）")
    if c["kind"] == "multi":
        return ("babble", "cafe", f"邻桌 {p['n_nb']} 人一直在聊（每人比主人轻 {p['sir']} dB）+ {bed}")
    extra = f"，邻座先说 {p['first_s']} s" if c["kind"] == "other_first" else ""
    user = "quiet" if p.get("snr") is None else "cafe"
    return ("neighbor", user, f"邻座 1 人（比主人轻 {p['sir']} dB）轮流说{extra} + {bed}")


# ---------------------------------------------------------------- 各实验适配器
def exp003_full(ans_path, build_dir, clips):
    ans = jload(ans_path)
    traps = {t["trap"]: t for t in ans["summary"]["traps"]}
    trap_identical_pass = traps.get("trap-identical", {}).get("pass")
    trap_degraded = traps.get("trap-degraded", {})
    # rubric 4.2：只错 trap-identical → 打折；trap-degraded 答「分不出」→ 打折
    deg_same = trap_degraded.get("answer", {}).get("natural") == "same" or trap_degraded.get("answer", {}).get("clearer") == "same"
    if trap_degraded.get("pass") is False:
        validity, why = "void", "trap-degraded 选了劣化版"
    elif trap_identical_pass is False or deg_same:
        validity, why = "discounted", "trap-identical 没过（全同题选了 C）；trap-degraded「最自然」答分不出（rubric 4.2 打折）"
    else:
        validity, why = "valid", "两道检验题都过"
    out = []
    for t in ans["trials"]:
        slots = dict(t["slots"])
        # 全同检验题三个槽位都叫 "same"，与「分不出」的取值撞名 → 改名 identical#1/#2/#3，答案从原始字母重新揭
        if len(set(slots.values())) < len(slots):
            seen = {}
            for L in sorted(slots):
                m = slots[L]
                seen[m] = seen.get(m, 0) + 1
                slots[L] = f"identical#{seen[m]}" if m == "same" else f"{m}#{seen[m]}"
        raw = t["answer"]
        mp = lambda x: slots.get(x, x)  # 字母 → 版本；"same" / "none" 原样
        bm = {"clearer": mp(raw["clearer"]), "natural": mp(raw["natural"]),
              "eaten": [mp(x) for x in raw.get("eaten", [])],
              "usable": {mp(k): v for k, v in raw.get("usable", {}).items()},
              "tags": {mp(k): v for k, v in raw.get("tags", {}).items()}}
        models = [m for m in slots.values()]
        pos = {m: L for L, m in slots.items()}
        meta = clips.get(t["clip"], {})
        fine, user, desc = EXP003_SCENE[t["clip"]]
        rec = base_rec("exp003", ans["build"], ans_path, build_dir / "unblind-key.json", t["order"], t)
        rec.update({
            "clip": t["clip"], "scene": {"fine": fine, "user": user, "desc": desc, "lang": meta.get("lang")},
            "systems": sorted(models), "positions": pos, "qorder": t.get("qorder"),
            "questions": ["clearer", "natural", "eaten", "usable", "tags"],
            "answers": {
                "clearer": bm["clearer"], "natural": bm["natural"], "quietest": None, "pick": None,
                "eaten": [m for m in bm["eaten"] if m not in ("none",)] if "none" not in bm["eaten"] else [],
                "eaten_none": "none" in bm["eaten"],
                "usable": bm["usable"], "tags": bm["tags"],
            },
            "note": t.get("note", ""),
            "listen_s": {(slots.get(k, k)): round(v, 2) for k, v in (t.get("listen_s") or {}).items()},
            "plays": {(slots.get(k, k)): v for k, v in (t.get("plays") or {}).items()},
            "switches": t.get("switches"),
            "started_at": t.get("started_at"), "answered_at": t.get("answered_at"),
            "session": {"id": f"exp003/{ans['build']}", "validity": validity, "why": why,
                        "started": ans.get("started"), "finished": ans.get("finished")},
            "audio": {
                "noisy_sha256": meta.get("noisy_sha256"), "clean_sha256": meta.get("clean_sha256"),
                "out_sha256": {m: None for m in models},
                "out_sha256_missing": "exp003 build 的 files.json 没记哈希，音频（_tmp/20261030-exp003/）本机已不在；复现走 enhance.py",
                "gen": {"script": "docs/experiments/exp003-offline-ab/make_clips.py", "seed": 20261030,
                        "enhance": "docs/experiments/exp003-offline-ab/enhance.py",
                        "loudness": "说话区间 BS.1770 对齐 −26 LUFS（build 20261030110755-f8c3 起）"},
            },
        })
        if t["kind"] == "trap":
            tr = traps.get(t["trap"], {})
            rec["trap"] = {"type": t["trap"], "pass": tr.get("pass"), "expect": tr.get("desc")}
            rec["raw_answer_letters"] = raw
        out.append(rec)
    return out


def exp003_partial(ans_path, key_path, clips):
    """旧 build 68ab：只有 c01–c03 的槽位能恢复（recovered-key.json），其余屏写进记录但标 unblindable。"""
    ans = jload(ans_path)
    key = jload(key_path)
    out = []
    for i, t in enumerate(ans["trials"], 1):
        if t["clip"] not in ans["answers"]:
            continue  # 没答到
        a = ans["answers"][t["clip"]] if t["kind"] == "real" else None
        if a is None or not a.get("answered_at"):
            continue  # 打开了但没答完（如第 7 屏 c07 只听了 B）
        rec = base_rec("exp003", key["build"], ans_path, key_path, i, t)
        fine, user, desc = EXP003_SCENE[t["clip"]]
        rec["clip"] = t["clip"]
        rec["scene"] = {"fine": fine, "user": user, "desc": desc, "lang": clips.get(t["clip"], {}).get("lang")}
        rec["session"] = {"id": f"exp003/{key['build']}", "validity": "unverified",
                          "why": "答到第 6 题中止，两道检验题都没答到；揭盲表丢失，只恢复了 c01–c03",
                          "started": ans.get("started"), "finished": None}
        rec["note"] = a.get("note", "")
        rec["started_at"], rec["answered_at"] = a.get("started_at"), a.get("answered_at")
        rec["switches"] = a.get("switches")
        rec["questions"] = ["clearer", "natural", "eaten", "usable", "tags"]
        if t["clip"] not in key["clips"]:
            rec["unblindable"] = True
            rec["systems"] = ["dpdfnet2", "gtcrn"]
            rec["answers"] = None
            rec["raw_answer_letters"] = {k: a.get(k) for k in ("clearer", "natural", "eaten", "usable", "tags")}
            out.append(rec)
            continue
        names = key["clips"][t["clip"]]
        L2M = {L: names[idx] for L, idx in t["slots"].items()}
        mp = lambda x: L2M.get(x, "same" if x == "same" else x)
        rec["systems"] = sorted(names)
        rec["positions"] = {m: L for L, m in L2M.items()}
        rec["answers"] = {
            "clearer": mp(a["clearer"]), "natural": mp(a["natural"]), "quietest": None, "pick": None,
            "eaten": [mp(x) for x in a["eaten"] if x != "none"], "eaten_none": "none" in a["eaten"],
            "usable": {mp(k): v for k, v in a["usable"].items()},
            "tags": {mp(k): v for k, v in a.get("tags", {}).items()},
        }
        rec["plays"] = {mp(k) if k != "ref" else "ref": v for k, v in a.get("plays", {}).items()}
        rec["audio"] = {"noisy_sha256": clips[t["clip"]].get("noisy_sha256"),
                        "clean_sha256": clips[t["clip"]].get("clean_sha256"),
                        "out_sha256": {m: None for m in names},
                        "out_sha256_missing": "旧 build 音频已删（exp003 README「事故与教训」）",
                        "gen": {"enhance": "docs/experiments/exp003-offline-ab/enhance.py",
                                "loudness": "整段响度对齐（旧口径，比现口径偏向停顿里压得狠的模型）"}}
        out.append(rec)
    return out


def exp005(ans_path, build_dir, clips_path):
    ans = jload(ans_path)
    key = jload(build_dir / "unblind-key.json")
    files = jload(build_dir / "files.json")
    clips = {c["id"]: c for c in jload(clips_path)["clips"]}
    sha = {}
    for f, v in files.items():
        parts = v.get("what", "").split()
        if len(parts) == 2:
            sha[(parts[0], parts[1])] = v.get("sha256")
    trap_ok = True
    out = []
    # 页面的题序在导出里没有（answers 是对象，按 JSON 键序 = 作答顺序存入的可能性大，但未证实）→ trial_order 记键序并注明
    for i, (qid, a) in enumerate(ans["answers"].items(), 1):
        k = key["key"][qid]
        order = k["order"]
        L2M = {"ABC"[j]: m for j, m in enumerate(order)}
        is_trap = k.get("trap", False)
        if is_trap:
            # 两个槽位相同：用 "personal#1/#2" 区分位置
            seen = {}
            for L, m in list(L2M.items()):
                seen[m] = seen.get(m, 0) + 1
                if order.count(m) > 1:
                    L2M[L] = f"{m}#{seen[m]}"

        def mp(x):
            if x in L2M:
                return L2M[x]
            return {"分不出": "same", "都没有": "none", "原声": "reference"}.get(x, x)

        eaten_raw = a.get("eaten", [])
        eaten = [mp(x) for x in eaten_raw if x != "都没有"]
        c = clips[k["clip"]]
        fine, user, desc = scene_exp005(c)
        rec = base_rec("exp005", ans["build"], ans_path, build_dir / "unblind-key.json", i, {"kind": "trap" if is_trap else "real"})
        rec.update({
            "clip": k["clip"], "qid": qid, "excerpt_s": k.get("excerpt_s"), "lock_t": k.get("lock_t"),
            "scene": {"fine": fine, "user": user, "desc": desc, "lang": "en"},
            "systems": sorted(set(L2M.values())), "positions": {m: L for L, m in L2M.items()}, "qorder": None,
            "questions": ["quietest", "eaten", "natural", "pick"],
            "answers": {
                "clearer": None, "natural": mp(a.get("natural")), "quietest": mp(a.get("quietest")),
                "pick": mp(a.get("pick")), "eaten": eaten, "eaten_none": "都没有" in eaten_raw,
                "usable": None, "tags": None,
            },
            "note": a.get("note", ""),
            "trial_order_note": "导出没有题序与时间戳；trial_order = 答卷 JSON 的键序（未证实等于作答顺序）",
            "session": {"id": f"exp005/{ans['build']}", "validity": None, "why": None,
                        "started": None, "finished": ans.get("exported_at")},
            "audio": {
                "noisy_sha256": sha.get((k["clip"], "mix")),
                "out_sha256": {m: sha.get((k["clip"], m.split("#")[0])) for m in set(L2M.values())},
                "note": "哈希是听测用的 25 s 节选（excerpt_s），不是整段",
                "gen": {"script": "docs/experiments/exp005-p2-speaker/make_clips.py",
                        "run": "docs/experiments/exp005-p2-speaker/score.py run（输出 _tmp/20261031-exp005/out-r2/）",
                        "page": "docs/experiments/exp005-p2-speaker/build_page.py",
                        "params": key["cands"]},
            },
        })
        if is_trap:
            vals = [rec["answers"][q] for q in ("quietest", "natural")]
            # 期望：两份相同的槽位不被分出高下——选到其中任一个「#」版本即算没过
            passed = not any(isinstance(v, str) and "#" in v for v in vals)
            trap_ok = trap_ok and passed
            rec["trap"] = {"type": "trap-identical-pair", "pass": passed, "expect": k.get("expect")}
        out.append(rec)
    validity = "valid" if trap_ok else "discounted"
    why = ("唯一一道检验题（两槽位相同）过了；没有劣化检验题；导出没有 listen_s / 时间戳，rubric 4.2 的屏级作废未检"
           if trap_ok else "检验题没过")
    for r in out:
        r["session"]["validity"], r["session"]["why"] = validity, why
    return out


def exp006(ans_path, data_build_dir):
    """exp006 scene 盲听页（4 候选）。页面只导出「字母 → slotK」，用对照表换成版本名（同 unblind.py）。"""
    ans = jload(ans_path)
    page_dir = data_build_dir / "pages" / ans["build"]
    key = jload(page_dir / "unblind-key.json")
    files = jload(page_dir / "files.json")["clips"]
    traps = {t["id"]: t for t in key["traps"]}
    out, trap_res = [], []
    for i, t in enumerate(ans["trials"], 1):
        names = traps[t["trap"]]["names"] if t["kind"] == "trap" else key["key"][t["clip"]]

        def mp(s):
            if isinstance(s, str) and s.startswith("slot"):
                return names[int(s[4:])]
            if isinstance(s, int):
                return names[s]
            return s

        bm = t["by_model"]
        slots = t.get("slots", {})
        L2M = {L: mp(v) for L, v in slots.items()}
        fl = files.get(t["clip"], {})
        rec = base_rec("exp006", ans["build"], ans_path, page_dir / "unblind-key.json", t.get("order", i), t)
        rec.update({
            "clip": t["clip"], "data_build": key.get("data_build"),
            "scene": {"fine": fl.get("fine"), "user": fl.get("user"), "device": fl.get("device"),
                      "snr_db": fl.get("snr_db"), "desc": fl.get("noise"), "lang": "en"},
            "systems": sorted(set(names)), "positions": {m: L for L, m in L2M.items()}, "qorder": t.get("qorder"),
            "questions": ["clearer", "natural", "eaten", "usable", "tags"],
            "answers": {
                "clearer": mp(bm["clearer"]), "natural": mp(bm["natural"]), "quietest": None, "pick": None,
                "eaten": [mp(x) for x in bm["eaten"] if x != "none"], "eaten_none": "none" in bm["eaten"],
                "usable": {mp(k): v for k, v in bm["usable"].items()},
                "tags": {mp(k): v for k, v in bm["tags"].items()},
            },
            "note": t.get("note", ""),
            "listen_s": {mp(slots.get(k, k)) if k != "ref" else "ref": round(v, 2) for k, v in (t.get("listen_s") or {}).items()},
            "plays": {mp(slots.get(k, k)) if k != "ref" else "ref": v for k, v in (t.get("plays") or {}).items()},
            "switches": t.get("switches"), "started_at": t.get("started_at"), "answered_at": t.get("answered_at"),
            "session": {"id": f"exp006/{ans['build']}", "validity": None, "why": None,
                        "started": ans.get("started"), "finished": ans.get("finished")},
            "audio": {
                "noisy_sha256": fl.get("versions", {}).get("noisy", {}).get("sha256"),
                "out_sha256": {m: fl.get("versions", {}).get(m, {}).get("sha256") for m in set(names)},
                "gen": {"script": "docs/experiments/exp006-scene-sim/sim.py",
                        "recipes": f"docs/experiments/exp006-scene-sim/builds/{key.get('data_build')}/recipes.snapshot.json",
                        "run": "docs/experiments/exp006-scene-sim/offline_ab.py"},
            },
        })
        if t["kind"] == "trap":
            kind = traps[t["trap"]].get("kind")
            a = rec["answers"]
            ok = (a["clearer"] == "same" and a["natural"] == "same") if kind == "identical" else \
                 (a["clearer"] != "degraded" and a["natural"] != "degraded")
            rec["trap"] = {"type": f"trap-{kind}", "pass": ok, "expect": traps[t["trap"]].get("label")}
            trap_res.append((kind, ok, a))
        out.append(rec)
    ident_fail = any(k == "identical" and not ok for k, ok, _ in trap_res)
    deg = [(ok, a) for k, ok, a in trap_res if k == "degraded"]
    if any(not ok for ok, _ in deg):
        v, why = "void", "劣化检验题选了劣化版"
    elif ident_fail or any(a["natural"] == "same" or a["clearer"] == "same" for _, a in deg):
        v, why = "discounted", "全同检验题没过，或劣化检验题答分不出（rubric 4.2）"
    else:
        v, why = "valid", "两道检验题都过"
    for r in out:
        r["session"]["validity"], r["session"]["why"] = v, why
    return out


def base_rec(exp, build, ans_path, key_path, order, t):
    return {
        "rec_id": None, "exp": exp, "build": build, "trial_order": order,
        "kind": t.get("kind", "real"), "trap": None,
        "source": {"answer_file": rel(ans_path), "key_file": rel(key_path)},
    }


# ---------------------------------------------------------------- 两两比较
def pairs_of(rec):
    """按 rubric 4.3：best-of 里被选中者对同屏其余各记 1 胜；没被选的两两之间无信息；「分不出」= 全部平。"""
    if rec["kind"] != "real" or not rec.get("answers"):
        return []
    a = rec["answers"]
    sys_ = rec["systems"]
    out = []
    shas = (rec.get("audio") or {}).get("out_sha256") or {}

    def emit(dim, x, y, winner, rule, strength):
        same_audio = bool(shas.get(x)) and shas.get(x) == shas.get(y)
        if same_audio:
            # 两份逐字节相同：分出胜负 = 随机（invalid）；判平 = 一致性检验（consistency）。两者都不当偏好用
            rule = rule + "+identical_audio"
            strength = "invalid" if winner != "tie" else "consistency"
        out.append({
            "rec_id": rec["rec_id"], "exp": rec["exp"], "build": rec["build"], "clip": rec["clip"],
            "scene_fine": rec["scene"]["fine"], "scene_user": rec["scene"]["user"],
            "dim": dim, "a": x, "b": y, "winner": winner, "rule": rule, "strength": strength,
            "pos_a": rec.get("positions", {}).get(x), "pos_b": rec.get("positions", {}).get(y),
            "identical_audio": same_audio,
            "session_validity": rec["session"]["validity"],
        })

    def best_of(dim, q):
        v = a.get(q)
        if v is None:
            return
        if v == "same":
            for i, x in enumerate(sys_):
                for y in sys_[i + 1:]:
                    emit(dim, x, y, "tie", "same", "weak")
            return
        if v not in sys_:
            return  # 如 exp005 pick = 原声：候选之间无信息
        for y in sys_:
            if y != v:
                x, yy = sorted([v, y])
                emit(dim, x, yy, v, "best_of", "normal")

    best_of("clearer", "clearer")
    best_of("natural", "natural")
    best_of("quietest", "quietest")
    best_of("meeting", "pick")
    # 开会评级（exp003 / exp006）：等级差 ≥ 1 才算胜负（012 §2.1 冻结规则），同级记平
    if a.get("usable"):
        for i, x in enumerate(sys_):
            for y in sys_[i + 1:]:
                gx, gy = GRADE.get(a["usable"].get(x), -1), GRADE.get(a["usable"].get(y), -1)
                if gx < 0 or gy < 0:
                    continue
                w = "tie" if gx == gy else (x if gx > gy else y)
                emit("meeting", x, y, w, "usable_grade", "normal" if abs(gx - gy) >= 1 else "weak")
    # 吃字：被勾 vs 没被勾
    if a.get("eaten") is not None:
        conflict = a.get("eaten_none") and len(a["eaten"]) > 0
        for i, x in enumerate(sys_):
            for y in sys_[i + 1:]:
                ex, ey = x in a["eaten"], y in a["eaten"]
                w = "tie" if ex == ey else (y if ex else x)
                emit("eaten", x, y, w, "eaten_mark" + ("_conflict" if conflict else ""), "weak" if conflict else "normal")
    # 缺陷标签：可选，只有「标了 vs 没标」才出一对，强度 weak
    if a.get("tags") is not None:
        for i, x in enumerate(sys_):
            for y in sys_[i + 1:]:
                tx, ty = bool(a["tags"].get(x)), bool(a["tags"].get(y))
                if tx != ty:
                    emit("defects", x, y, y if tx else x, "tag_presence", "weak")
    return out


# ---------------------------------------------------------------- 汇总
def summarize(trials, pairs):
    sessions = {}
    for r in trials:
        s = sessions.setdefault(r["session"]["id"], {
            "exp": r["exp"], "build": r["build"], "validity": r["session"]["validity"], "why": r["session"]["why"],
            "screens_real": 0, "screens_trap": 0, "unblindable": 0, "notes": 0, "best_of_letters": {}})
        if r.get("unblindable"):
            s["unblindable"] += 1
            continue
        s["screens_" + ("trap" if r["kind"] == "trap" else "real")] += 1
        if r.get("note", "").strip():
            s["notes"] += 1
        if r["kind"] == "real" and r.get("answers"):
            for q in ("clearer", "natural", "quietest", "pick"):
                v = r["answers"].get(q)
                L = r.get("positions", {}).get(v)
                if L:
                    s["best_of_letters"].setdefault(q, {}).setdefault(L, 0)
                    s["best_of_letters"][q][L] += 1
    by_dim = {}
    for p in pairs:
        d = by_dim.setdefault(p["dim"], {"pairs": 0, "decisive": 0, "tie": 0, "identical_audio": 0, "invalid": 0, "consistency": 0})
        d["consistency"] += p["strength"] == "consistency"
        d["pairs"] += 1
        d["tie" if p["winner"] == "tie" else "decisive"] += 1
        d["identical_audio"] += p["identical_audio"]
        d["invalid"] += p["strength"] == "invalid"
    return {"n_trials": len(trials), "n_pairs": len(pairs), "sessions": sessions, "pairs_by_dim": by_dim,
            "dims": DIMS}


def collect():
    trials = []
    e3 = EXP / "exp003-offline-ab"
    clips3 = {c["id"]: c for c in jload(e3 / "clips.json")["clips"]}
    trials += exp003_partial(e3 / "results/20261030-human-partial-build-20261030102301-68ab.json",
                             e3 / "builds/20261030102301-68ab/recovered-key.json", clips3)
    trials += exp003_full(e3 / "results/20261030-human-build-20261030110755-f8c3.json",
                          e3 / "builds/20261030110755-f8c3", clips3)
    e5 = EXP / "exp005-p2-speaker"
    trials += exp005(e5 / "results/20261031-human-build-20261030224448-ab48.json",
                     e5 / "builds/20261030224448-ab48", e5 / "clips.json")
    # exp006：自动发现答卷（人交回后放 results/）
    e6 = EXP / "exp006-scene-sim"
    for p in sorted((e6 / "results").glob("*.json")) if (e6 / "results").exists() else []:
        if p.name.endswith(".unblinded.json"):
            continue
        data_build = "20261030214530-44df"
        trials += exp006(p, e6 / "builds" / data_build)
    for r in trials:
        r["rec_id"] = f"{r['exp']}/{r['build']}/{r['trial_order']:02d}-{r.get('clip')}"
    pairs = [p for r in trials for p in pairs_of(r)]
    return trials, pairs


def dumps_lines(rows):
    return "".join(json.dumps(r, ensure_ascii=False, sort_keys=True) + "\n" for r in rows)


def main():
    trials, pairs = collect()
    outs = {
        HERE / "trials.jsonl": dumps_lines(trials),
        HERE / "pairs.jsonl": dumps_lines(pairs),
        HERE / "summary.json": json.dumps(summarize(trials, pairs), ensure_ascii=False, indent=1, sort_keys=True) + "\n",
    }
    if "--check" in sys.argv:
        bad = [p.name for p, t in outs.items() if not p.exists() or p.read_text() != t]
        if bad:
            print("与仓里不一致（重跑 build_dataset.py 后提交）：", bad)
            sys.exit(1)
        print("一致")
        return
    for p, t in outs.items():
        p.write_text(t)
    s = summarize(trials, pairs)
    print(f"trials {s['n_trials']} · pairs {s['n_pairs']}")
    for k, v in s["sessions"].items():
        print(f"  {k}: {v['validity']} · 正式 {v['screens_real']} · 检验 {v['screens_trap']} · 无法揭盲 {v['unblindable']} · 备注 {v['notes']}")


if __name__ == "__main__":
    main()
