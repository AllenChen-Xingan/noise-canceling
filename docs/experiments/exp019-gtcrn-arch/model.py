"""把一个候选模块挂进官方流式 GTCRN 的瓶颈处，每次推一帧（16 ms）。

基座结构、权重、缓存形状都取自官方 `StreamGTCRN`（正本），这里只重写一次前向的串接顺序，
并在第二个双路 RNN 之后插入候选。`base` 候选的输出与官方流式版逐值相同（测试核对）。
"""
import torch
import torch.nn as nn

from gtcrn_src import import_official, load_stream_model
import variants


class ArchModel(nn.Module):
    def __init__(self, stream_base, extra):
        super().__init__()
        self.base = stream_base
        self.extra = extra

    def init_state(self, batch=1):
        return {
            "conv": torch.zeros(2, batch, 16, 16, 33),
            "tra": torch.zeros(2, 3, 1, batch, 16),
            "inter": torch.zeros(2, 1, batch * 33, 16),
            "extra": self.extra.init_state(batch),
        }

    def forward(self, spec, st, cond=None):
        """spec: (B, 257, 1, 2) 一帧复数频谱。返回 (增强频谱, 新状态)。"""
        m = self.base
        conv, tra, inter = st["conv"].clone(), st["tra"].clone(), st["inter"].clone()
        re = spec[..., 0].permute(0, 2, 1)
        im = spec[..., 1].permute(0, 2, 1)
        mag = torch.sqrt(re ** 2 + im ** 2 + 1e-12)
        feat = m.sfe(m.erb.bm(torch.stack([mag, re, im], dim=1)))
        feat, en_outs, conv[0], tra[0] = m.encoder(feat, conv[0], tra[0])
        feat, inter[0] = m.dpgrnn1(feat, inter[0])
        feat, inter[1] = m.dpgrnn2(feat, inter[1])
        feat, extra = self.extra(feat, st["extra"], cond)
        mfeat, conv[1], tra[1] = m.decoder(feat, en_outs, conv[1], tra[1])
        mask = m.erb.bs(mfeat)
        out = m.mask(mask, spec.permute(0, 3, 2, 1)).permute(0, 3, 2, 1)
        return out, {"conv": conv, "tra": tra, "inter": inter, "extra": extra}


def make(name, weights=True):
    """造一个带候选的模型。weights=False 时基座随机初始化（只数乘加、测速用也行）。"""
    _, StreamGTCRN, _ = import_official()
    if weights:
        _, st = load_stream_model()
    else:
        st = StreamGTCRN().eval()
    dpgrnn_cls = type(st.dpgrnn1)
    return ArchModel(st, variants.build(name, dpgrnn_cls)).eval()
