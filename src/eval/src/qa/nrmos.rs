//! 无参考听感分（评分模型）：SCOREQ（主）与 Distill-MOS（交叉），都在 Rust 里用 ort 跑 ONNX。
//!
//! - **SCOREQ**（natural · NR，上游包把 natural 映射到 `adapt_nr_telephone.onnx`，`scoreq.py:93`）：
//!   16 kHz 单声道 → 尾部补零到 320 的整数倍 → 一次前向出一个分（`scoreq.py` `_predict_onnx` + `dynamic_pad`）
//! - **Distill-MOS**（v7）：上游只有 PyTorch 权重；本线用 `tools/export_distillmos.py` 把「不分段」的前向
//!   （固定 122,880 样本，含逐段峰值归一）导出成 ONNX，分段与求均值照上游 `sqa.py` forward 在这里做：
//!   不够长补零；超出部分按 `linspace(0, 1, ceil(超出 / 16000) + 1)` 取起点，各段分数取平均
//!
//! 模型文件不进仓：路径按环境变量 → 默认位置找，sha256 必须等于下面的常数（模型身份的正本），对不上就不用。
//! 对拍：`tests/nrmos_parity.rs`（与上游 Python 结果差 ≤ 0.01）。ort 一个进程只许一个环境，QA 层在子进程里跑
//!（`vdn-eval nrmos <scoreq|distill> <模型> <wav>`，同 DNSMOS）。

use std::path::{Path, PathBuf};

use ort::session::Session;
use ort::session::builder::{AutoDevicePolicy, GraphOptimizationLevel};
use ort::value::Tensor;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// 两个评分模型。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Scoreq,
    Distill,
}

impl Kind {
    pub const ALL: [Kind; 2] = [Kind::Scoreq, Kind::Distill];

    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Scoreq => "scoreq",
            Kind::Distill => "distill",
        }
    }

    pub fn parse(s: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.as_str() == s)
    }

    /// 对应的 verifier id（`verifiers::REGISTRY`）。
    pub fn verifier(self) -> &'static str {
        match self {
            Kind::Scoreq => "V-NRMOS-SCOREQ",
            Kind::Distill => "V-NRMOS-DISTILL",
        }
    }

    /// 覆盖默认路径的环境变量。
    pub fn env_var(self) -> &'static str {
        match self {
            Kind::Scoreq => "VDN_SCOREQ_ONNX",
            Kind::Distill => "VDN_DISTILLMOS_ONNX",
        }
    }

    /// 默认位置（相对家目录）。
    pub fn default_rel(self) -> &'static str {
        match self {
            Kind::Scoreq => ".cache/scoreq/onnx-models/adapt_nr_telephone.onnx",
            Kind::Distill => "vdn-data/models/distillmos/distill_mos_v7-seg122880.onnx",
        }
    }

    /// 模型文件 sha256（正本）。SCOREQ 与 exp009 README 记录一致；Distill-MOS 是本线导出物的哈希。
    pub fn sha256(self) -> &'static str {
        match self {
            Kind::Scoreq => "d493ba8acfe37d0aa0acceb85a41f22405fc4b9a5101359a6363d919307300b4",
            Kind::Distill => DISTILL_SHA256,
        }
    }
}

/// Distill-MOS 导出物的哈希（`tools/export_distillmos.py` 2026-11-03 导出：distillmos 0.9.1 · torch 2.15.0 · opset 17 · 17,613,491 B；原模型 ↔ ONNX 最大差 1.4e−6）。
pub const DISTILL_SHA256: &str = "b8e2dc186286377ddd5145698446ce949933c356a306a758eb986c0f881d2928";

/// SCOREQ 补零的倍数（wav2vec 2.0 卷积前端总步长）。
pub const SCOREQ_PAD: usize = 320;
/// Distill-MOS 一段的样本数与最大跳长。
pub const DISTILL_SEQ: usize = 122_880;
pub const DISTILL_HOP: usize = 16_000;

/// 找模型文件：环境变量优先，其次家目录下的默认位置。只看存不存在，不核哈希（核哈希见 [`verify`]）。
pub fn locate(k: Kind) -> Option<PathBuf> {
    if let Ok(p) = std::env::var(k.env_var()) {
        let p = PathBuf::from(p);
        return p.exists().then_some(p);
    }
    let home = std::env::var("HOME").ok()?;
    let p = Path::new(&home).join(k.default_rel());
    p.exists().then_some(p)
}

/// 文件哈希与正本一致吗。
pub fn verify(k: Kind, path: &Path) -> Result<(), String> {
    let got = crate::wav::sha256(path)?;
    if got == k.sha256() {
        Ok(())
    } else {
        Err(format!(
            "{} 的模型哈希 {got} ≠ 登记的 {}",
            k.as_str(),
            k.sha256()
        ))
    }
}

/// SCOREQ 的输入：尾部补零到 320 的整数倍。
pub fn scoreq_input(x: &[f32]) -> Vec<f32> {
    let mut a = x.to_vec();
    a.resize(x.len().div_ceil(SCOREQ_PAD) * SCOREQ_PAD, 0.0);
    a
}

/// Distill-MOS 的分段起点（上游 `sqa.py` forward：`int(c · 超出)`，c 取 linspace(0, 1, 段数)）。
/// 返回补零后的长度与各段起点。
pub fn distill_segments(len: usize) -> (usize, Vec<usize>) {
    if len <= DISTILL_SEQ {
        return (DISTILL_SEQ, vec![0]);
    }
    let over = len - DISTILL_SEQ;
    let hops = over.div_ceil(DISTILL_HOP) + 1;
    // numpy linspace 的算法：c = i · (1 / (n − 1))，最后一个强制等于 1（逐位一致，免得取整差一个样本）
    let step = 1.0 / (hops - 1) as f64;
    let starts = (0..hops)
        .map(|i| {
            let c = if i == hops - 1 { 1.0 } else { i as f64 * step };
            (c * over as f64) as usize
        })
        .collect();
    (len, starts)
}

pub struct Nrmos {
    kind: Kind,
    session: Session,
}

impl Nrmos {
    pub fn open(kind: Kind, path: &Path) -> Result<Nrmos, BoxError> {
        let env = ort::init()
            .with_name(format!("vdn-eval-{}", kind.as_str()))
            .build()?;
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
        Ok(Nrmos { kind, session })
    }

    fn forward(&mut self, x: Vec<f32>) -> Result<f64, BoxError> {
        let n = x.len();
        let t = Tensor::from_array((vec![1usize, n], x))?;
        let outs = self.session.run(ort::inputs![t])?;
        let first = outs.values().next().ok_or("评分模型没有输出")?;
        let (_, v) = first.try_extract_tensor::<f32>()?;
        let s = *v.first().ok_or("评分模型输出为空")? as f64;
        if !s.is_finite() {
            return Err(format!("评分模型输出不是有限数：{s}").into());
        }
        Ok(s)
    }

    /// 16 kHz 单声道整段打一个分（MOS，1–5 附近）。
    pub fn score(&mut self, x: &[f32]) -> Result<f64, BoxError> {
        if x.is_empty() {
            return Err("空音频".into());
        }
        match self.kind {
            Kind::Scoreq => self.forward(scoreq_input(x)),
            Kind::Distill => {
                let (len, starts) = distill_segments(x.len());
                let mut a = x.to_vec();
                a.resize(len, 0.0);
                let mut sum = 0.0;
                for &s in &starts {
                    sum += self.forward(a[s..s + DISTILL_SEQ].to_vec())?;
                }
                Ok(sum / starts.len() as f64)
            }
        }
    }
}

/// 子进程入口：`vdn-eval nrmos <scoreq|distill> <模型.onnx> <音频.wav>…` → 每段一行 JSON `{"wav", "mos"}`。
/// 一次进程打多段，模型只载一次（SCOREQ 378 MB）。哈希对不上直接报错，不打分。
pub fn cli(args: &[String]) -> Result<Vec<serde_json::Value>, String> {
    let usage = "用法：vdn-eval nrmos <scoreq|distill> <模型.onnx> <音频.wav>…";
    let kind = args.first().and_then(|s| Kind::parse(s)).ok_or(usage)?;
    let model = args.get(1).ok_or(usage)?;
    if args.len() < 3 {
        return Err(usage.into());
    }
    verify(kind, Path::new(model))?;
    let mut m = Nrmos::open(kind, Path::new(model)).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for w in &args[2..] {
        let (x, sr) = crate::wav::read(Path::new(w))?;
        if sr != vdn_types::MODEL_RATE {
            return Err(format!("{w}：只支持 16 kHz，实际 {sr}"));
        }
        let mos = m.score(&x).map_err(|e| format!("{w}：{e}"))?;
        out.push(serde_json::json!({"wav": w, "mos": mos}));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scoreq_pads_to_320() {
        assert_eq!(scoreq_input(&[0.1; 320]).len(), 320);
        assert_eq!(scoreq_input(&[0.1; 321]).len(), 640);
        let a = scoreq_input(&[0.5; 10]);
        assert_eq!(&a[..10], &[0.5; 10]);
        assert!(a[10..].iter().all(|v| *v == 0.0));
    }

    #[test]
    fn distill_segments_match_upstream() {
        // 短于一段：补零、一段
        assert_eq!(distill_segments(48_000), (DISTILL_SEQ, vec![0]));
        assert_eq!(distill_segments(DISTILL_SEQ), (DISTILL_SEQ, vec![0]));
        // 10.6 s：超出 46,720 → ceil(2.92) + 1 = 4 段，起点 0 · 15573 · 31146 · 46720
        let (len, s) = distill_segments(169_600);
        assert_eq!(len, 169_600);
        assert_eq!(s, vec![0, 15_573, 31_146, 46_720]);
        // 最后一段正好到结尾
        assert_eq!(s.last().unwrap() + DISTILL_SEQ, len);
    }

    #[test]
    fn kinds_round_trip() {
        for k in Kind::ALL {
            assert_eq!(Kind::parse(k.as_str()), Some(k));
            assert!(
                crate::verifiers::info(k.verifier()).is_some(),
                "{}",
                k.verifier()
            );
        }
    }
}
