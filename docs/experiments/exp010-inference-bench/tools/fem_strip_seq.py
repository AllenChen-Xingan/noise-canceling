"""把 FastEnhancer ONNX 里的 SplitToSequence(axis=a, keepdims=0) + SequenceAt(常数下标 i) 换成 Gather(axis=a, 下标 i)。

为什么：tract 0.23.9 不支持 SplitToSequence（报 Unimplemented(SplitToSequence)）。
两者数学上等价：「沿 a 轴拆开、不保留该维，取第 i 片」= 「沿 a 轴取下标 i（标量下标，结果去掉该维）」。
只改这一种模式；遇到别的用法（split 输入、keepdims=1、序列被多处使用、下标非常数）就报错退出，不猜。
等价性另由 bench `fem-parity`（ORT 跑原图 对 tract 跑改后的图）实测证明。

用法：python fem_strip_seq.py <in.onnx> <out.onnx>
"""
import sys
import onnx
from onnx import helper, numpy_helper

src, dst = sys.argv[1], sys.argv[2]
m = onnx.load(src)
g = m.graph
consts = {i.name: numpy_helper.to_array(i) for i in g.initializer}
for n in g.node:
    if n.op_type == "Constant":
        for a in n.attribute:
            if a.name == "value":
                consts[n.output[0]] = numpy_helper.to_array(a.t)
users = {}
for n in g.node:
    for i in n.input:
        users.setdefault(i, []).append(n)

drop, new = set(), {}
for n in g.node:
    if n.op_type != "SplitToSequence":
        continue
    at = {a.name: helper.get_attribute_value(a) for a in n.attribute}
    if len(n.input) != 1 or at.get("keepdims", 1) != 0:
        sys.exit(f"{n.name}: 不是 keepdims=0 且无 split 输入的用法，不改")
    us = users.get(n.output[0], [])
    if len(us) != 1 or us[0].op_type != "SequenceAt":
        sys.exit(f"{n.name}: 序列不是只被一个 SequenceAt 使用")
    sa = us[0]
    if sa.input[1] not in consts or consts[sa.input[1]].ndim != 0:
        sys.exit(f"{sa.name}: 下标不是常数标量")
    gather = helper.make_node(
        "Gather", [n.input[0], sa.input[1]], [sa.output[0]], name=n.name + "_as_gather", axis=at.get("axis", 0)
    )
    drop |= {id(n), id(sa)}
    new[id(sa)] = gather
    print(f"{n.name} + {sa.name} → Gather(axis={at.get('axis', 0)}, 下标={int(consts[sa.input[1]])})")

nodes = [new.get(id(n), n) for n in g.node if id(n) not in drop or id(n) in new]
del g.node[:]
g.node.extend(nodes)
onnx.checker.check_model(m)
onnx.save(m, dst)
print("写出", dst, "替换", len(new), "处")
