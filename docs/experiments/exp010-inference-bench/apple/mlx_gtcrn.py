"""exp010 · GTCRN 流式模型的 MLX 版（自写，按 resources/gtcrn/stream/gtcrn_stream.py 逐层对照），用来测 MLX 后端。

三个子命令：
  export <resources/gtcrn/stream 目录> <out.npz>
      用 torch 建 StreamGTCRN（DNS3 权重，与 gtcrn_simple.onnx 同源），把 state_dict 存成 npz。要 torch。
  check  <resources/gtcrn/stream 目录> <weights.npz> [--frames 50]
      同一串随机输入，逐帧比 MLX 版和 torch 版（缓存一路往下走），报最大绝对差。要 torch。
  bench  <weights.npz> <in.f32> <ort.f32> [--device cpu|gpu] [--no-compile] [--frames N] [--warmup N] [--parity-frames N]
      同 rustbe bench 口径：1 冷启动 + 预热 + 计时；对拍前 610 帧对 ORT 转储；一行 JSON。
      计时包住：numpy → mx.array、调用、mx.eval、取回 numpy（实时线程真要做的全部）。

形状同 ONNX：mix [1,257,1,2] · conv_cache [2,1,16,16,33] · tra_cache [2,3,1,1,16] · inter_cache [2,1,33,16]。
MLX 卷积是通道在后（NHWC，权重 OHWI），这里张量保持 torch 的 NCHW，只在卷积前后转置。
"""
import json
import os
import sys
import time
import types

import numpy as np

BINS = 257


# ───────────────────────── 导出权重（torch） ─────────────────────────
def build_torch(src):
    sys.path.insert(0, src)
    sys.modules.setdefault("einops", types.SimpleNamespace(rearrange=None))
    import torch
    from gtcrn import GTCRN
    from gtcrn_stream import StreamGTCRN
    from modules.convert import convert_to_stream

    base = GTCRN().eval()
    base.load_state_dict(torch.load(f"{src}/onnx_models/model_trained_on_dns3.tar", map_location="cpu")["model"])
    stream = StreamGTCRN().eval()
    convert_to_stream(stream, base)
    return stream


# ───────────────────────── MLX 前向 ─────────────────────────
def make_forward(W):
    import mlx.core as mx

    def P(k):
        return W[k]

    def conv2d(x, w, b, stride=(1, 1), padding=(0, 0), dilation=(1, 1), groups=1):
        """x: NCHW；w: torch OIHW。"""
        y = mx.conv2d(x.transpose(0, 2, 3, 1), w.transpose(0, 2, 3, 1), stride=stride, padding=padding,
                      dilation=dilation, groups=groups)
        y = y.transpose(0, 3, 1, 2)
        return y if b is None else y + b.reshape(1, -1, 1, 1)

    def conv_t_1x5_s2(x, wt, b, groups):
        """nn.ConvTranspose2d(k=(1,5), stride=(1,2), padding=(0,2))。做法：F 方向插零上采样 + 两边各补 2 + 翻转核的普通卷积。
        wt: torch (in, out/groups, 1, 5)。"""
        B, C, T, F = x.shape
        up = mx.concatenate([x[..., None], mx.zeros((B, C, T, F, 1), dtype=x.dtype)], axis=-1).reshape(B, C, T, 2 * F)
        up = up[..., : 2 * F - 1]
        cin, og = wt.shape[0], wt.shape[1]
        ig = cin // groups
        ws = []
        for g in range(groups):
            blk = wt[g * ig:(g + 1) * ig]  # (ig, og, 1, 5)
            ws.append(blk.transpose(1, 0, 2, 3)[..., ::-1])  # (og, ig, 1, 5)
        w = mx.concatenate(ws, axis=0)
        return conv2d(up, w, b, padding=(0, 2), groups=groups)

    def conv_t_1x1(x, wt, b):
        """1×1 ConvTranspose2d：out[o] = Σ_i W[i,o]·x[i] + b。"""
        y = mx.einsum("bitf,io->botf", x, wt[:, :, 0, 0])
        return y + b.reshape(1, -1, 1, 1)

    def bn(x, p):
        s = P(p + ".weight") / mx.sqrt(P(p + ".running_var") + 1e-5)
        return x * s.reshape(1, -1, 1, 1) + (P(p + ".bias") - P(p + ".running_mean") * s).reshape(1, -1, 1, 1)

    def prelu(x, p):
        a = P(p + ".weight")
        return mx.maximum(x, 0) + a.reshape(1, -1, 1, 1) * mx.minimum(x, 0) if a.size > 1 else mx.maximum(x, 0) + a * mx.minimum(x, 0)

    def linear(x, p, bias=True):
        y = x @ P(p + ".weight").T
        return y + P(p + ".bias") if bias else y

    def gru_dir(xs, h, p, sfx):
        """单方向 GRU，torch 门顺序 r,z,n。xs: list of (B,I)；h: (B,H)。返回每步输出 list 和末状态。"""
        wih, whh = P(f"{p}.weight_ih_l0{sfx}"), P(f"{p}.weight_hh_l0{sfx}")
        bih, bhh = P(f"{p}.bias_ih_l0{sfx}"), P(f"{p}.bias_hh_l0{sfx}")
        H = whh.shape[1]
        outs = []
        for x in xs:
            gi = x @ wih.T + bih
            gh = h @ whh.T + bhh
            r = mx.sigmoid(gi[:, :H] + gh[:, :H])
            z = mx.sigmoid(gi[:, H:2 * H] + gh[:, H:2 * H])
            n = mx.tanh(gi[:, 2 * H:] + r * gh[:, 2 * H:])
            h = (1 - z) * n + z * h
            outs.append(h)
        return outs, h

    def gru(x, h0, p, bidir):
        """x: (B,L,I) batch_first；h0: (D,B,H)。返回 (B,L,D·H), (D,B,H)。"""
        L = x.shape[1]
        xs = [x[:, t] for t in range(L)]
        of, hf = gru_dir(xs, h0[0], p, "")
        if not bidir:
            return mx.stack(of, axis=1), hf[None]
        ob, hb = gru_dir(xs[::-1], h0[1], p, "_reverse")
        ob = ob[::-1]
        return mx.concatenate([mx.stack(of, axis=1), mx.stack(ob, axis=1)], axis=-1), mx.stack([hf, hb])

    def grnn(x, h, p, bidir):
        hs = x.shape[-1] // 2
        hh = h.shape[-1] // 2
        y1, h1 = gru(x[..., :hs], h[..., :hh], p + ".rnn1", bidir)
        y2, h2 = gru(x[..., hs:], h[..., hh:], p + ".rnn2", bidir)
        return mx.concatenate([y1, y2], -1), mx.concatenate([h1, h2], -1)

    def sfe(x):
        B, C, T, F = x.shape
        xp = mx.pad(x, [(0, 0), (0, 0), (0, 0), (1, 1)])
        return mx.stack([xp[..., k:k + F] for k in range(3)], axis=2).reshape(B, 3 * C, T, F)

    def erb_bm(x):
        return mx.concatenate([x[..., :65], x[..., 65:] @ P("erb.erb_fc.weight").T], -1)

    def erb_bs(x):
        return mx.concatenate([x[..., :65], x[..., 65:] @ P("erb.ierb_fc.weight").T], -1)

    def conv_block(x, p, enc, groups, last):
        if enc:
            y = conv2d(x, P(p + ".conv.weight"), P(p + ".conv.bias"), stride=(1, 2), padding=(0, 2), groups=groups)
        else:
            y = conv_t_1x5_s2(x, P(p + ".conv.weight"), P(p + ".conv.bias"), groups)
        y = bn(y, p + ".bn")
        return mx.tanh(y) if last else prelu(y, p + ".act")

    def tra(x, h, p):
        zt = mx.mean(x * x, axis=-1)  # (B,C,T)
        at, h = gru(zt.transpose(0, 2, 1), h, p + ".att_gru", False)
        at = mx.sigmoid(linear(at, p + ".att_fc").transpose(0, 2, 1))
        return x * at[..., None], h

    def gt_block(x, cache, h, p, dil, deconv):
        C = x.shape[1]
        x1, x2 = x[:, :C // 2], x[:, C // 2:]
        x1 = sfe(x1)
        if deconv:
            h1 = conv_t_1x1(x1, P(p + ".point_conv1.weight"), P(p + ".point_conv1.bias"))
        else:
            h1 = conv2d(x1, P(p + ".point_conv1.weight"), P(p + ".point_conv1.bias"))
        h1 = prelu(bn(h1, p + ".point_bn1"), p + ".point_act")
        inp = mx.concatenate([cache, h1], axis=2)
        new_cache = inp[:, :, 1:]
        layer = "ConvTranspose2d" if deconv else "Conv2d"
        h1 = conv2d(inp, P(f"{p}.depth_conv.{layer}.weight"), P(f"{p}.depth_conv.{layer}.bias"), padding=(0, 1),
                    dilation=(dil, 1), groups=h1.shape[1])
        h1 = prelu(bn(h1, p + ".depth_bn"), p + ".depth_act")
        if deconv:
            h1 = conv_t_1x1(h1, P(p + ".point_conv2.weight"), P(p + ".point_conv2.bias"))
        else:
            h1 = conv2d(h1, P(p + ".point_conv2.weight"), P(p + ".point_conv2.bias"))
        h1 = bn(h1, p + ".point_bn2")
        h1, h = tra(h1, h, p + ".tra")
        B, Ch, T, F = h1.shape
        x = mx.stack([h1, x2], axis=2).reshape(B, 2 * Ch, T, F)  # shuffle：通道交错
        return x, new_cache, h

    def ln(x, p):
        mu = mx.mean(x, axis=(-2, -1), keepdims=True)
        var = mx.mean((x - mu) ** 2, axis=(-2, -1), keepdims=True)
        return (x - mu) / mx.sqrt(var + 1e-8) * P(p + ".weight") + P(p + ".bias")

    def dpgrnn(x, h, p):
        B, C, T, F = x.shape
        x = x.transpose(0, 2, 3, 1)  # (B,T,F,C)
        ia = x.reshape(B * T, F, C)
        z = mx.zeros((2, B * T, 8), dtype=x.dtype)
        ia, _ = grnn(ia, z, p + ".intra_rnn", True)
        ia = linear(ia, p + ".intra_fc").reshape(B, T, F, C)
        intra_out = x + ln(ia, p + ".intra_ln")
        y = intra_out.transpose(0, 2, 1, 3).reshape(B * F, T, C)
        y, h = grnn(y, h, p + ".inter_rnn", False)
        y = linear(y, p + ".inter_fc").reshape(B, F, T, C).transpose(0, 2, 1, 3)
        out = intra_out + ln(y, p + ".inter_ln")
        return out.transpose(0, 3, 1, 2), h

    DIL = [1, 2, 5]
    SL = [(0, 2), (2, 6), (6, 16)]

    def forward(spec, conv_cache, tra_cache, inter_cache):
        re = spec[..., 0].transpose(0, 2, 1)
        im = spec[..., 1].transpose(0, 2, 1)
        mag = mx.sqrt(re * re + im * im + 1e-12)
        feat = sfe(erb_bm(mx.stack([mag, re, im], axis=1)))
        # 编码器
        en = []
        x = conv_block(feat, "encoder.en_convs.0", True, 1, False); en.append(x)
        x = conv_block(x, "encoder.en_convs.1", True, 2, False); en.append(x)
        ec, et = [], []
        for i in range(3):
            a, b = SL[i]
            x, c, h = gt_block(x, conv_cache[0][:, :, a:b], tra_cache[0][i], f"encoder.en_convs.{i + 2}", DIL[i], False)
            en.append(x); ec.append(c); et.append(h)
        x, h1 = dpgrnn(x, inter_cache[0], "dpgrnn1")
        x, h2 = dpgrnn(x, inter_cache[1], "dpgrnn2")
        # 解码器（缓存段倒序用）
        dc, dt = [None] * 3, []
        for j, i in enumerate([2, 1, 0]):
            a, b = SL[i]
            x, c, h = gt_block(x + en[4 - j], conv_cache[1][:, :, a:b], tra_cache[1][j], f"decoder.de_convs.{j}", DIL[i], True)
            dc[i] = c; dt.append(h)
        x = conv_block(x + en[1], "decoder.de_convs.3", False, 2, False)
        x = conv_block(x + en[0], "decoder.de_convs.4", False, 1, True)
        m = erb_bs(x)
        s = spec.transpose(0, 3, 2, 1)
        sr = s[:, 0] * m[:, 0] - s[:, 1] * m[:, 1]
        si = s[:, 1] * m[:, 0] + s[:, 0] * m[:, 1]
        enh = mx.stack([sr, si], axis=1).transpose(0, 3, 2, 1)
        conv_out = mx.stack([mx.concatenate(ec, axis=2), mx.concatenate(dc, axis=2)])
        tra_out = mx.stack([mx.stack(et), mx.stack(dt)])
        inter_out = mx.stack([h1, h2])
        return enh, conv_out, tra_out, inter_out

    return forward


def load_weights(npz):
    import mlx.core as mx
    d = np.load(npz)
    return {k: mx.array(d[k].astype(np.float32)) for k in d.files}


SHAPES = {"conv": (2, 1, 16, 16, 33), "tra": (2, 3, 1, 1, 16), "inter": (2, 1, 33, 16)}


# ───────────────────────── 子命令 ─────────────────────────
def cmd_export(src, out):
    m = build_torch(src)
    sd = {k: v.detach().cpu().numpy() for k, v in m.state_dict().items() if not k.endswith("num_batches_tracked")}
    np.savez(out, **sd)
    print(json.dumps({"tensors": len(sd), "params": int(sum(v.size for v in sd.values()))}))


def cmd_check(src, npz, frames):
    import mlx.core as mx
    import torch
    tm = build_torch(src)
    fwd = make_forward(load_weights(npz))
    rng = np.random.default_rng(0)
    tc = [torch.zeros(s) for s in SHAPES.values()]
    mc = [mx.zeros(s) for s in SHAPES.values()]
    worst = 0.0
    for f in range(frames):
        x = rng.standard_normal((1, BINS, 1, 2)).astype(np.float32) * 0.1
        with torch.no_grad():
            ty, *tc = tm(torch.from_numpy(x), *[c.clone() for c in tc])
        my, *mc = fwd(mx.array(x), *mc)
        mx.eval(my, *mc)
        d = float(np.abs(np.array(my) - ty.numpy()).max())
        dc = max(float(np.abs(np.array(a) - b.numpy()).max()) for a, b in zip(mc, tc))
        worst = max(worst, d, dc)
    print(json.dumps({"frames": frames, "max_abs_vs_torch": worst}))


def cmd_bench(npz, fin, fref, device, use_compile, frames, warmup, parity_n):
    import mlx.core as mx
    mx.set_default_device(mx.gpu if device == "gpu" else mx.cpu)
    t0 = time.perf_counter()
    W = load_weights(npz)
    fwd = make_forward(W)
    f = mx.compile(fwd) if use_compile else fwd
    open_ms = (time.perf_counter() - t0) * 1e3
    inp = np.fromfile(fin, dtype="<f4").reshape(-1, 2 * BINS)
    ref = np.fromfile(fref, dtype="<f4").reshape(-1, 2 * BINS)
    n_in = len(inp)
    # 输入换成 [1,257,1,2] 交错布局
    mixes = np.stack([inp[:, :BINS], inp[:, BINS:]], axis=-1).reshape(n_in, 1, BINS, 1, 2).astype(np.float32)
    caches = [mx.zeros(s) for s in SHAPES.values()]
    mx.eval(*caches)

    alloc = None
    try:  # 全进程 malloc 计数（含 Python 解释器自己的分配，偏大）
        import ctypes
        lib = ctypes.CDLL(os.path.join(os.path.dirname(os.path.abspath(__file__)), "build", "libvdn_alloc.dylib"))
        lib.vdn_alloc_count.restype = ctypes.c_uint64
        lib.vdn_alloc_hook_install()
        alloc = lib.vdn_alloc_count
    except OSError:
        pass

    total, t0w = 1 + warmup + frames, 1 + warmup
    times = []
    first = 0.0
    maxabs = sig = err = 0.0
    worst, bad, nonfinite = 0, None, 0
    a0 = c0 = w0 = 0
    for i in range(total):
        if i == t0w:
            a0 = alloc() if alloc else 0
            c0 = time.process_time()
            w0 = time.perf_counter()
        t = time.perf_counter()
        y, *caches = f(mx.array(mixes[i % n_in]), *caches)
        mx.eval(y, *caches)
        out = np.array(y)
        dt = (time.perf_counter() - t) * 1e3
        if i == 0:
            first = dt
        o = np.concatenate([out[0, :, 0, 0], out[0, :, 0, 1]])
        if i >= t0w:
            times.append(dt)
            if not np.isfinite(o).all():
                nonfinite += 1
        if i < min(parity_n, n_in):
            d = np.abs(o.astype(np.float64) - ref[i])
            sig += float((ref[i].astype(np.float64) ** 2).sum())
            err += float((d ** 2).sum())
            if d.max() > maxabs:
                maxabs, worst = float(d.max()), i
            if bad is None and (d > 1e-3).any():
                bad = (i, int(np.argmax(d > 1e-3)) % BINS)
    wall = time.perf_counter() - w0
    cpu = time.process_time() - c0
    allocs = (alloc() - a0) / frames if alloc else None
    ts = np.sort(np.array(times))
    pct = lambda q: float(ts[min(len(ts) - 1, int(len(ts) * q))])
    print(json.dumps({
        "model": "gtcrn", "backend": f"mlx-{device}" + ("" if use_compile else "-nocompile"), "frames": frames, "warmup": warmup,
        "p50_ms": round(pct(0.5), 4), "p99_ms": round(pct(0.99), 4), "p999_ms": round(pct(0.999), 4),
        "max_ms": round(float(ts[-1]), 4), "mean_ms": round(float(ts.mean()), 4), "open_ms": round(open_ms, 2),
        "first_frame_ms": round(first, 3), "cpu_over_wall": round(cpu / wall, 3),
        "process_allocs_per_frame": None if allocs is None else round(allocs, 3),
        "nonfinite_frames": nonfinite, "parity_frames": min(parity_n, n_in), "parity_max_abs": float(f"{maxabs:.3e}"),
        "parity_worst_frame": worst, "parity_snr_db": round(10 * np.log10(sig / max(err, 1e-300)), 2),
        "parity_first_bad": "None" if bad is None else f"Some({bad})", "mlx": mx.__version__,
    }))


def main():
    a = sys.argv[1:]
    opt = {}
    pos = []
    i = 0
    while i < len(a):
        if a[i] == "--no-compile":
            opt["no-compile"] = True; i += 1
        elif a[i].startswith("--"):
            opt[a[i][2:]] = a[i + 1]; i += 2
        else:
            pos.append(a[i]); i += 1
    cmd = pos[0]
    if cmd == "export":
        cmd_export(pos[1], pos[2])
    elif cmd == "check":
        cmd_check(pos[1], pos[2], int(opt.get("frames", 50)))
    elif cmd == "bench":
        cmd_bench(pos[1], pos[2], pos[3], opt.get("device", "cpu"), not opt.get("no-compile"),
                  int(opt.get("frames", 6000)), int(opt.get("warmup", 200)), int(opt.get("parity-frames", 610)))
    else:
        sys.exit("用法见文件顶部")


if __name__ == "__main__":
    main()
