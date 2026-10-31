//! vdn-scene：场景识别与路由（D40 · D41）。感知 → 细场景 → 用户层场景 → 参数组；滞回与平滑过渡。
//!
//! ```text
//!  麦克风原声（模型采样率 16 kHz）
//!     │ 每 16 ms：FeatureExtractor::push_hop      ← 逐帧，一次 FFT + 一次逆 FFT，不分配
//!     ▼
//!  2 s 环形缓冲
//!     │ 约每 0.5 s：SceneAnalyzer::judge           ← 判断回路（C 档线程），不碰声音、不加延迟
//!     ▼
//!  27 维特征 → 分类器后验（7 个噪音大类）→ 后验平滑（EMA）
//!           + 设备线索（窄带蓝牙 / 削波）
//!     ▼
//!  细场景 = 大类 × 设备（21 个 + 多人会议占位）→ SceneLabel
//!     ▼
//!  HysteresisGate（连续确认、最短停留）→ 已生效细场景 → route::params_for → ParamSet（模型 + 修音链预设）
//!     ▼
//!  engine：连续参数按 Transition.glide_s 平滑；换模型交叉淡化（规约 §3.5，下一批接入）
//! ```
//!
//! **算力**（`examples/scene_offline.rs` 在本机实测，见 exp006 README）：逐帧部分与判断部分都在微秒量级，
//! 相对 16 ms 的 hop 与 0.5 s 的判断周期可忽略。**延迟**：0（只改参数，不在音频路径上）。
//!
//! 用户控制（手动选 / 锁定用户层场景、旋钮）按 vdn-types 草案的 `resolve_params` 叠在路由表之上，本 crate 只给
//! [`SceneAnalyzer::set_manual`] 的最小版本：手动选了用户层场景后，自动判断只在该场景下属的细场景里挑。

pub mod classify;
pub mod features;
pub mod mirror;
pub mod route;
pub mod weights;

use classify::{NoiseClass, posterior};
use features::{FeatureExtractor, NF};
use mirror::{Confidence, FineSceneId, HysteresisGate, ParamSet, SampleClock, SceneLabel, UserScene};
use route::Device;
use weights::NC;

/// 一次判断的全部结果（离线评测与决策日志用）。
#[derive(Clone, Copy, Debug)]
pub struct Judgement {
    /// 本次原始判断（平滑后取最大）
    pub label: SceneLabel,
    /// 未平滑的后验
    pub raw: [f32; NC],
    /// 平滑后的后验
    pub smooth: [f32; NC],
    pub device: Device,
    /// 滞回后生效的场景
    pub committed: (UserScene, FineSceneId),
    pub switched: Option<(FineSceneId, FineSceneId)>,
    /// 生效场景的参数组
    pub params: ParamSet,
    pub features: [f32; NF],
}

pub struct SceneAnalyzer {
    fx: FeatureExtractor,
    feats: [f32; NF],
    ema: [f32; NC],
    bt: f32,
    clip: f32,
    have: bool,
    gate: HysteresisGate,
    clock: u64,
    rate: u32,
    ema_s: f32,
    last_judge: u64,
    manual: Option<UserScene>,
}

impl SceneAnalyzer {
    /// `rate` = 喂进来的采样率（目前只支持 16000），`hop` = 每次 `push_hop` 的样本数（256）。
    pub fn new(rate: u32, hop: usize) -> Self {
        Self {
            fx: FeatureExtractor::new(rate as f32, hop),
            feats: [0.0; NF],
            ema: [1.0 / NC as f32; NC],
            bt: 0.0,
            clip: 0.0,
            have: false,
            gate: HysteresisGate::new(weights::HYSTERESIS),
            clock: 0,
            rate,
            ema_s: weights::EMA_S,
            last_judge: 0,
            manual: None,
        }
    }

    /// 改滞回参数与后验平滑时间常数（标定扫参用）。
    pub fn set_tuning(&mut self, h: mirror::Hysteresis, ema_s: f32) {
        self.gate = HysteresisGate::new(h);
        self.ema_s = ema_s;
    }

    /// 手动选用户层场景（`None` = 自动）。
    pub fn set_manual(&mut self, u: Option<UserScene>) {
        self.manual = u;
    }

    pub fn reset(&mut self) {
        self.fx.reset();
        self.ema = [1.0 / NC as f32; NC];
        self.bt = 0.0;
        self.clip = 0.0;
        self.have = false;
        self.gate.reset();
        self.clock = 0;
        self.last_judge = 0;
    }

    /// 逐帧：喂一个 hop 的麦克风原声。
    pub fn push_hop(&mut self, x: &[f32]) {
        self.fx.push_hop(x);
        self.clock += x.len() as u64;
    }

    /// 判断回路：算特征 → 分类 → 平滑 → 滞回 → 查表。约 0.5 s 调一次。
    pub fn judge(&mut self) -> Judgement {
        self.fx.compute(&mut self.feats);
        let mut raw = [0.0f32; NC];
        posterior(&self.feats, &mut raw);
        let dt = (self.clock - self.last_judge) as f32 / self.rate as f32;
        self.last_judge = self.clock;
        let a = if !self.have || self.ema_s <= 0.0 { 1.0 } else { 1.0 - (-dt / self.ema_s).exp() };
        let bt_now = if self.feats[8] < weights::BT_HF_P90_MAX { 1.0 } else { 0.0 };
        let clip_now = if self.feats[21] > weights::CLIP_MIN { 1.0 } else { 0.0 };
        for (e, r) in self.ema.iter_mut().zip(&raw) {
            *e += a * (r - *e);
        }
        self.bt += a * (bt_now - self.bt);
        self.clip += a * (clip_now - self.clip);
        self.have = true;
        // 取最大（手动选了用户层场景时只在其下属大类里取）
        let mut best = 0usize;
        let mut best_p = -1.0f32;
        let mut mass = 0.0f32;
        for (i, &p) in self.ema.iter().enumerate() {
            let c = NoiseClass::ALL[i];
            if let Some(u) = self.manual {
                if route::user_of(c) != u {
                    continue;
                }
            }
            mass += p;
            if p > best_p {
                best_p = p;
                best = i;
            }
        }
        let class = NoiseClass::ALL[best];
        let device = if self.clip > 0.5 {
            Device::Clipping
        } else if self.bt > 0.5 {
            Device::BtNarrow
        } else {
            Device::Normal
        };
        let conf = if self.manual.is_some() { best_p / mass.max(1e-6) } else { best_p };
        let (fine, user) = match self.manual {
            Some(UserScene::Meeting) => (route::MEETING, UserScene::Meeting),
            _ => (route::fine_id(class, device), route::user_of(class)),
        };
        let label = SceneLabel { user, fine, confidence: Confidence::new(conf), at: SampleClock(self.clock) };
        let switched = self.gate.step(&label, self.rate);
        let committed = self.gate.committed();
        Judgement {
            label,
            raw,
            smooth: self.ema,
            device,
            committed,
            switched,
            params: route::params_for(committed.1),
            features: self.feats,
        }
    }
}
