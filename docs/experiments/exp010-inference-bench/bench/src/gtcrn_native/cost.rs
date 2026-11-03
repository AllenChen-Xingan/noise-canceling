//! 研究 020：每段计算的工作量（`Cost`）与打点接口（`Probe`）。


// ───────────────────── 计算量与打点（研究 020 下界分析） ─────────────────────
// 每层的计算量由各层自己的尺寸当场算出（正本 = 本文件的结构体字段），层的顺序与名字只在 `infer_probed` 里写一处。
// `infer` = `infer_probed(.., &mut NoProbe)`，空打点编译后消失，输出与原来逐位相同。

/// 一段计算的工作量。`chain_*` 是关键路径（最短依赖链）：归约按树形算（⌈log2 K⌉ + 1 步），
/// 循环网络沿序列串行相乘。下界分析拿它乘本机实测的单步延迟。
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Cost {
    pub macs: u64,      // 乘加次数
    pub nonlin: u64,    // exp 级非线性（sigmoid / tanh）次数
    pub other: u64,     // 其它逐元素运算（PReLU、加法、sqrt、拷贝的元素数）
    pub w_bytes: u64,   // 读权重与偏置的字节数
    pub io_bytes: u64,  // 读入 + 写出特征的字节数
    pub chain_fma: u64, // 关键路径上依赖的乘加 / 加法步数
    pub chain_nl: u64,  // 关键路径上依赖的非线性次数
}

impl std::ops::Add for Cost {
    type Output = Cost;
    fn add(self, o: Cost) -> Cost {
        Cost {
            macs: self.macs + o.macs,
            nonlin: self.nonlin + o.nonlin,
            other: self.other + o.other,
            w_bytes: self.w_bytes + o.w_bytes,
            io_bytes: self.io_bytes + o.io_bytes,
            chain_fma: self.chain_fma + o.chain_fma,
            chain_nl: self.chain_nl + o.chain_nl,
        }
    }
}

/// ⌈log2 n⌉（n ≥ 1）：n 个数树形求和的层数
pub fn clog2(n: usize) -> u64 {
    if n <= 1 {
        0
    } else {
        (usize::BITS - (n - 1).leading_zeros()) as u64
    }
}

pub(crate) const F32B: u64 = 4;

/// 打点：每段计算之后调一次。`cost` 只在需要时才求值（计时打点不调它）。
pub trait Probe {
    #[inline(always)]
    fn mark(&mut self, _blk: &'static str, _op: &'static str, _cost: impl FnOnce() -> Cost) {}
}
/// 不打点（正式推理用）
pub struct NoProbe;
impl Probe for NoProbe {}


/// 工作量乘 `n` 份，关键路径乘 `serial`（串行的份数），权重有 `wsets` 套（同一套反复用只读一次）
pub(crate) fn scale(c: Cost, n: u64, serial: u64, wsets: u64) -> Cost {
    Cost {
        macs: c.macs * n,
        nonlin: c.nonlin * n,
        other: c.other * n,
        w_bytes: c.w_bytes * wsets,
        io_bytes: c.io_bytes * n,
        chain_fma: c.chain_fma * serial,
        chain_nl: c.chain_nl * serial,
    }
}

/// `layer_norm_add` 的工作量：求均值、求方差（各一次树形归约）、sqrt 与倒数、逐元素仿射加回
pub(crate) fn ln_cost(n: usize) -> Cost {
    let m = n as u64;
    Cost {
        macs: 2 * m, // 方差平方和 + 仿射
        other: 2 * m + 2,
        w_bytes: 2 * m * F32B,
        io_bytes: 3 * m * F32B,
        chain_fma: 2 * (clog2(n) + 1) + 2 + 2,
        ..Cost::default()
    }
}
