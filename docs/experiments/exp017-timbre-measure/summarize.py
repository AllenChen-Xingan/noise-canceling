"""exp017：把 readings.csv 压成按劣化分组的中位数 / 95 分位表（Markdown），写到 stdout。

用法：python -I summarize.py <readings.csv>
"""

import sys

import pandas as pd

COLS = ["code_max", "doc_rms", "third_rms", "third_max", "code_noisy_path"]
BANDS = [f"code_b{i}" for i in range(4)] + [f"doc_b{i}" for i in range(4)]


def main(path):
    d = pd.read_csv(path)
    order = list(dict.fromkeys(d.deg))
    g = d.groupby("deg")
    med = g[COLS].median().loc[order]
    p95 = g[["code_max", "doc_rms"]].quantile(0.95).loc[order].add_suffix("_p95")
    t = pd.concat([med, p95], axis=1).round(2)
    print(f"样本：{d.clip.nunique()} 段 × {len(order)} 种劣化\n")
    print("## 合成值（中位数；_p95 = 95 分位）\n")
    print(t.to_markdown())
    print("\n## 各带偏差（中位数，dB）\n")
    print(g[BANDS].median().loc[order].round(1).to_markdown())


if __name__ == "__main__":
    main(sys.argv[1])
