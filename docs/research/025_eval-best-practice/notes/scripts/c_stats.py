"""C 组（调研 025）· exp014 本组五项的分布、配对差与所需段数。

用途：重算 group-C.md 第一节各项的「exp014 实测」数字（中位、第 90 百分位、按信噪比分档），
      以及第二节第 4 条「要多少段」表（配对差均值、标准差、dz、所需段数）。
输入：
  1. run 20261104-000444-e4511abc 的逐段分数 clips.jsonl（480 段 × 3 系统：passthrough / ref / gtcrn）
  2. 测试集清单 exp013 build 20261103230756-17f5 的 clips.jsonl（取 measured.snr_all_db 作信噪比；按 id 对段）
  判「unknown」的段不计入。
用法（在仓库根执行）：
  python3 -I c_stats.py \
    ../noise_canceling-card1/docs/experiments/exp014-scorecard-gtcrn/runs/20261104-000444-e4511abc/clips.jsonl \
    docs/experiments/exp013-scene-suite/builds/20261103230756-17f5/clips.jsonl
方法：所需段数 n ≈ ((1.96 + 0.84) × sd / 效应)²（配对 t 检验正态近似，双侧 α = 0.05，检验力 80%），
      效应取观测均值差与其一半。
"""
import json, sys, math, statistics as st, random
rows=[json.loads(l) for l in open(sys.argv[1])]
man={}
for l in open(sys.argv[2]):
    m=json.loads(l); man[m.get('id') or m.get('clip')]=m
items=['V-PUMPV','V-TIMBRE','V-ONSET','V-WEAK','V-MUSIC','V-SPLVL']
val={}
for r in rows:
    for s in r['scores']:
        if s['v'] in items and s['verdict']!='unknown':
            val[(r['system'],r['clip'],s['v'])]=(s['value'],s['verdict'],s['extra'])
clips=sorted({r['clip'] for r in rows})
def snr(c):
    m=man.get(c) or {}
    return ((m.get('measured') or {}).get('snr_all_db'))
print('manifest hits', sum(snr(c) is not None for c in clips), len(clips))
def q(x,p):
    x=sorted(x); return x[min(len(x)-1,int(p*len(x)))]
for it in items:
    for sy in ['passthrough','ref','gtcrn']:
        v=[val[(sy,c,it)][0] for c in clips if (sy,c,it) in val]
        if v: print(it,sy,'n',len(v),'med %.2f p10 %.2f p90 %.2f mean %.2f sd %.2f'%(q(v,.5),q(v,.1),q(v,.9),st.mean(v),st.pstdev(v)))
    for sy in ['gtcrn','ref']:
        d=[val[(sy,c,it)][0]-val[('passthrough',c,it)][0] for c in clips if (sy,c,it) in val and ('passthrough',c,it) in val]
        if len(d)>2:
            m=st.mean(d); sd=st.stdev(d)
            # n for 80% power alpha .05 two-sided detecting effect = 0.5*|m| and |m|
            for eff in [abs(m), abs(m)/2]:
                n=((1.96+0.84)*sd/eff)**2 if eff>0 else float('nan')
                print('  diff',sy,'n %d mean %.2f sd %.2f  dz %.2f  n_needed(eff=%.2f)=%.0f'%(len(d),m,sd,m/sd,eff,n))
            # by SNR bins for gtcrn
            lo=[x for c,x in zip([c for c in clips if (sy,c,it) in val and ('passthrough',c,it) in val],d) if snr(c) is not None and snr(c)>=10]
            if lo: print('    SNR>=10: n %d mean %.2f sd %.2f'%(len(lo),st.mean(lo),st.stdev(lo) if len(lo)>1 else 0))
    # passthrough by SNR
    for sy in ['passthrough','gtcrn']:
        b={}
        for c in clips:
            if (sy,c,it) in val and snr(c) is not None:
                k='<5' if snr(c)<5 else ('5-15' if snr(c)<15 else '>=15')
                b.setdefault(k,[]).append(val[(sy,c,it)][0])
        print('  bySNR',sy,{k:'med %.2f p90 %.2f n %d'%(q(v,.5),q(v,.9),len(v)) for k,v in sorted(b.items())})
