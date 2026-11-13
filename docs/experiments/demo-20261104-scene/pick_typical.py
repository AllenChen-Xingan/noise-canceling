import json,sys
run,keyf=sys.argv[1],sys.argv[2]
blind={i["clip"].rsplit("-",1)[0] for i in json.load(open(keyf))["items"]}
rows=[json.loads(l) for l in open(f"{run}/clips.jsonl")]
by={}
for r in rows: by.setdefault(r["clip"],{})[r["system"]]=r
def v(r,k):
    for s in r["scores"]:
        if s["v"]==k: return s.get("value"),s.get("verdict")
    return None,None
for scene in ("cafe","transit-hub"):
    c=[]
    for clip,d in by.items():
        if clip.rsplit("-",1)[0] in blind or d["gtcrn"]["scene"]!=scene or d["gtcrn"]["tier"]!="common": continue
        a,_=v(d["gtcrn"],"V-NR"); b,_=v(d["passthrough"],"V-NR")
        if a is None or b is None: continue
        c.append((a-b,clip,{k:v(d["gtcrn"],k) for k in ("V-SPLVL","V-EAT-ASR","V-NRMOS-SCOREQ","V-NRMOS-DISTILL","V-MUSIC")}))
    c.sort(); m=c[len(c)//2]
    print(json.dumps(dict(scene=scene,clip=m[1],dnr=m[0],n=len(c),scores=m[2]),ensure_ascii=False))
