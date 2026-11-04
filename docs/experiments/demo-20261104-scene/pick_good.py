import json,sys,glob,os
run,keyf=sys.argv[1],sys.argv[2]
blind={i["clip"].rsplit("-",1)[0] for i in json.load(open(keyf))["items"]}
rows=[json.loads(l) for l in open(f"{run}/clips.jsonl")]
def sc(r): return {s["v"]:s for s in r["scores"]}
by={}
for r in rows: by.setdefault(r["clip"],{})[r["system"]]=r
cands=[]
for clip,d in by.items():
    if clip.rsplit("-",1)[0] in blind or "gtcrn" not in d or "passthrough" not in d: continue
    g,p=sc(d["gtcrn"]),sc(d["passthrough"])
    bad=[v for v in ("V-SPLVL","V-EAT-ASR","V-MUSIC","V-CLICK","V-ONSET") if g.get(v,{}).get("verdict")=="fail"]
    if bad: continue
    nr=g.get("V-NR",{}).get("value"); nrp=p.get("V-NR",{}).get("value")
    q=g.get("V-NRMOS-SCOREQ",{}); dq=q.get("value")
    ds=g.get("V-NRMOS-DISTILL",{}).get("value")
    if None in (nr,nrp,dq,ds) or dq<=0 or ds<=0: continue
    cands.append(dict(clip=clip,scene=d["gtcrn"]["scene"],tier=d["gtcrn"]["tier"],device=d["gtcrn"]["device"],dnr=nr-nrp,dscoreq=dq,ddistill=ds,splvl=g["V-SPLVL"]["value"],sounds=d["gtcrn"].get("sounds")))
cands.sort(key=lambda c:(c["dnr"]-3*c["dscoreq"]))
seen=set();out=[]
for c in cands:
    if c["scene"] in seen: continue
    seen.add(c["scene"]);out.append(c)
    if len(out)==6: break
print(len(cands),"candidates")
for c in out: print(json.dumps(c,ensure_ascii=False))
