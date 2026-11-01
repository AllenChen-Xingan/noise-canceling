"""从人耳偏好数据集生成 eval 任务清单（vdn-eval 格式，见 evalimpl 的 docs/experiments/exp007-eval-suite/task-schema.md）。

只做数据整理：任务的「人判结论」从 trials.jsonl 自动取（不手抄），过线值只用 verifier 注册表锚点或已有实测值，
两者都标出处。暂不改 src/eval/（evalimpl 合并后由它加载本文件，或让 parse_suite 读 tasks/）。

输出：tasks/human-tasks.json（vdn-eval 现在就能解析、能跑的）· tasks/human-tasks-blocked.json（同格式，但 vdn-eval 还跑不了，
  每个任务带 blocked 字段说明缺什么；补上能力后挪回主清单）
用法：python3 docs/experiments/human-prefs/make_tasks.py [--check] [--registry <verifiers.rs>]
  --check     只比对，不写
  --registry  verifier 注册表；不给时依次找 src/eval/src/verifiers.rs、`git show origin/main:src/eval/src/verifiers.rs`，都没有就报错退出
校验（每次都跑，任一不过退出码 1）：id 唯一（含 src/eval/suite/*.json，同样回退到 origin/main）· set / polarity 取值 ·
  origin 非空 · real.kind ∈ exp003/exp005 · graders / info 的 v 都在注册表里 ·
  可运行性：exp003 片段必须在 clips.json 里有 noisy_src / clean_src（vdn-eval runner 只读现成片段），否则进 blocked
注意：V-EAT-ASR / V-DNSMOS 是 qa 层 verifier，这批任务应并进 `vdn-eval qa` 跑（ci 层对它们只会判「不知道」）
"""
import re
import json
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT = HERE / "tasks" / "human-tasks.json"
OUT_BLOCKED = HERE / "tasks" / "human-tasks-blocked.json"
REPO = HERE.parent.parent.parent

E3 = {"kind": "exp003", "clips": "docs/experiments/exp003-offline-ab/clips.json"}
E5 = {"kind": "exp005", "clips": "docs/experiments/exp005-p2-speaker/clips.json", "audio_dir": "_tmp/20261031-exp005/src"}

# 系统名：答卷里的名字 → vdn-eval --system 名（没有对应的记 None：现在的 vdn-eval 跑不了）
SYS = {"gtcrn": "gtcrn", "fastenhancer_m": "fastenhancer-m", "dpdfnet2": None,
       "raw": "gtcrn", "personal": None, "aggressive": None}

# 锚点出处（verifier 注册表 src/eval/src/verifiers.rs 的 anchor 字段，全部〔建议值〕）
A = "verifier 注册表锚点〔建议值〕"
# exp005 README §3 实测（锁定后、只有邻座在说的样本，10·log10(Σ输出² / Σ输入邻座²)；口径接近 V-INTF 但不相同）
E5_RESID = {"c01": {"raw": -1.9, "personal": -3.8, "aggressive": -6.5},
            "c02": {"raw": -2.3, "personal": -0.9, "aggressive": -2.6},
            "c03": {"raw": -4.3, "personal": -1.4, "aggressive": -5.4},
            "c04": {"raw": -5.4, "personal": -2.7, "aggressive": -7.2},
            "c05": {"raw": -5.6, "personal": -3.4, "aggressive": -8.3},
            "c07": {"raw": -0.6, "personal": -0.4, "aggressive": -0.9}}
E5_RESID_SRC = "docs/experiments/exp005-p2-speaker/README.md §3 表 1（整段锁定后，非听测节选）"

# 任务规格：(id, set, polarity, exp, clip, 被判的系统, desc, graders, info, xfail, threshold_basis)
SPECS = [
    # ---------- 回归：人判「能用 / 最自然」的输出，不许变差 ----------
    ("R60-h-e003-c09-clatter-clean", "regression", "both", "exp003", "c09", ["dpdfnet2", "gtcrn", "fastenhancer_m"],
     "杯碟水声、无旁人：三家人都判能用，DPDFNet-2 最好（停顿里压 −72 dB 仍被说「非常干净」）",
     [{"v": "V-EAT-ASR", "max": 1.0}, {"v": "V-SPLVL", "min": -3}, {"v": "V-ONSET", "max": 3}],
     [{"v": "V-NR"}, {"v": "V-VACUUM"}, {"v": "V-MUSIC"}, {"v": "V-DNSMOS"}], None,
     A + "；V-VACUUM 只报：本题是它的反例（深压 −72 dB 人说干净）"),
    ("R61-h-e003-c10-music-fem", "regression", "both", "exp003", "c10", ["fastenhancer_m", "dpdfnet2"],
     "背景钢琴：FE-M 被选最清楚 / 最自然、能用；DPDFNet-2 能用；GTCRN 勉强并被标背景忽大忽小",
     [{"v": "V-EAT-ASR", "max": 1.0}, {"v": "V-SPLVL", "min": -3}],
     [{"v": "V-NR"}, {"v": "V-MUSIC"}, {"v": "V-DNSMOS"}], None, A),
    ("R62-h-e003-c08-babble10-gtcrn", "regression", "preserve", "exp003", "c08", ["gtcrn"],
     "邻桌 4 人 10 dB：GTCRN 最清楚 / 最自然、能用、无缺陷标签；另两家被打 4–5 个缺陷标签",
     [{"v": "V-EAT-ASR", "max": 1.0}, {"v": "V-SPLVL", "min": -3}, {"v": "V-ONSET", "max": 3}, {"v": "V-TIMBRE", "max": 6}],
     [{"v": "V-MUSIC"}, {"v": "V-VACUUM"}, {"v": "V-DNSMOS"}], None, A),
    ("R63-h-e003-c01-casino5-fem", "regression", "both", "exp003", "c01", ["fastenhancer_m"],
     "赌场 5 dB：FE-M 最清楚 / 最自然、能用、没被勾吃字；另两家勉强且被勾吃字",
     [{"v": "V-EAT-ASR", "max": 1.0}, {"v": "V-SPLVL", "min": -3}, {"v": "V-ONSET", "max": 3}],
     [{"v": "V-NR"}, {"v": "V-DNSMOS"}], None, A),
    ("R64-h-e005-c06-owner-cafe-bg", "regression", "preserve", "exp005", "c06", ["personal"],
     "只有主人 + 咖啡馆底噪：保守版最自然、开会选它，人原话要「有一点点 bg」——底噪不许压成死寂",
     [{"v": "V-SPLVL", "min": -3}, {"v": "V-ONSET", "max": 3}, {"v": "V-SIM-GATE", "max": 0.05, "onset_max": 0.2}],
     [{"v": "V-VACUUM"}, {"v": "V-NR"}, {"v": "V-EAT-ASR"}, {"v": "V-DNSMOS"}], None,
     A + "；「留一点背景」现在没有 verifier 能判（V-VACUUM 只报深压），见 preferences.md 未知项"),
    ("R65-h-e005-c04-neighbor-first-keep-bg", "regression", "preserve", "exp005", "c04", ["personal"],
     "邻座先说 25 s：保守版最自然、开会选它；人原话「喜欢带着一点点背景声」",
     [{"v": "V-SPLVL", "min": -3}, {"v": "V-ONSET", "max": 3}],
     [{"v": "V-VACUUM"}, {"v": "V-INTF"}, {"v": "V-EAT-ASR"}], None, A),
    ("R66-h-e005-c02-neighbor-loud-keep-reverb", "regression", "preserve", "exp005", "c02", ["personal"],
     "邻座只轻 3 dB：保守版最自然、开会选它；人原话「留下来一点点混响」；激进版被勾吃字",
     [{"v": "V-SPLVL", "min": -3}, {"v": "V-ONSET", "max": 3}, {"v": "V-SIM-GATE", "max": 0.05, "onset_max": 0.2}],
     [{"v": "V-INTF"}, {"v": "V-TIMBRE"}, {"v": "V-EAT-ASR"}], None, A),
    ("R67-h-e005-c01-neighbor-aggr", "regression", "suppress", "exp005", "c01", ["aggressive"],
     "邻座轻 6 dB：激进版最自然、开会选它、没被勾吃字（邻座残留实测 −6.5 dB）——压邻座的力度不许比这更弱",
     [{"v": "V-INTF", "max": -6.5}, {"v": "V-SPLVL", "min": -3}],
     [{"v": "V-ONSET"}, {"v": "V-EAT-ASR"}], None,
     "V-INTF −6.5 取自 " + E5_RESID_SRC + "（激进版实测；口径不同，接入后用 V-INTF 重量一次再定）"),
    # ---------- 能力：人判「不能用 / 吃字 / 别人太吵」，起点应失败 ----------
    ("C70-h-e003-c03-casino-m5", "capability", "both", "exp003", "c03", ["dpdfnet2", "gtcrn", "fastenhancer_m"],
     "赌场 −5 dB：三家都判不能用、都被勾吃字、最清楚分不出",
     [{"v": "V-EAT-ASR", "max": 1.0}, {"v": "V-SPLVL", "min": -3}], [{"v": "V-NR"}, {"v": "V-DNSMOS"}],
     "人耳：三家全「不能用」（exp003 f8c3 第 1 屏）", A),
    ("C71-h-e003-c07-babble6-m5", "capability", "both", "exp003", "c07", ["dpdfnet2", "gtcrn", "fastenhancer_m"],
     "邻桌 6 人 −5 dB：三家都不能用、都被勾吃字",
     [{"v": "V-EAT-ASR", "max": 1.0}, {"v": "V-SPLVL", "min": -3}, {"v": "V-ONSET", "max": 3}], [{"v": "V-NR"}, {"v": "V-DNSMOS"}],
     "人耳：三家全「不能用」（exp003 f8c3 第 5 屏）", A),
    ("C72-h-e003-c11-zh-babble3", "capability", "both", "exp003", "c11", ["dpdfnet2", "gtcrn", "fastenhancer_m"],
     "中文目标 + 英文邻桌 3 人：三家都不能用；人原话「都不太清楚」",
     [{"v": "V-EAT-ASR", "max": 1.0}, {"v": "V-SPLVL", "min": -3}], [{"v": "V-NR"}, {"v": "V-DNSMOS"}],
     "人耳：三家全「不能用」（exp003 f8c3 第 12 屏）", A),
    ("C73-h-e003-c12-zh-neighbor", "capability", "suppress", "exp003", "c12", ["dpdfnet2", "gtcrn", "fastenhancer_m"],
     "中文目标 + 英文邻座：三家都不能用；GTCRN 相对最好；ASR 转写里带出邻座的话",
     [{"v": "V-EAT-ASR", "max": 1.0}, {"v": "V-SPLVL", "min": -3}], [{"v": "V-NR"}, {"v": "V-DNSMOS"}],
     "人耳：三家全「不能用」（exp003 f8c3 第 9 屏）", A + "；压邻座要三轨（exp003 没分轨）→ V-INTF 现在算不了"),
    ("C74-h-e003-c06-babble3-0db", "capability", "both", "exp003", "c06", ["dpdfnet2", "gtcrn", "fastenhancer_m"],
     "邻桌 3 人 0 dB：GTCRN 勉强、另两家不能用；三家都被勾吃字（ASR 也说 D / F 丢 “bannister in the”）",
     [{"v": "V-EAT-ASR", "max": 1.0}, {"v": "V-ONSET", "max": 3}, {"v": "V-SPLVL", "min": -3}], [{"v": "V-NR"}, {"v": "V-DNSMOS"}],
     "人耳：最好的也只「勉强」、全部被勾吃字（exp003 f8c3 第 8 屏）", A),
    ("C75-h-e003-c05-neighbor-0db", "capability", "suppress", "exp003", "c05", ["dpdfnet2", "gtcrn", "fastenhancer_m"],
     "邻座 1 人 0 dB：GTCRN 勉强、另两家不能用且被勾吃字（机器分偏 DPDFNet-2，人耳相反）",
     [{"v": "V-EAT-ASR", "max": 1.0}, {"v": "V-SPLVL", "min": -3}], [{"v": "V-NR"}, {"v": "V-DNSMOS"}],
     "人耳：最好的也只「勉强」（exp003 f8c3 第 4 屏）", A),
    ("C76-h-e003-c04-neighbor-5db", "capability", "suppress", "exp003", "c04", ["dpdfnet2", "gtcrn", "fastenhancer_m"],
     "邻座 1 人 5 dB：GTCRN 勉强、另两家不能用；三家都被勾吃字",
     [{"v": "V-EAT-ASR", "max": 1.0}, {"v": "V-SPLVL", "min": -3}], [{"v": "V-NR"}, {"v": "V-DNSMOS"}],
     "人耳：最好的也只「勉强」、全部被勾吃字（exp003 f8c3 第 6 屏）", A),
    ("C77-h-e003-c02-casino-0db", "capability", "both", "exp003", "c02", ["dpdfnet2", "gtcrn", "fastenhancer_m"],
     "赌场 0 dB：FE-M 勉强、另两家不能用；三家都被勾吃字",
     [{"v": "V-EAT-ASR", "max": 1.0}, {"v": "V-SPLVL", "min": -3}], [{"v": "V-NR"}, {"v": "V-DNSMOS"}],
     "人耳：最好的也只「勉强」、全部被勾吃字（exp003 f8c3 第 13 屏）", A),
    ("C78-h-e005-c03-neighbors3-too-loud", "capability", "suppress", "exp005", "c03", ["personal", "raw", "aggressive"],
     "邻桌 3 人：人原话「c（保守版）别人说太吵了」，开会选了只有 GTCRN",
     [{"v": "V-INTF", "max": -10}, {"v": "V-SPLVL", "min": -3}], [{"v": "V-EAT-ASR"}, {"v": "V-ONSET"}],
     "实测邻座残留 保守 −1.4 / GTCRN −4.3 / 激进 −5.4 dB，都没到 −10（" + E5_RESID_SRC + "）", A),
    ("C79-h-e005-c08-neighbors4-nolock", "capability", "suppress", "exp005", "c08", ["personal", "raw"],
     "邻桌 4 人、认人没锁住：人原话「b（保守版）别人的声音太吵了，c 虽然有点不自然，但是可以开会」",
     [{"v": "V-INTF", "max": -10}, {"v": "V-SPLVL", "min": -3}], [{"v": "V-EAT-ASR"}, {"v": "V-DNSMOS"}],
     "c08 认人没锁（exp005 README §2），门控不收；人判保守版太吵", A),
    ("C80-h-e005-c05-neighbor-first-quieter", "capability", "suppress", "exp005", "c05", ["aggressive", "personal", "raw"],
     "邻座先说 45 s：人原话「本来周围就是比较安静的情况，那就比较安静会更好」；开会选激进版，但激进版被勾吃字",
     [{"v": "V-INTF", "max": -10}, {"v": "V-ONSET", "max": 3}, {"v": "V-SIM-GATE", "max": 0.05, "onset_max": 0.2}],
     [{"v": "V-NR"}, {"v": "V-EAT-ASR"}],
     "实测邻座残留 激进 −8.3 dB 没到 −10；激进版句首 2 s 内误压 13.7%（exp005 README §3）；人勾了激进版吃字", A),
    ("C81-h-e005-c07-no-bg-quieter", "capability", "suppress", "exp005", "c07", ["aggressive", "personal", "raw"],
     "邻座、无底噪：人原话「这种还是安静一点比较好」；开会选激进版，但 GTCRN 与激进版都被勾吃字",
     [{"v": "V-INTF", "max": -10}, {"v": "V-ONSET", "max": 3}, {"v": "V-SIM-GATE", "max": 0.05, "onset_max": 0.2}],
     [{"v": "V-EAT-ASR"}],
     "实测邻座残留 三版都在 −0.4 至 −0.9 dB（门几乎不收，原因 exp005 没查）", A),
]

DIM_Q = {"exp003": ("clearer", "natural", "usable"), "exp005": ("natural", "pick", "quietest")}


def load_trials():
    rows = [json.loads(l) for l in (HERE / "trials.jsonl").read_text().splitlines()]
    return [r for r in rows if r["kind"] == "real" and r.get("answers")]


def human_block(trials, exp, clip, judged):
    # 只取有效或打折会话里、该片段的所有正式屏（一般一屏；旧 build 68ab 未验证会话也列出但标出）
    rs = [r for r in trials if r["exp"] == exp and r["clip"] == clip]
    screens = []
    for r in rs:
        a = r["answers"]
        screens.append({
            "rec_id": r["rec_id"], "session_validity": r["session"]["validity"],
            "best": {q: a.get(q) for q in ("clearer", "natural", "quietest", "pick") if a.get(q) is not None},
            "usable": a.get("usable"), "eaten": a.get("eaten"), "eaten_none": a.get("eaten_none"),
            "tags": a.get("tags") or None, "note": r.get("note") or None,
            "positions": r.get("positions"),
            "audio_out_sha256": {m: (r.get("audio") or {}).get("out_sha256", {}).get(m) for m in judged},
            "excerpt_s": r.get("excerpt_s"),
        })
    return {"judged_systems": judged,
            "eval_systems": {m: SYS.get(m) for m in judged},
            "screens": screens,
            "evidence": f"{len(screens)} 屏 · 1 人",
            "confidence": "很低（一人一屏、合成素材；exp003 会话打折、有位置偏好）" if exp == "exp003"
                          else "很低（一人一屏、合成素材、25 s 节选）"}


def build():
    trials = load_trials()
    tasks = []
    for (tid, set_, pol, exp, clip, judged, desc, graders, info, xfail, basis) in SPECS:
        real = dict(E3 if exp == "exp003" else E5)
        real["clip"] = clip
        hb = human_block(trials, exp, clip, judged)
        notes = [s["note"] for s in hb["screens"] if s["note"]]
        origin = f"{exp} {clip} 人耳盲听（{', '.join(s['rec_id'] for s in hb['screens'])}）" + \
                 (f" · 人原话「{notes[0].strip()}」" if notes else "")
        t = {"id": tid, "set": set_, "polarity": pol, "origin": origin, "desc": desc, "real": real,
             "graders": graders, "info": info, "lang": "zh" if clip in ("c11", "c12") and exp == "exp003" else "en",
             "human": hb, "threshold_basis": basis,
             "repro": repro(exp, clip)}
        if xfail:
            t["xfail"] = xfail
        if exp == "exp005":
            t["human"]["caveat"] = ("人听的是锁定之后约 25 s 节选（excerpt_s）；vdn-eval 跑整段 80–120 s。"
                                    "被判的保守版 / 激进版是 vdn-speaker-lab 的个性化管线，现在 vdn-eval 的 --system 里没有，"
                                    "接入前只能拿 GTCRN（= raw）跑")
        why = blocked_reason(exp, clip)
        if why:
            t["blocked"] = why
        tasks.append(t)
    return {
        "name": "human-prefs",
        "doc": ("人耳盲听判过的片段 → 任务。回归 = 人判能用 / 最自然的，不许变差；能力 = 人判不能用 / 吃字 / 别人太吵的，"
                "起点应失败（xfail）。证据全是一个人、合成素材、每题一屏，过线值是 verifier 锚点〔建议值〕或已有实测，"
                "不是人定的线。human 字段由 make_tasks.py 从 trials.jsonl 生成，别手改。"),
        "seed": 20261031,
        "defaults": {"trials": 1},
        "tasks": tasks,
    }


def blocked_reason(exp, clip):
    if exp != "exp003":
        return None
    c = {x["id"]: x for x in json.loads((REPO / E3["clips"]).read_text())["clips"]}[clip]
    if not (c.get("noisy_src") and c.get("clean_src")):
        return (f"exp003 {clip} 是合成片段（kind = {c.get('kind')}），clips.json 没有 noisy_src / clean_src；"
                "vdn-eval 的 Exp003 来源只读现成片段。要跑：给 vdn-eval 加合成来源（按 make_clips.py 种子 20261030 重建），"
                "或把合成片段落盘并在 clips.json 补路径与哈希")
    return None


def split(d):
    main = dict(d, tasks=[t for t in d["tasks"] if "blocked" not in t])
    blk = dict(d, name="human-prefs-blocked",
               doc="与 human-tasks.json 同格式，但 vdn-eval 现在跑不了（见每个任务的 blocked）。补上能力后挪回主清单。",
               tasks=[t for t in d["tasks"] if "blocked" in t])
    return main, blk


def git_show(path):
    try:
        return subprocess.run(["git", "-C", str(REPO), "show", f"origin/main:{path}"], capture_output=True,
                              text=True, check=True).stdout
    except (subprocess.CalledProcessError, FileNotFoundError):
        return None


def read_repo_or_main(path):
    f = REPO / path
    if f.exists():
        return f.read_text(), str(f.relative_to(REPO))
    t = git_show(path)
    return (t, f"origin/main:{path}") if t is not None else (None, None)


def repro(exp, clip):
    if exp == "exp003":
        return {"inputs": "docs/experiments/exp003-offline-ab/clips.json（c01–c03 现成：noisy_sha256 / clean_sha256；"
                          "c04–c12 合成：make_clips.py，种子 20261030，来源哈希 source_sha256）",
                "outputs": "docs/experiments/exp003-offline-ab/enhance.py（逐帧流式，同 build 20261030110755-f8c3）",
                "gap": "被判的那几份输出音频本机已不在、也没记哈希；重跑 enhance.py 得到的输出与人听的是否逐字节相同，未核"}
    return {"inputs": "docs/experiments/exp005-p2-speaker/make_clips.py → clips.json（timeline、params、哈希）",
            "outputs": "cargo build --release -p vdn-speaker-lab；score.py run（参数见 builds/20261030224448-ab48/unblind-key.json 的 cands）",
            "judged_audio": "builds/20261030224448-ab48/files.json 有被听的每份节选的 sha256（human.screens[].audio_out_sha256）"}


def validate(d):
    errs = []
    if "--registry" in sys.argv:
        rp = Path(sys.argv[sys.argv.index("--registry") + 1])
        reg_text, reg_src = (rp.read_text(), str(rp)) if rp.exists() else (None, None)
    else:
        reg_text, reg_src = read_repo_or_main("src/eval/src/verifiers.rs")
    if reg_text is None:
        print("校验失败：找不到 verifier 注册表（src/eval/src/verifiers.rs 与 origin/main 都没有；可用 --registry 指定）")
        sys.exit(1)
    reg = set(re.findall(r'id: "(V-[A-Z0-9-]+)"', reg_text))
    other = []
    for name in ("ci.json", "qa.json"):
        txt, _ = read_repo_or_main(f"src/eval/suite/{name}")
        if txt is None:
            errs.append(f"找不到 src/eval/suite/{name}（无法查 id 冲突）")
            continue
        other += [t["id"] for t in json.loads(txt).get("tasks", [])]
    ids = [t["id"] for t in d["tasks"]]
    dup = {i for i in ids + other if (ids + other).count(i) > 1}
    if dup:
        errs.append(f"id 重复：{sorted(dup)}")
    for t in d["tasks"]:
        if t["set"] not in ("regression", "capability", "probe"):
            errs.append(f"{t['id']} set={t['set']}")
        if t.get("polarity") not in (None, "suppress", "preserve", "both", "robust"):
            errs.append(f"{t['id']} polarity={t['polarity']}")
        if not t.get("origin"):
            errs.append(f"{t['id']} 缺 origin")
        if t["real"]["kind"] not in ("exp003", "exp005"):
            errs.append(f"{t['id']} real.kind={t['real']['kind']}（vdn-eval 只认 exp003 / exp005）")
        if not t["human"]["screens"]:
            errs.append(f"{t['id']} 没有人判屏")
        for g in t["graders"] + t.get("info", []):
            if g["v"] not in reg:
                errs.append(f"{t['id']} verifier {g['v']} 不在注册表")
    if errs:
        print("校验失败：\n  " + "\n  ".join(errs))
        sys.exit(1)
    print(f"校验通过（注册表 {reg_src}：{len(reg)} 个 verifier；对照 ci / qa 清单 {len(other)} 个任务 id）")


def main():
    d = build()
    validate(d)
    main_d, blk = split(d)
    outs = {OUT: json.dumps(main_d, ensure_ascii=False, indent=1) + "\n",
            OUT_BLOCKED: json.dumps(blk, ensure_ascii=False, indent=1) + "\n"}
    if "--check" in sys.argv:
        bad = [p.name for p, t in outs.items() if not p.exists() or p.read_text() != t]
        print("一致" if not bad else f"与仓里不一致（重跑 make_tasks.py）：{bad}")
        sys.exit(0 if not bad else 1)
    for p, t in outs.items():
        p.write_text(t)
    cnt = lambda x, s: sum(t["set"] == s for t in x["tasks"])
    print(f"能跑 {len(main_d['tasks'])} 个（回归 {cnt(main_d, 'regression')} · 能力 {cnt(main_d, 'capability')}）· "
          f"暂跑不了 {len(blk['tasks'])} 个（回归 {cnt(blk, 'regression')} · 能力 {cnt(blk, 'capability')}）")


if __name__ == "__main__":
    main()
