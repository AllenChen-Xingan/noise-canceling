"""把 GTCRN 流式模型（DNS3 权重，与 gtcrn_simple.onnx 同源）的 state_dict 导成扁平 f32：
<out>.bin（小端 f32 依次拼接）+ <out>.txt（每行：名字 形状(逗号分隔) 偏移(元素) 个数）。
只读权重，不跑推理。用法：python dump_gtcrn.py <resources/gtcrn/stream 目录> <输出前缀>"""
import sys, types
import numpy as np

src, out = sys.argv[1], sys.argv[2]
sys.path.insert(0, src)
sys.modules.setdefault("einops", types.SimpleNamespace(rearrange=None))  # 源码 import 了但没用
import torch
from gtcrn import GTCRN
from gtcrn_stream import StreamGTCRN
from modules.convert import convert_to_stream

model = GTCRN().eval()
model.load_state_dict(torch.load(f"{src}/onnx_models/model_trained_on_dns3.tar", map_location="cpu")["model"])
stream = StreamGTCRN().eval()
convert_to_stream(stream, model)

off = 0
with open(out + ".bin", "wb") as fb, open(out + ".txt", "w") as ft:
    for k, v in stream.state_dict().items():
        if k.endswith("num_batches_tracked"):
            continue
        a = v.detach().cpu().numpy().astype("<f4")
        fb.write(a.tobytes())
        ft.write(f"{k} {','.join(map(str, a.shape)) or '1'} {off} {a.size}\n")
        off += a.size
print("params", off)
