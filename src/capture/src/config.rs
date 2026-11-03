//! `Forwarder` 的参数与计数器。

/// `Forwarder` 的参数（全部按 48 kHz 样本数）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForwarderConfig {
    /// 总延迟：本地原声延迟线的长度，等于总线头里的 `l_total`
    pub l_total: usize,
    /// 每次 `process` 最多处理多少样本；更长的输入内部切块
    pub max_block: usize,
    /// 心跳多少样本不动算过期（100 ms = 4800）
    pub timeout: usize,
    /// 处理后 ↔ 原声 的淡化长度（10 ms = 480）
    pub fade: usize,
    /// Processed 里积压超过这么多就丢掉多余的（防延迟越积越大）；0 = 自动取 `l_total + 2 × max_block`
    pub max_backlog: usize,
}

impl ForwarderConfig {
    pub fn new(l_total: usize, max_block: usize) -> Self {
        Self {
            l_total,
            max_block,
            timeout: 4800,
            fade: 480,
            max_backlog: 0,
        }
    }
}

/// 计数器（只增不减；实时线程写，非实时线程读拷贝）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ForwarderStats {
    /// 处理过的样本总数
    pub samples: u64,
    /// Raw 环满，没推进去的原声样本
    pub raw_overflow: u64,
    /// 心跳新鲜时 Processed 不够、用原声补的样本
    pub proc_underrun: u64,
    /// 心跳过期期间输出的样本（整段送原声）
    pub stale_samples: u64,
    /// Out 环满，没推进去的样本
    pub out_overflow: u64,
    /// 心跳 新鲜 → 过期 的次数（进程级送原声触发次数）
    pub stale_switches: u64,
    /// 心跳 过期 → 新鲜 的次数（含启动时第一次）
    pub recoveries: u64,
    /// 为保持对齐或限积压而丢掉的处理后样本
    pub discarded: u64,
    /// 积压超限、按「从现在起对齐」重新对账的次数
    pub resyncs: u64,
    /// 缺口记录满了、合并进最后一段的次数（只影响落后那一段的对齐，那段样本本来就迟到会被丢；追上后恢复）
    pub gap_overflow: u64,
    /// 硬切次数：该用处理后的位置却没有处理后的样本，只能立刻切原声（用接缝偏移保持波形连续）
    pub cuts: u64,
    /// 「降噪太慢」次数：心跳在动、但处理后的连续缺了超过心跳超时那么久（心跳过期计数看不到这种情况）
    pub slow_episodes: u64,
    /// 降噪进程报了跳过积压（缺口计数涨了）、送音端改送一个 L_total 窗口原声的次数
    pub gap_holds: u64,
    /// 处理后的样本不是有限值、换成原声的个数（防御：不该出现）
    pub nonfinite: u64,
    /// 最近一次 新鲜 → 过期 发生在第几个样本（`samples` 计）
    pub last_stale_at: Option<u64>,
}
