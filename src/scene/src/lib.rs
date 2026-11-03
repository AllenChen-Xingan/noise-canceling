//! vdn-scene：场景识别与路由（D40 · D41）。感知 → 细场景 → 用户层场景 → 参数组；滞回与平滑过渡。
//!
//! ```text
//!  麦克风原声（模型采样率 16 kHz）
//!     │ 每 16 ms：FeatureExtractor::push_hop      ← 逐帧，一次 FFT + 一次逆 FFT，不分配
//!     ▼
//!  2 s 环形缓冲
//!     │ 约每 0.5 s：SceneAnalyzer::judge           ← 判断回路（C 档线程），不碰声音、不加延迟
//!     ▼
//!  35 维特征 → 分类器后验（7 个噪音大类）→ 后验平滑（EMA）
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

#![forbid(unsafe_code)]
pub mod classify;
pub mod features;
pub mod route;
#[rustfmt::skip]
pub mod weights;

use classify::{NoiseClass, posterior};
use features::{FeatureExtractor, NF};
use route::Device;
use vdn_types::{
    Confidence, FineSceneId, HysteresisGate, ParamSet, SampleClock, SceneLabel, UserScene,
};
use weights::NC;

/// 缓冲里至少有这么多帧（≈1 s）才把判断交给滞回。
const MIN_FILL: usize = 62;

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
    device: Device,
    last_committed: FineSceneId,
}

impl SceneAnalyzer {
    /// `rate` = 喂进来的采样率（目前只支持 16000），`hop` = 每次 `push_hop` 的样本数（256）。
    /// 本 crate 的 `SampleClock` 按这个 `rate` 计（离线 = 16 kHz 模型样本），滞回换算秒时传同一个 `rate`；
    /// `vdn-types` 约定流时间按 IO 采样率计，接入实时链路时把 `rate` 设成 IO 采样率、`at` 用 IO 时钟（两者必须同口径，错了确认与停留时长会偏 3 倍）。
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
            device: Device::Normal,
            last_committed: FineSceneId::UNKNOWN,
        }
    }

    /// 改滞回参数与后验平滑时间常数（标定扫参用）。两者都按描述表（`vdn_types::desc`）夹进范围；
    /// 要拒收越界值，调用方先调 `Hysteresis::validate` / `SCENE_EMA_S.check`。
    pub fn set_tuning(&mut self, h: vdn_types::Hysteresis, ema_s: f32) {
        self.gate = HysteresisGate::new(h.clamped());
        self.ema_s = vdn_types::desc::SCENE_EMA_S.clamp(ema_s);
    }

    /// 当前的滞回参数与后验平滑时间常数。
    pub fn tuning(&self) -> (vdn_types::Hysteresis, f32) {
        (self.gate.params, self.ema_s)
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
        self.device = Device::Normal;
        self.last_committed = FineSceneId::UNKNOWN;
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
        let warm = self.fx.filled() >= 8; // 不足 8 帧时特征全 0，不拿它当后验平滑的初值
        let a = if !warm {
            0.0
        } else if !self.have || self.ema_s <= 0.0 {
            1.0
        } else {
            1.0 - (-dt / self.ema_s).exp()
        };
        let bt_now = if self.feats[8] < weights::BT_HF_P90_MAX {
            1.0
        } else {
            0.0
        };
        let clip_now = if self.feats[21] > weights::CLIP_MIN {
            1.0
        } else {
            0.0
        };
        for (e, r) in self.ema.iter_mut().zip(&raw) {
            *e += a * (r - *e);
        }
        self.bt += a * (bt_now - self.bt);
        self.clip += a * (clip_now - self.clip);
        self.have |= warm;
        // 取最大（手动选了用户层场景时只在其下属大类里取）
        let mut best = 0usize;
        let mut best_p = -1.0f32;
        let mut mass = 0.0f32;
        for (i, &p) in self.ema.iter().enumerate() {
            let c = NoiseClass::ALL[i];
            if let Some(u) = self.manual
                && route::user_of(c) != u
            {
                continue;
            }
            mass += p;
            if p > best_p {
                best_p = p;
                best = i;
            }
        }
        let class = NoiseClass::ALL[best];
        // 设备：施密特触发（升过 0.7 才认、降过 0.3 才撤），与噪音大类分开滞回——设备是物理的，很少变
        self.device = match self.device {
            Device::Clipping if self.clip > 0.3 => Device::Clipping,
            Device::BtNarrow if self.bt > 0.3 && self.clip <= 0.7 => Device::BtNarrow,
            _ if self.clip > 0.7 => Device::Clipping,
            _ if self.bt > 0.7 => Device::BtNarrow,
            _ => Device::Normal,
        };
        let device = self.device;
        let mut conf = if self.manual.is_some() {
            best_p / mass.max(1e-6)
        } else {
            best_p
        };
        if self.fx.filled() < MIN_FILL {
            conf = 0.0; // 缓冲不满 1 s 的判断不进滞回（置信度 0 = 不推进）
        }
        if self.manual == Some(UserScene::Meeting) {
            conf = 1.0; // 会议下没有噪音大类可判：人手选的直接算确定（自审 P1：否则永远提交不了）
        }
        // 滞回只看噪音大类（设备另有施密特触发），否则设备估计一抖，候选就从头确认
        let (fine, user) = match self.manual {
            Some(UserScene::Meeting) => (route::MEETING, UserScene::Meeting),
            _ => (route::fine_id(class, Device::Normal), route::user_of(class)),
        };
        let at = SampleClock(self.clock);
        let gate_label = SceneLabel {
            user,
            fine,
            confidence: Confidence::new(conf),
            at,
        };
        let class_switch = self.gate.step(&gate_label, self.rate);
        let (cu, cf) = self.gate.committed();
        let committed_fine = match route::decode(cf) {
            Some((c, _)) => route::fine_id(c, device),
            None => cf,
        };
        let switched = if class_switch.is_some()
            || (committed_fine != self.last_committed && cf != FineSceneId::UNKNOWN)
        {
            Some((self.last_committed, committed_fine))
        } else {
            None
        };
        self.last_committed = committed_fine;
        let label = SceneLabel {
            fine: match self.manual {
                Some(UserScene::Meeting) => route::MEETING,
                _ => route::fine_id(class, device),
            },
            ..gate_label
        };
        Judgement {
            label,
            raw,
            smooth: self.ema,
            device,
            committed: (cu, committed_fine),
            switched,
            params: route::params_for(committed_fine),
            features: self.feats,
        }
    }
}
