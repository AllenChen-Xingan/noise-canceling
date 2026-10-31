import sys, collections
def varint(b,i):
    r=0;s=0
    while True:
        c=b[i];i+=1;r|=(c&0x7f)<<s;s+=7
        if c<0x80: return r,i
def fields(b):
    i=0;n=len(b)
    while i<n:
        k,i=varint(b,i);f=k>>3;t=k&7
        if t==0: v,i=varint(b,i)
        elif t==1: v=b[i:i+8];i+=8
        elif t==5: v=b[i:i+4];i+=4
        elif t==2:
            l,i=varint(b,i);v=b[i:i+l];i+=l
        else: raise ValueError(t)
        yield f,t,v
def graph_ops(g,cnt,depth=0):
    for f,t,v in fields(g):
        if f==1 and t==2:
            op=None;dom=b''
            for nf,nt,nv in fields(v):
                if nf==4: op=nv.decode()
                if nf==7: dom=nv
                if nf==5: # attribute -> may contain subgraph (field 6 g, 11 graphs)
                    for af,at,av in fields(nv):
                        if af==6 and at==2: graph_ops(av,cnt,depth+1)
            cnt[(dom.decode() or 'ai.onnx',op)]+=1
def vinfo(g,fid):
    out=[]
    for f,t,v in fields(g):
        if f==fid and t==2:
            for vf,vt,vv in fields(v):
                if vf==1: out.append(vv.decode())
    return out
b=open(sys.argv[1],'rb').read()
cnt=collections.Counter()
for f,t,v in fields(b):
    if f==8 and t==2:
        d=b'';ver=None
        for of,ot,ov in fields(v):
            if of==1: d=ov
            if of==2: ver=ov
        print('opset',d.decode() or 'ai.onnx',ver)
    if f==7 and t==2:
        graph_ops(v,cnt)
        print('inputs',vinfo(v,11)); print('outputs',vinfo(v,12))
for k,c in sorted(cnt.items()): print(k,c)
