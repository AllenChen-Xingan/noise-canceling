import json,sys,shutil,hashlib
from pathlib import Path
wavs,out,man=Path(sys.argv[1]),Path(sys.argv[2]),Path(sys.argv[3])
assert not out.exists(), "目标已存在，不覆盖"
out.mkdir(parents=True)
picks=[("phone-booth-c09-mba","电话亭","好"),("meeting-room-c04-mba","会议室","好"),("home-quiet-c06-mba","安静的家","好"),
       ("hotel-room-c05-ext","酒店房间","好"),("library-classroom-c05-ext","图书馆 / 教室","好"),("bus-metro-cabin-c03-ext","公交地铁车厢","好"),
       ("cafe-c03-ext","咖啡馆","典型"),("transit-hub-c02-ext","车站候车厅","典型")]
rows=[]
for i,(clip,name,kind) in enumerate(picks,1):
    for tag,lab in (("mix","1原声"),("gtcrn","2降噪后"),("clean","3干净参考")):
        h=sorted(wavs.glob(f"{clip}#*-{tag}-*.wav")); assert len(h)==1,(clip,tag,len(h))
        dst=out/f"{i:02d}-{name.replace(' / ','-')}-{lab}.wav"
        shutil.copy2(h[0],dst)
        rows.append(dict(n=i,clip=clip,kind=kind,file=dst.name,src=h[0].name,sha256=hashlib.sha256(dst.read_bytes()).hexdigest()))
man.write_text(json.dumps(rows,ensure_ascii=False,indent=1))
print(len(rows),"files")
