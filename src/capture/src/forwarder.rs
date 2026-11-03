//! 送音转发器 `Forwarder`：原声 → Raw；按心跳与对齐账从 Processed / 延迟原声取样混合 → Out（IO 回调，A 档）。

use vdn_dsp::DelayLine;

use crate::{ForwarderConfig, ForwarderStats, Heartbeat, Pop, Push};

/// Raw 环满时丢掉的一段原声：在「已推进 Raw 的第 `at` 个样本」之前缺了 `len` 个。
#[derive(Clone, Copy, Debug, Default)]
struct Gap {
    at: i64,
    len: i64,
}

/// `have` 的取值：没取到 / 取到真声音 / 取到占位样本
const MISSING: u8 = 0;
const OK: u8 = 1;
const PLACEHOLDER: u8 = 2;

/// 最多记多少段未消化的缺口；满了就并进最后一段（对齐会错，计入 `gap_overflow`）
const MAX_GAPS: usize = 32;

/// 送音进程的转发器：原声 → Raw；Processed / 延迟原声 → Out。
///
/// 对齐账：Processed 里第 j 个样本 = 开头 `lead` 个零之后、管线（延迟 `L_pipe`）对「推进 Raw 的第
/// `j - lead - L_pipe = j - L_total` 个原声」的输出，所以它该在输出时刻 `A(j - L_total) + L_total` 播出
/// （`A` 把「推进去的序号」换成原声时刻，跨过 Raw 环满丢掉的缺口）。只用到 `L_total`，总线头里就有。
/// 迟到的（时刻已过）丢掉，超前的（缺口造成）先用原声顶，缺的用原声补——任何时候都逐样本对齐。
pub struct Forwarder {
    cfg: ForwarderConfig,
    delay: DelayLine,
    raw_delayed: Vec<f32>,
    proc_buf: Vec<f32>,
    out_buf: Vec<f32>,
    /// 最近一次推进 Out 的块长（`last_output` 用）
    last_n: usize,
    /// 这一块里每个位置有没有取到处理后的样本
    have: Vec<u8>,
    /// 处理后声音的权重（0 = 全原声，1 = 全处理后）
    gain: f32,
    step: f32,
    /// 连续取到处理后样本的个数（回滞：够 `fade` 个才往回淡）
    ok_run: usize,
    /// 心跳新鲜时连续缺处理后样本的个数（判「降噪太慢」）
    late_run: usize,
    /// 接缝偏移：硬切那一刻输出的落差，按指数衰减加回去，保持波形连续（规约 R11）
    seam: f32,
    seam_decay: f32,
    last_y: f32,
    /// 上一个样本的延迟原声（接缝偏移 = 上一个输出 − 上一个原声）
    last_r: f32,
    /// 下一个样本强制按硬切处理（重新对账之后）
    pending_cut: bool,
    /// 上次看到的降噪进程缺口计数；`None` = 还没看过（第一次只记基准）
    last_gaps: Option<u64>,
    /// 还要送多少个样本的原声（缺口计数涨了之后的一个 L_total 窗口）
    gap_hold: usize,
    last_beat: Option<u64>,
    since_beat: usize,
    fresh: bool,
    /// 已推进 Raw 的原声样本数
    accepted: i64,
    /// Processed 环头那个样本对应的「推进序号」（开头 = -L_total：lead 个零 + 管线开头的 L_pipe 个输出）
    head_acc: i64,
    /// 环头已跨过的缺口总长
    head_shift: i64,
    /// 环头之后还没跨过的缺口（按 at 递增）
    gaps: [Gap; MAX_GAPS],
    n_gaps: usize,
    stats: ForwarderStats,
}

// 构造：非实时线程
impl Forwarder {
    pub fn new(cfg: ForwarderConfig) -> Self {
        let max_block = cfg.max_block.max(1);
        let mut cfg = cfg;
        cfg.max_block = max_block;
        if cfg.max_backlog == 0 {
            cfg.max_backlog = cfg.l_total + 2 * max_block;
        }
        let l_total = cfg.l_total as i64;
        Self {
            cfg,
            delay: DelayLine::new(cfg.l_total),
            raw_delayed: vec![0.0; max_block],
            proc_buf: vec![0.0; max_block],
            out_buf: vec![0.0; max_block],
            last_n: 0,
            have: vec![MISSING; max_block],
            gain: 0.0,
            step: 1.0 / cfg.fade.max(1) as f32,
            ok_run: 0,
            late_run: 0,
            seam: 0.0,
            // 时间常数 2 ms（96 样本 @ 48 kHz）
            seam_decay: (-1.0f32 / 96.0).exp(),
            last_y: 0.0,
            last_r: 0.0,
            pending_cut: false,
            last_gaps: None,
            gap_hold: 0,
            last_beat: None,
            since_beat: 0,
            fresh: false,
            accepted: 0,
            head_acc: -l_total,
            head_shift: 0,
            gaps: [Gap::default(); MAX_GAPS],
            n_gaps: 0,
            stats: ForwarderStats::default(),
        }
    }
}

// 实时（specs/rust/architecture.md §5.3）
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl Forwarder {
    pub fn config(&self) -> ForwarderConfig {
        self.cfg
    }
    pub fn stats(&self) -> ForwarderStats {
        self.stats
    }
    /// 心跳是否新鲜（新鲜 = 用处理后的声音）
    pub fn is_fresh(&self) -> bool {
        self.fresh
    }
    /// 当前处理后声音的权重
    pub fn gain(&self) -> f32 {
        self.gain
    }

    /// 最近一次 `process` 推进 Out 的最后一块（纠漂移时补一块用）
    pub fn last_output(&self) -> &[f32] {
        self.out_buf.get(..self.last_n).unwrap_or(&[])
    }

    /// 换到一条新总线（降噪进程重启后建的，`L_total` 不变）：保留本地延迟线和时间轴（原声不断），
    /// 只把对齐账清零——新总线 Processed 的第 0 个样本对应「从现在起推进 Raw 的第 −L_total 个」。
    /// 立刻切原声，等新心跳和回滞再淡回。`L_total` 变了不能用它，要新建 `Forwarder`。
    pub fn rebind(&mut self) {
        let t = self.stats.samples as i64;
        self.accepted = 0;
        self.head_acc = -(self.cfg.l_total as i64);
        self.head_shift = t;
        self.n_gaps = 0;
        self.last_beat = None;
        self.since_beat = 0;
        self.fresh = false;
        self.late_run = 0;
        self.ok_run = 0;
        self.pending_cut = self.gain > 0.0;
    }

    /// 一块原声进，一块结果推进 Out。
    pub fn process<R, P, O, H>(
        &mut self,
        input: &[f32],
        raw: &mut R,
        processed: &mut P,
        out: &mut O,
        hb: &H,
    ) where
        R: Push,
        P: Pop,
        O: Push,
        H: Heartbeat + ?Sized,
    {
        let mb = self.cfg.max_block;
        for chunk in input.chunks(mb) {
            let n = chunk.len();
            let mut buf = core::mem::take(&mut self.out_buf);
            if let Some(y) = buf.get_mut(..n) {
                self.chunk(chunk, y, raw, processed, hb);
                let pushed = out.push(y);
                self.stats.out_overflow += (n - pushed.min(n)) as u64;
            }
            self.out_buf = buf;
            self.last_n = n;
        }
    }

    /// 同 [`Self::process`]，但结果写进 `output`（与 `input` 等长）而不推 Out：开发期直接接扬声器 / 文件用。
    pub fn process_into<R, P, H>(
        &mut self,
        input: &[f32],
        output: &mut [f32],
        raw: &mut R,
        processed: &mut P,
        hb: &H,
    ) where
        R: Push,
        P: Pop,
        H: Heartbeat + ?Sized,
    {
        let n = input.len().min(output.len());
        let mb = self.cfg.max_block;
        let (head, tail) = output.split_at_mut(n);
        tail.fill(0.0);
        let input = input.get(..n).unwrap_or(&[]);
        for (x, y) in input.chunks(mb).zip(head.chunks_mut(mb)) {
            self.chunk(x, y, raw, processed, hb);
        }
    }

    /// 环头样本该在哪个输出时刻播出
    fn head_time(&self) -> i64 {
        self.head_acc + self.head_shift + self.cfg.l_total as i64
    }

    /// 环头到下一个缺口还有几个连续样本
    fn run_to_gap(&self) -> i64 {
        match self.gaps.first() {
            Some(g) if self.n_gaps > 0 => (g.at - self.head_acc).max(0),
            _ => i64::MAX,
        }
    }

    /// 环头前进 k 个样本，跨过的缺口计入 head_shift
    fn advance(&mut self, k: usize) {
        self.head_acc += k as i64;
        while self.n_gaps > 0 {
            let Some(g) = self.gaps.first().copied() else {
                break;
            };
            if g.at > self.head_acc {
                break;
            }
            self.head_shift += g.len;
            // 切片写法：不变式（n_gaps ≤ MAX_GAPS）坏了也不 panic
            if let Some(live) = self.gaps.get_mut(..self.n_gaps) {
                live.copy_within(1.., 0);
            }
            self.n_gaps -= 1;
        }
    }

    fn note_gap(&mut self, len: i64) {
        let at = self.accepted;
        if let Some(last) = self
            .n_gaps
            .checked_sub(1)
            .and_then(|i| self.gaps.get_mut(i))
            && last.at == at
        {
            last.len += len;
            return;
        }
        if let Some(slot) = self.gaps.get_mut(self.n_gaps) {
            *slot = Gap { at, len };
            self.n_gaps += 1;
        } else if let Some(last) = self.gaps.last_mut() {
            last.len += len;
            self.stats.gap_overflow += 1;
        }
    }

    /// `x.len() == y.len() ≤ max_block`
    fn chunk<R, P, H>(&mut self, x: &[f32], y: &mut [f32], raw: &mut R, processed: &mut P, hb: &H)
    where
        R: Push,
        P: Pop,
        H: Heartbeat + ?Sized,
    {
        let n = x.len();
        let t0 = self.stats.samples as i64;
        // 1. 原声推 Raw；推不进去的记成缺口
        let pushed = raw.push(x).min(n);
        self.accepted += pushed as i64;
        let dropped = n - pushed;
        if dropped > 0 {
            self.stats.raw_overflow += dropped as u64;
            self.note_gap(dropped as i64);
        }

        // 2. 本地延迟原声；处理后的先默认成它（缺的位置就是原声）
        match (
            self.raw_delayed.get_mut(..n),
            self.proc_buf.get_mut(..n),
            self.have.get_mut(..n),
        ) {
            (Some(rd), Some(pb), Some(hv)) => {
                self.delay.process(x, rd);
                pb.copy_from_slice(rd);
                hv.fill(MISSING);
            }
            _ => return,
        }

        // 3. 心跳
        let beat = hb.heartbeat();
        match self.last_beat {
            Some(b) if b == beat => {
                self.since_beat = self.since_beat.saturating_add(n);
                if self.fresh && self.since_beat > self.cfg.timeout {
                    self.fresh = false;
                    self.stats.stale_switches += 1;
                    self.stats.last_stale_at = Some(self.stats.samples);
                }
            }
            Some(_) => {
                self.last_beat = Some(beat);
                self.since_beat = 0;
                if !self.fresh {
                    self.fresh = true;
                    self.stats.recoveries += 1;
                }
            }
            // 第一次看到：只记基准，等它动了才算新鲜
            None => {
                self.last_beat = Some(beat);
                self.since_beat = 0;
            }
        }

        // 3b. 降噪进程跳过了积压（带外：缺口计数）：接下来一个 L_total 窗口里有零占位，这段送原声
        let g = hb.gaps();
        if self.last_gaps.is_some_and(|last| g > last) {
            self.gap_hold = self.cfg.l_total + 2 * self.cfg.max_block;
            self.stats.gap_holds += 1;
        }
        self.last_gaps = Some(g);

        // 4. 按对齐账取处理后的
        let end = t0 + n as i64;
        let mut t = t0;
        let mut missing = 0usize;
        while t < end {
            let ht = self.head_time();
            let run = self.run_to_gap();
            if ht < t {
                // 迟到：丢掉（不跨缺口，跨了时刻会跳）
                let want = usize::try_from((t - ht).min(run).max(1)).unwrap_or(1);
                let d = self.discard(processed, want);
                self.advance(d);
                if d < want {
                    // 没有更多了
                    missing += usize::try_from(end - t).unwrap_or(0);
                    break;
                }
                continue;
            }
            if ht > t {
                // 超前（前面有缺口）：这段用原声
                let h = (ht - t).min(end - t);
                missing += usize::try_from(h).unwrap_or(0);
                t += h;
                continue;
            }
            let i = usize::try_from(t - t0).unwrap_or(0);
            let k = usize::try_from((end - t).min(run).max(1)).unwrap_or(1);
            let got = match self.proc_buf.get_mut(i..i + k) {
                Some(b) => processed.pop(b).min(k),
                None => 0,
            };
            // 推进序号 < 0 的是占位样本（开头 lead 个零 + 管线起振的 L_pipe 个输出），不是真声音
            let placeholders = usize::try_from(-self.head_acc).unwrap_or(0).min(got);
            if let (Some(hv), Some(pb), Some(rd)) = (
                self.have.get_mut(i..i + got),
                self.proc_buf.get_mut(i..i + placeholders),
                self.raw_delayed.get(i..i + placeholders),
            ) {
                hv.fill(OK);
                if let Some(ph) = hv.get_mut(..placeholders) {
                    ph.fill(PLACEHOLDER);
                }
                pb.copy_from_slice(rd);
            }
            self.advance(got);
            t += got as i64;
            if got < k {
                missing += usize::try_from(end - t).unwrap_or(0);
                break;
            }
        }
        if self.fresh {
            self.stats.proc_underrun += missing as u64;
        } else {
            self.stats.stale_samples += n as u64;
        }

        // 5. 混合（逐样本）：
        //    - 心跳过期 → 按 `fade` 淡到原声（处理后的还有，能淡）
        //    - 该用处理后的位置却没有 → 只能立刻切原声（硬切），落差用接缝偏移抹平
        //    - 回滞：连续取到 `fade` 个处理后样本、且心跳新鲜，才往回淡
        let fade = self.cfg.fade;
        let timeout = self.cfg.timeout;
        let rd = self.raw_delayed.get(..n).unwrap_or(&[]);
        for (((yi, &r), &p), &has) in y
            .iter_mut()
            .zip(rd.iter())
            .zip(self.proc_buf.iter())
            .zip(self.have.iter())
        {
            let mut cut = core::mem::take(&mut self.pending_cut);
            if has == OK {
                self.ok_run = self.ok_run.saturating_add(1);
                self.late_run = 0;
            } else if has == PLACEHOLDER {
                // 占位：照常用原声，不算迟到
                self.ok_run = 0;
                self.late_run = 0;
                if self.gain > 0.0 {
                    cut = true;
                }
            } else {
                self.ok_run = 0;
                if self.fresh {
                    self.late_run = self.late_run.saturating_add(1);
                    if self.late_run == timeout {
                        self.stats.slow_episodes += 1;
                    }
                }
                if self.gain > 0.0 {
                    cut = true;
                }
            }
            if cut {
                self.gain = 0.0;
                self.ok_run = 0;
                self.stats.cuts += 1;
            }
            let held = self.gap_hold > 0;
            if held {
                self.gap_hold -= 1;
                self.ok_run = 0;
            }
            let target = if self.fresh && !held && self.ok_run >= fade {
                1.0
            } else {
                0.0
            };
            // 防御：处理后的不是有限值就当没有（只用原声）
            let p = if p.is_finite() {
                p
            } else {
                self.stats.nonfinite += 1;
                if self.gain > 0.0 && !cut {
                    cut = true;
                    self.gain = 0.0;
                    self.stats.cuts += 1;
                }
                r
            };
            if self.gain < target {
                self.gain = (self.gain + self.step).min(1.0);
            } else if self.gain > target {
                self.gain = (self.gain - self.step).max(0.0);
            }
            if cut {
                // 切到原声：输出原来比原声高 / 低多少，就从这里开始把这段落差慢慢收掉
                self.seam = self.last_y - self.last_r;
            }
            let v = self.gain * p + (1.0 - self.gain) * r + self.seam;
            self.seam *= self.seam_decay;
            self.last_y = v;
            self.last_r = r;
            *yi = v;
        }
        self.stats.samples += n as u64;

        // 6. 积压上限（迟到的已在第 4 步按账丢过）。同一条总线上正常不会超（积压 ≤ lead）；
        //    实际触发多是送音端重启、接到一条 Raw 积满的旧总线。全部丢掉，之后新到的当作「从下一块起对齐」，
        //    并强制下一个样本硬切到原声、再按回滞淡回（对上的账是猜的，不能直接满权重播）
        let backlog = processed.available();
        if backlog > self.cfg.max_backlog {
            let d = self.discard(processed, backlog);
            self.advance(d);
            self.head_shift += end - self.head_time();
            self.n_gaps = 0;
            self.stats.resyncs += 1;
            self.pending_cut = self.gain > 0.0;
            self.ok_run = 0;
        }
    }

    /// 从 Processed 丢掉至多 `want` 个样本，返回丢了多少。
    fn discard<P: Pop>(&mut self, processed: &mut P, want: usize) -> usize {
        let d = processed.discard(want).min(want);
        self.stats.discarded += d as u64;
        d
    }
}
