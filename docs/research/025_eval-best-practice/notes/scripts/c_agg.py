"""C 组（调研 025）· exp014 汇总口径对比：「任一段不过」vs「组合均值」vs「组合中位数」。

用途：重算 group-C.md 第二节第 3 条那张表（每项每系统的不过组合数、不过段 / 判得了的段）。
输入：run 20261104-000444-e4511abc 的逐段分数 clips.jsonl；组合 = 场景 × 档 × 设备；判「unknown」的段不计入。
      过线值取自 src/eval/src/thresholds/table.rs 的默认 max：PUMPV 1 · TIMBRE 6 · ONSET 3 · MUSIC 2（越大越差）。
用法（在仓库根执行）：
  python3 -I c_agg.py \
    ../noise_canceling-card1/docs/experiments/exp014-scorecard-gtcrn/runs/20261104-000444-e4511abc/clips.jsonl
"""
import json,sys,statistics as st
rows=[json.loads(l) for l in open(sys.argv[1])]
th={'V-PUMPV':1.0,'V-TIMBRE':6.0,'V-ONSET':3.0,'V-MUSIC':2.0}
for it,t in th.items():
  for sy in ['passthrough','gtcrn']:
    g={}
    for r in rows:
      if r['system']!=sy: continue
      for s in r['scores']:
        if s['v']==it and s['verdict']!='unknown':
          g.setdefault((r['scene'],r['tier'],r['device']),[]).append(s['value'])
    worst=sum(any(v>t for v in vs) for vs in g.values())
    mean=sum(st.mean(vs)>t for vs in g.values())
    med=sum(st.median(vs)>t for vs in g.values())
    seg=sum(v>t for vs in g.values() for v in vs); n=sum(len(vs) for vs in g.values())
    print(it,sy,'combos',len(g),'worst-rule fail',worst,'mean-rule',mean,'median-rule',med,'seg fail %d/%d'%(seg,n), 'seg/combo med',st.median([len(v) for v in g.values()]))
