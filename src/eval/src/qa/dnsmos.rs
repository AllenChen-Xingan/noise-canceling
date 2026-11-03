//! DNSMOS P.835（`sig_bak_ovr.onnx`）在 Rust 里跑。口径照官方 `dnsmos_local.py`（读懂后自写，不照抄）：
//! 9.01 s 窗、1 s 跳；片段不够 9.01 s 时自我重复拼接到够长；每窗出原始 SIG / BAK / OVR，
//! 过非个性化的二次多项式映射后取各窗平均。P.808 那个模型（要 mel 特征）不跑。
//!
//! 多项式系数是官方脚本里的常数（事实，不是代码）。ort 单线程 CPU。

use std::path::Path;

use ort::session::Session;
use ort::session::builder::{AutoDevicePolicy, GraphOptimizationLevel};
use ort::value::Tensor;

const SR: usize = vdn_types::MODEL_RATE as usize;
const WIN: usize = 144_160; // 9.01 s
const HOP: usize = SR;

/// 非个性化映射：(SIG, BAK, OVRL) 各自的 [a2, a1, a0]。
const P_SIG: [f64; 3] = [-0.083_972_78, 1.220_839_53, 0.005_243_9];
const P_BAK: [f64; 3] = [-0.131_668_88, 1.609_155_14, -0.396_045_46];
const P_OVR: [f64; 3] = [-0.067_662_83, 1.115_464_68, 0.046_025_35];

fn poly(p: &[f64; 3], x: f64) -> f64 {
    p[0] * x * x + p[1] * x + p[2]
}

pub struct Dnsmos {
    session: Session,
}

#[derive(Clone, Copy, Debug)]
pub struct Mos {
    pub sig: f64,
    pub bak: f64,
    pub ovrl: f64,
    pub windows: usize,
}

type BoxError = Box<dyn std::error::Error + Send + Sync>;

impl Dnsmos {
    pub fn open(path: &Path) -> Result<Dnsmos, BoxError> {
        let env = ort::init().with_name("vdn-eval-dnsmos").build()?;
        let b = |e: ort::Error<ort::session::builder::SessionBuilder>| -> BoxError {
            e.to_string().into()
        };
        let session = Session::builder(&env)?
            .with_auto_device(AutoDevicePolicy::PreferCPU)
            .map_err(b)?
            .with_optimization_level(GraphOptimizationLevel::All)
            .map_err(b)?
            .with_intra_threads(1)
            .map_err(b)?
            .commit_from_file(path)?;
        Ok(Dnsmos { session })
    }

    pub fn score(&mut self, x: &[f32]) -> Result<Mos, BoxError> {
        if x.is_empty() {
            return Err("空音频".into());
        }
        let mut a = x.to_vec();
        while a.len() < WIN {
            let c = a.clone();
            a.extend_from_slice(&c);
        }
        // 窗数 = int(floor(len / 1 s) − 9.01) + 1：官方用 int() 向零截断，不是 floor（自审 P1）
        let n = ((a.len() / SR) as f64 - WIN as f64 / SR as f64).trunc() as isize + 1;
        let (mut s, mut b, mut o, mut k) = (0.0, 0.0, 0.0, 0usize);
        for i in 0..n.max(0) as usize {
            let from = i * HOP;
            if from + WIN > a.len() {
                continue;
            }
            let t = Tensor::from_array((vec![1usize, WIN], a[from..from + WIN].to_vec()))?;
            let outs = self.session.run(ort::inputs![t])?;
            let first = outs.values().next().ok_or("DNSMOS 没有输出")?;
            let (_, v) = first.try_extract_tensor::<f32>()?;
            if v.len() < 3 {
                return Err(format!("DNSMOS 输出长度 {}", v.len()).into());
            }
            s += poly(&P_SIG, v[0] as f64);
            b += poly(&P_BAK, v[1] as f64);
            o += poly(&P_OVR, v[2] as f64);
            k += 1;
        }
        if k == 0 {
            return Err("一个完整窗都没有".into());
        }
        let k_f = k as f64;
        Ok(Mos {
            sig: s / k_f,
            bak: b / k_f,
            ovrl: o / k_f,
            windows: k,
        })
    }
}
