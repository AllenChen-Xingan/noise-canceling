//! `Pipeline` 的实时路径（规约 §5.3、R1–R4）：处理、模式切换、复位与只读访问。不分配、不加锁、不 panic。

use vdn_dsp::FtzGuard;
use vdn_dsp::resample;
use vdn_types::{BypassReason, FrameCtx, Model, ParamSet, SampleClock, StageChain, Status};

use super::{Mode, Pipeline, PipelineStats};
use crate::engine::{Engine, sanitize_input};

// 实时（规约 §5.3、R1–R4）
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl<M: Model, S: StageChain> Pipeline<M, S> {
    /// 整条链的延迟（IO 采样率样本）。两种模式相同。
    pub fn latency_samples(&self) -> usize {
        self.latency
    }

    pub fn io_rate(&self) -> u32 {
        self.io_rate
    }

    /// 48 kHz hop（= 模型 hop × 3）。
    pub fn hop_io(&self) -> usize {
        self.hop_io
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// 切换模式：从下一个样本起，按一个 48 kHz hop 的时长线性淡化过去。
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn stats(&self) -> PipelineStats {
        self.stats
    }

    pub fn engine(&self) -> &Engine<M> {
        &self.engine
    }

    pub fn stages(&self) -> &S {
        &self.stages
    }

    /// 把策略参数下发给每个后处理级（实时线程可调，不分配）。
    pub fn set_stage_params(&mut self, p: &ParamSet) {
        self.stages.set_params_all(p);
    }

    /// 最近一个 48k hop 的状态（**输入侧**：刚攒满、跑完推理的那个 hop，比耳朵听到的早约一个 hop；一次调用跑了几个 hop
    /// 时只留最后一个——要数次数看 `engine().stats()` 与 `stats().stage_faults`）：Engine 回报的 `Ok` / `Bypassed` / `Holding`；Engine 已恢复（`Ok`）但
    /// Pipeline 还在把 48 kHz 原声淡回模型输出时报 `Crossfading`。
    pub fn last_status(&self) -> Status {
        match self.last_status {
            Status::Ok | Status::Crossfading if self.fb_ratio > 0.0 => Status::Crossfading,
            Status::Crossfading => Status::Ok,
            s => s,
        }
    }

    /// 清空所有状态（含 Engine 与模型），回到刚构造时；模式保持，淡化直接落到目标。
    pub fn reset(&mut self) {
        self.engine.reset();
        self.stages.reset_all();
        self.q_bypass.fill(false);
        self.fb_ratio = 0.0;
        self.seam = 0.0;
        self.last_mix = 0.0;
        self.last_dry = 0.0;
        self.last_status = Status::Ok;
        self.prev_bypass = false;
        // clock 不归零：流时间单调，reset 前发出的声纹判决的时间戳不会跑到 now 之后
        self.dec.reset();
        self.int.reset();
        self.acc.fill(0.0);
        self.acc_len = 0;
        self.q.fill(0.0);
        self.q_read = 0;
        self.q_len = self.prefill;
        self.dry_line.reset();
        self.fade_pos = self.fade_target();
    }

    /// 处理任意长度的一块。`output.len()` 应等于 `input.len()`（调用方的 bug：debug 下 panic；
    /// release 下只处理较短的那段，`output` 多出的部分清零并计数，不 panic）。
    ///
    /// `with_block` 构造时块长不等于 `block`：不 panic，计入 `stats().misaligned_blocks`，照常处理（宁可抖动也不断声）。
    /// 这时输出队列可能欠载，欠载的样本补零（计入 `queue_faults`），之后降噪路比声明延迟晚最多 `block - 1` 个样本、
    /// 与原声路错位，直到 `reset()`。
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        self.process_ctx(input, output, FrameCtx::default());
    }

    /// 同 `process`，另给后处理级一份上下文（声纹判决、人声概率）。`ctx.now` 不用：Pipeline 每 hop 按自己的样本计数填。
    pub fn process_ctx(&mut self, input: &[f32], output: &mut [f32], ctx: FrameCtx) {
        self.ctx = ctx;
        debug_assert_eq!(input.len(), output.len());
        let n = input.len().min(output.len());
        if self.block.is_some_and(|b| b != n) {
            self.stats.misaligned_blocks += 1;
        }
        let (head, tail) = output.split_at_mut(n);
        if !tail.is_empty() || input.len() != n {
            self.stats.bad_len += 1;
        }
        tail.fill(0.0);
        let (mut src, mut dst) = (input.get(..n).unwrap_or(&[]), head);
        while !src.is_empty() {
            let k = src.len().min(self.hop_io - self.acc_len);
            let (xi, rest_in) = src.split_at(k);
            let (yi, rest_out) = dst.split_at_mut(k);
            self.step(xi, yi);
            src = rest_in;
            dst = rest_out;
        }
    }

    /// `x.len() == y.len() <= hop_io - acc_len`。
    fn step(&mut self, x: &[f32], y: &mut [f32]) {
        let k = x.len();
        let start = self.acc_len;
        if let Some(slot) = self.acc.get_mut(start..start + k) {
            slot.copy_from_slice(x);
            self.stats.bad_input += sanitize_input(slot);
        }
        self.acc_len += k;
        self.clock += k as u64;
        if self.acc_len == self.hop_io {
            self.acc_len = 0;
            self.run_hop();
        }

        let _ftz = FtzGuard::new();
        self.pop(y);
        let target = self.fade_target();
        let full = self.hop_io;
        let dry = self.dry.get_mut(..k).unwrap_or(&mut []);
        // 原声路读入口已清 0 的那段（run_hop 只读 acc，不改它）
        let clean = self.acc.get(start..start + k).unwrap_or(&[]);
        self.dry_line.process(clean, dry);
        let flags = self.out_bypass.get(..k).unwrap_or(&[]);
        for ((o, d), &bypass) in y.iter_mut().zip(dry.iter()).zip(flags) {
            let d = *d;
            // 1. 模型级送原声：按 Engine 回报的状态，用 48 kHz 原声切换（规约 §3.3 规则 2、R11）
            if !o.is_finite() {
                *o = d;
            }
            if bypass {
                if self.fb_ratio < 1.0 {
                    // 切到原声：坏输出不能拿来淡化，用接缝偏移从上一个输出样本平滑过渡
                    self.seam = self.last_mix - self.last_dry;
                    if !self.seam.is_finite() {
                        self.seam = 0.0;
                    }
                    self.fb_ratio = 1.0;
                }
                *o = d + self.seam;
                self.seam *= self.seam_decay;
            } else if self.fb_ratio > 0.0 || self.seam != 0.0 {
                // 切回模型：线性交叉淡化
                *o += self.fb_ratio * (d - *o) + self.seam;
                self.fb_ratio = (self.fb_ratio - self.fb_step).max(0.0);
                self.seam *= self.seam_decay;
                if self.seam.abs() < 1e-9 {
                    self.seam = 0.0;
                }
            }
            self.last_mix = *o;
            self.last_dry = d;

            // 2. 模式（降噪 / 直通）淡化
            if self.fade_pos < target {
                self.fade_pos += 1;
            } else if self.fade_pos > target {
                self.fade_pos -= 1;
            }
            if self.fade_pos == 0 {
                continue;
            }
            if self.fade_pos >= full {
                *o = d;
            } else {
                let g = self.fade_pos as f32 / full as f32;
                *o += g * (d - *o);
            }
        }
    }

    fn fade_target(&self) -> usize {
        match self.mode {
            Mode::Denoise => 0,
            Mode::Passthrough => self.hop_io,
        }
    }

    /// 一个 48k hop：降采样 → Engine（推理）→ 后处理级 → 升采样 → 连同状态进输出队列。
    fn run_hop(&mut self) {
        self.stats.hops += 1;
        {
            let _ftz = FtzGuard::new();
            self.dec.process(&self.acc, &mut self.lo_in);
        }
        // 推理在 FTZ 守卫外（R9：守卫开着等于对 ORT 开 FTZ，DeepFilterNet #713）
        let mut status = self.engine.process_status(&self.lo_in, &mut self.lo_out);
        let mut bypass = matches!(status, Status::Bypassed(_) | Status::Holding);
        let ctx = FrameCtx {
            now: SampleClock(self.clock.saturating_sub(self.hop_io as u64)),
            ..self.ctx
        };
        {
            let _ftz = FtzGuard::new();
            self.stages
                .process_all(&mut self.lo_out, self.engine.aligned_dry(), ctx);
            // 后处理级的输出也按 Engine 的规则查（R11）：坏了整 hop 送原声、复位后处理级；坏值不进升采样器
            let cfg = self.engine.config();
            if cfg.fallback && !bypass {
                let mut peak = 0.0f32;
                let mut finite = true;
                for v in &self.lo_out {
                    finite &= v.is_finite();
                    peak = peak.max(v.abs());
                }
                if !finite || peak > cfg.max_abs {
                    status = Status::Bypassed(if finite {
                        BypassReason::Abnormal
                    } else {
                        BypassReason::NonFinite
                    });
                    bypass = true;
                    self.stats.stage_faults += 1;
                    self.lo_out.copy_from_slice(self.engine.aligned_dry());
                    self.stages.reset_all();
                }
            }
            self.int.process(&self.lo_out, &mut self.hi_out);
        }
        self.last_status = status;
        // 切回的第一个 hop：前 2 × delay_high 个样本里还有上一 hop 的 16 kHz 垫底（升采样延迟与拖尾），也标成送原声
        let tail = if self.prev_bypass && !bypass {
            2 * resample::delay_high()
        } else {
            0
        };
        self.prev_bypass = bypass;
        // 入队（容量 2 × hop_io ≥ prefill + hop_io，不会溢出）
        let cap = self.q.len();
        let mut w = (self.q_read + self.q_len) % cap;
        for (j, &v) in self.hi_out.iter().enumerate() {
            if let Some(s) = self.q.get_mut(w) {
                *s = v;
            }
            if let Some(b) = self.q_bypass.get_mut(w) {
                *b = bypass || j < tail;
            }
            w += 1;
            if w == cap {
                w = 0;
            }
        }
        let len = self.q_len + self.hop_io;
        debug_assert!(len <= cap, "输出队列溢出");
        if len > cap {
            // 到不了（见上）；万一溢出，最旧的样本已被覆盖，读指针跟着前移，保持 q 与 q_bypass 顺序一致
            let over = len - cap;
            self.stats.queue_faults += over as u64;
            self.q_read = (self.q_read + over) % cap;
        }
        self.q_len = len.min(cap);
    }

    /// 出队 `y.len()` 个样本，状态写进 `out_bypass` 的前 `y.len()` 个。
    fn pop(&mut self, y: &mut [f32]) {
        let cap = self.q.len();
        for (o, b) in y.iter_mut().zip(self.out_bypass.iter_mut()) {
            // 由预填保证不欠载；万一欠载补零（`new` 构造时不应发生；`with_block` 构造时块长不对会发生，不算 bug）
            debug_assert!(self.q_len > 0 || self.block.is_some(), "输出队列欠载");
            if self.q_len == 0 {
                self.stats.queue_faults += 1;
                *o = 0.0;
                *b = false;
                continue;
            }
            *o = self.q.get(self.q_read).copied().unwrap_or(0.0);
            *b = self.q_bypass.get(self.q_read).copied().unwrap_or(false);
            self.q_read += 1;
            if self.q_read == cap {
                self.q_read = 0;
            }
            self.q_len -= 1;
        }
    }
}
