import re, statistics as st, sys, os
d=sys.argv[1]
rows=[]
for name in ['baseline','none','tc','tc_wg']:
    t=open(os.path.join(d,name+'.txt')).read()
    cpu=[int(x) for x in re.findall(r'^CPU Power: (\d+) mW',t,re.M)]
    ef=[int(x) for x in re.findall(r'^E-Cluster HW active frequency: (\d+) MHz',t,re.M)]
    er=[float(x) for x in re.findall(r'^E-Cluster HW active residency:\s+([\d.]+)%',t,re.M)]
    sf=[int(x) for x in re.findall(r'^S-Cluster HW active frequency: (\d+) MHz',t,re.M)]
    sr=[float(x) for x in re.findall(r'^S-Cluster HW active residency:\s+([\d.]+)%',t,re.M)]
    f=lambda v: f"{st.median(v):.0f} / {st.mean(v):.0f}" if v else "?"
    g=lambda v: f"{st.median(v):.1f} / {st.mean(v):.1f}" if v else "?"
    rows.append((name,len(cpu),f(cpu),f(ef),g(er),f(sf),g(sr)))
print("| 设置 | 采样数 | CPU 功耗 mW 中位 / 平均 | 能效簇频率 MHz 中位 / 平均 | 能效簇活跃 % 中位 / 平均 | 性能簇频率 MHz 中位 / 平均 | 性能簇活跃 % 中位 / 平均 |")
print("|---|---|---|---|---|---|---|")
for r in rows: print("| "+" | ".join(str(x) for x in r)+" |")
