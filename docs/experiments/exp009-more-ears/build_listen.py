"""exp009 · 「大众已评过」的盲听题：给作者听 2 对公开音频，对照众人的配对打分（CCR）。

来源：URGENT 2026 语音增强挑战的主观听测集 urgent-challenge/urgent2026-sqa（CC BY 4.0）的 ccr 部分。
每行 = 同一句话、两个系统的处理结果，众人按 −3…+3 打分，正值表示 audio_a 更好（数据卡原文）。
只按行取了 2 对音频（datasets-server，几百 KB），没下整包。
选题（2026-11-02，ears）：is_simulated = true（照 D62 只听合成声）、中文 1 对 + 英文 1 对、|CMOS| ≥ 1.5、
票的方向全部一致——
  中文 row 8575 ccr_fileid_20_1348_1397：CMOS −1.571，7 票全选 b
  英文 row 1463 ccr_fileid_60_1395_1399：CMOS +1.75，8 票全选 a
音频不做响度统一：保持众人当时听到的样子。

输出（build id = 时间戳 + 4 位随机）：
  _tmp/exp009-listen/<build>/listen/   给人听：index.html + q1_X.wav …（无模型名、无答案）
  _tmp/exp009-listen/<build>/key/      揭盲表 unblind-key.json + 清单 manifest.json（不给人）
  docs/experiments/exp009-more-ears/builds/<build>/  揭盲表与清单的副本（进仓，CLAUDE.md 第五节第 1 条）

用法：<主目录>/_tmp/exp009-venv/bin/python docs/experiments/exp009-more-ears/build_listen.py <build-id>
（<build-id> 目录下 raw/ 要先有 row_*.json 与 4 个 flac；本脚本不联网）
"""
import hashlib
import json
import shutil
import sys
from pathlib import Path

import numpy as np
import soundfile as sf

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
ROWS = [8575, 1463]  # 题序：先中文后英文
SEED = 20261102


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


PAGE = """<!doctype html>
<html lang="zh"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>盲听两题</title>
<style>
:root{--bg:#fafaf8;--fg:#1d1d1b;--mut:#6b6b66;--card:#fff;--line:#deded8;--acc:#2f5fb3}
@media (prefers-color-scheme:dark){:root{--bg:#161615;--fg:#ececea;--mut:#a3a39d;--card:#20201e;--line:#3a3a36;--acc:#8fb0ec}}
body{background:var(--bg);color:var(--fg);font:16px/1.6 -apple-system,"PingFang SC",sans-serif;margin:0;padding:24px 16px}
main{max-width:680px;margin:0 auto}h1{font-size:22px;margin:0 0 4px}p.mut{color:var(--mut);margin:0 0 20px}
section{background:var(--card);border:1px solid var(--line);border-radius:10px;padding:16px;margin:0 0 16px}
.row{display:flex;gap:12px;flex-wrap:wrap}.row>div{flex:1 1 260px}audio{width:100%}
fieldset{border:0;padding:0;margin:12px 0 0}label{display:block;padding:4px 0;cursor:pointer}
textarea{width:100%;box-sizing:border-box;min-height:56px;background:var(--bg);color:var(--fg);border:1px solid var(--line);border-radius:6px;padding:6px}
button{background:var(--acc);color:#fff;border:0;border-radius:6px;padding:10px 18px;font-size:16px;cursor:pointer}
pre{white-space:pre-wrap;word-break:break-all;background:var(--bg);border:1px solid var(--line);padding:8px;border-radius:6px;font-size:13px}
</style></head><body><main>
<h1>盲听两题</h1>
<p class="mut">每题是同一句话的两种降噪处理。请戴耳机，各听至少一遍，选你觉得整体听感更好的一边。没有标准答案；答完点最下面的按钮保存。</p>
__QUESTIONS__
<button id="save">保存答卷</button>
<p class="mut" id="msg"></p><pre id="out" hidden></pre>
</main>
<script>
const BUILD="__BUILD__", QS=__QIDS__;
document.getElementById('save').onclick=()=>{
  const ans={build:BUILD,saved_at:new Date().toISOString(),answers:{}};
  for(const q of QS){
    const v=document.querySelector('input[name="'+q+'"]:checked');
    ans.answers[q]={choice:v?v.value:null,note:document.getElementById(q+'_note').value};
  }
  const txt=JSON.stringify(ans,null,1);
  try{const a=document.createElement('a');a.href=URL.createObjectURL(new Blob([txt],{type:'application/json'}));
      a.download='answers-'+BUILD+'.json';a.click();}catch(e){}
  const o=document.getElementById('out');o.textContent=txt;o.hidden=false;
  document.getElementById('msg').textContent='已下载 answers-'+BUILD+'.json；如果没下载成功，把下面这段文字复制给我也行。';
};
</script></body></html>
"""

CHOICES = [("X++", "X 明显更好"), ("X+", "X 稍好"), ("=", "差不多 / 分不出"), ("Y+", "Y 稍好"), ("Y++", "Y 明显更好")]


def question(qid, n, text):
    opts = "".join(f'<label><input type="radio" name="{qid}" value="{v}"> {t}</label>' for v, t in CHOICES)
    return (f'<section><h2>第 {n} 题</h2><p class="mut">句子：{text}</p><div class="row">'
            f'<div><b>X</b><br><audio controls preload="auto" src="{qid}_X.wav"></audio></div>'
            f'<div><b>Y</b><br><audio controls preload="auto" src="{qid}_Y.wav"></audio></div></div>'
            f'<fieldset>{opts}</fieldset><p class="mut">想说为什么（可空）：</p><textarea id="{qid}_note"></textarea></section>')


def main():
    bid = sys.argv[1]
    W = ROOT / "_tmp/exp009-listen" / bid
    raw, listen, key = W / "raw", W / "listen", W / "key"
    if listen.exists():
        sys.exit(f"{listen} 已存在：不覆盖旧 build，换一个 build id")
    listen.mkdir(parents=True)
    key.mkdir(exist_ok=True)
    rng = np.random.default_rng(SEED + int(bid.split("-")[0][-6:]))
    K, files, qs = {"build": bid, "source": "urgent-challenge/urgent2026-sqa · ccr · CC BY 4.0",
                    "cmos_sign": "正值 = audio_a 更好（数据卡）", "loudness": "未统一，保持众人听到的样子", "questions": {}}, {}, []
    for n, idx in enumerate(ROWS, 1):
        r = json.loads((raw / f"row_{idx}.json").read_text())
        qid = f"q{n}"
        flip = bool(rng.integers(2))  # True：X = b
        side = {"X": "b" if flip else "a", "Y": "a" if flip else "b"}
        for slot, s in side.items():
            src = raw / r[f"audio_{s}"]["file"]
            assert sha(src) == r[f"audio_{s}"]["sha256"]
            x, sr = sf.read(src, dtype="float32")
            out = listen / f"{qid}_{slot}.wav"
            sf.write(out, x, sr, subtype="PCM_16")
            files[out.name] = sha(out)
        crowd_better = "a" if r["cmos"] > 0 else "b"
        K["questions"][qid] = {
            "row_idx": idx, "sample_id": r["sample_id"], "language": r["language"], "text": r["text"],
            "X": {"side": side["X"], "system_id": r[f"system_id_{side['X']}"], "team": r[f"team_name_{side['X']}"]},
            "Y": {"side": side["Y"], "system_id": r[f"system_id_{side['Y']}"], "team": r[f"team_name_{side['Y']}"]},
            "cmos_a_minus_b": r["cmos"], "votes_a_minus_b": r["listener_scores"], "n_votes": len(r["listener_scores"]),
            "crowd_prefers_slot": "X" if side["X"] == crowd_better else "Y",
            "source_sha256": {s: r[f"audio_{s}"]["sha256"] for s in "ab"},
        }
        qs.append(question(qid, n, r["text"]))
    page = PAGE.replace("__QUESTIONS__", "\n".join(qs)).replace("__BUILD__", bid).replace("__QIDS__", json.dumps([f"q{i}" for i in range(1, len(ROWS) + 1)]))
    (listen / "index.html").write_text(page)
    files["index.html"] = sha(listen / "index.html")
    (key / "unblind-key.json").write_text(json.dumps(K, ensure_ascii=False, indent=1))
    (key / "manifest.json").write_text(json.dumps({"build": bid, "listen_files_sha256": files, "seed": SEED}, ensure_ascii=False, indent=1))
    # 给人听的目录里不许有答案：只许 index.html 与 q*_X/Y.wav
    assert sorted(p.name for p in listen.iterdir()) == sorted(files), "listen/ 里混进了别的文件"
    for bad in ["1348", "1397", "1395", "1399", "team", "unblind", "cmos"]:
        assert bad not in page, f"页面里出现了 {bad}"
    D = HERE / "builds" / bid
    D.mkdir(parents=True, exist_ok=False)
    for f in ["unblind-key.json", "manifest.json"]:
        shutil.copyfile(key / f, D / f)
    for f in raw.glob("row_*.json"):
        shutil.copyfile(f, D / f.name)
    print("给人听：", listen)
    print("揭盲表（已复制进仓）：", D)


if __name__ == "__main__":
    main()
