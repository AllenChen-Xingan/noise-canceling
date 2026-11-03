//! tract（纯 Rust ONNX 推理，sonos/tract 0.23.9）后端：通用「第 i 个输出回灌第 i 个输入」的流式网络。
//!
//! 约定（GTCRN 与 FastEnhancer 的 ONNX 都满足）：输入 0 是本帧数据，输出 0 是本帧结果，
//! 输入 i ≥ 1 是缓存，下一帧用输出 i 覆盖。输入形状由调用方给（从 ORT 会话读，同一份 ONNX）。
//! tract 默认单线程（不设 executor），与 ORT 基线 intra/inter = 1 同口径。

use std::path::Path;

use tract_onnx::prelude::*;

pub type Err = Box<dyn std::error::Error + Send + Sync>;

pub struct TractNet {
    state: TypedSimpleState,
    ins: Vec<Tensor>,
}

impl TractNet {
    pub fn open(path: &Path, shapes: &[Vec<usize>]) -> Result<Self, Err> {
        let mut m = tract_onnx::onnx().model_for_path(path)?;
        for (i, s) in shapes.iter().enumerate() {
            m.set_input_fact(i, f32::fact(s).into())?;
        }
        let opt = if std::env::var_os("EXP010_TRACT_DECLUTTER").is_some() {
            m.into_typed()?.into_decluttered()?
        } else {
            m.into_optimized()?
        };
        if std::env::var_os("EXP010_DEBUG").is_some() {
            for (k, o) in opt.output_outlets()?.iter().enumerate() {
                eprintln!("tract out {k}: {} {:?}", opt.node(o.node).name, opt.outlet_fact(*o)?);
            }
            for n in opt.nodes() {
                if n.op().state(&Default::default()).map(|s| s.is_some()).unwrap_or(false) {
                    eprintln!("tract 有状态算子: {} {}", n.name, n.op().name());
                }
            }
            for (k, o) in opt.input_outlets()?.iter().enumerate() {
                eprintln!("tract in {k}: {} {:?}", opt.node(o.node).name, opt.outlet_fact(*o)?);
            }
        }
        let plan = opt.into_runnable()?;
        let state = plan.spawn()?;
        let ins = shapes
            .iter()
            .map(|s| Tensor::zero::<f32>(s))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { state, ins })
    }

    /// 输入 0 的可写视图（本帧数据）。
    pub fn input0(&mut self) -> &mut [f32] {
        // 安全：ins 全部由 Tensor::zero::<f32> 构造，类型必为 f32、内存连续
        unsafe { self.ins[0].as_slice_mut_unchecked::<f32>() }
    }

    /// 跑一帧：输出 0 写进 `out0`，其余输出回灌缓存。
    pub fn step(&mut self, out0: &mut [f32]) -> Result<(), Err> {
        let inputs: TVec<TValue> = self.ins.iter().map(|t| t.clone().into_tvalue()).collect();
        // 每帧新开一个状态：复用同一个 SimpleState 时，第 1 帧起输出就和 ORT 对不上（实测 SNR 19 dB），
        // 新开则一致（130 dB）。代价实测 p50 +0.02 ms。EXP010_TRACT_REUSE=1 可复现错误的复用版。
        if std::env::var_os("EXP010_TRACT_REUSE").is_none() {
            self.state = self.state.plan().spawn()?;
        }
        let outs = self.state.run(inputs)?;
        if outs.len() != self.ins.len() {
            return Err("tract 输出个数 ≠ 输入个数".into());
        }
        out0.copy_from_slice(outs[0].try_as_plain_ram()?.as_slice::<f32>()?);
        for (dst, src) in self.ins[1..].iter_mut().zip(&outs[1..]) {
            // 安全：同上，dst 是 f32 张量
            unsafe { dst.as_slice_mut_unchecked::<f32>() }
                .copy_from_slice(src.try_as_plain_ram()?.as_slice::<f32>()?);
        }
        Ok(())
    }
}
