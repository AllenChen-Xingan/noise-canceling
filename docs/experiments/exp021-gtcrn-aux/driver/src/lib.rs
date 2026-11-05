//! exp021 重混：把外部离线目录里的「模型原始输出」按指定的原声混回参数，
//! 交给**真引擎**（`vdn_engine::Engine`）重新混一遍，写成新的外部输出目录。
//!
//! 混回逻辑只有一个正本：引擎的 `mix_out` + 快开慢关包络。这里不另写包络，只换 `SafetyConfig` 的
//! `open_db` / `floor_db`。新目录再用关掉保命的派生测试集打分（[`suite_safety_off`]），
//! 这样混回只做一次。默认参数下重混的输出要和成绩单原路径逐段一致：
//! 波形层见本文件测试，分数层由 `analyze.py check` 比对。

use vdn_engine::{Engine, Model, ModelError, SafetyConfig};

/// 回放的 hop（与评测器、内置模型同口径，取自前端正本）。
pub const HOP: usize = vdn_types::FRONTEND.hop;
/// 采样率正本。
pub const SR: u32 = vdn_types::MODEL_RATE;

/// 回放模型：吐出「前面补上声明延迟的已对齐输出」。和评测器的 `Replay` 同一口径
/// （那份按输入指纹找文件，截半输入不进它的索引，所以这里自带一个）。
pub struct ReplayBuf {
    raw: Vec<f32>,
    pos: usize,
    latency: usize,
}

impl ReplayBuf {
    pub fn new(aligned: &[f32], latency: usize) -> Self {
        let mut raw = vec![0.0f32; latency];
        raw.extend_from_slice(aligned);
        Self {
            raw,
            pos: 0,
            latency,
        }
    }
}

impl ReplayBuf {
    /// 前面接模型真实的预热输出（长度 = 延迟），而不是补零。
    pub fn with_head(head: &[f32], aligned: &[f32]) -> Self {
        let mut raw = head.to_vec();
        raw.extend_from_slice(aligned);
        Self {
            raw,
            pos: 0,
            latency: head.len(),
        }
    }
}

impl Model for ReplayBuf {
    fn name(&self) -> &'static str {
        "exp021-replay"
    }
    fn sample_rate(&self) -> u32 {
        SR
    }
    fn hop(&self) -> usize {
        HOP
    }
    fn latency_samples(&self) -> usize {
        self.latency
    }
    /// 引擎出故障时会复位模型；时间照样往前走，回放位置不回卷（同评测器）。
    fn reset(&mut self) {}
    fn process(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), ModelError> {
        if input.len() != HOP || output.len() != HOP {
            return Err(ModelError::Shape);
        }
        for (i, o) in output.iter_mut().enumerate() {
            *o = self.raw.get(self.pos + i).copied().unwrap_or(0.0);
        }
        self.pos += HOP;
        Ok(())
    }
}

/// 一组混回参数。`None` 的字段沿用引擎默认值（正本 `vdn_types::desc`）。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MixParams {
    pub open_db: Option<f32>,
    pub floor_db: Option<f32>,
    /// true = 三件保命全关，输出就是模型原始输出（「不混回」那一档）
    pub off: bool,
}

impl MixParams {
    pub fn config(&self) -> SafetyConfig {
        if self.off {
            return SafetyConfig::off();
        }
        let mut c = SafetyConfig::default();
        if let Some(v) = self.open_db {
            c.open_db = v;
        }
        if let Some(v) = self.floor_db {
            c.floor_db = v;
        }
        c
    }
}

/// 驱动引擎走一段（和评测器 `system::run` 同一循环：补零到 hop 整数倍、丢前 latency 个样本）。
/// `x` 是送进系统的输入（原声），`y` 是该输入对应的模型已对齐输出。返回与 `x` 等长的对齐输出。
pub fn remix_one(x: &[f32], y: &[f32], latency: usize, cfg: SafetyConfig) -> Vec<f32> {
    run_engine(ReplayBuf::new(y, latency), x, cfg)
}

/// 任意模型过一遍引擎，返回与 `x` 等长的对齐输出（重混与生成原始输出共用这一个循环）。
pub fn run_engine<M: Model>(model: M, x: &[f32], cfg: SafetyConfig) -> Vec<f32> {
    let mut eng = Engine::new(model, cfg);
    let hop = eng.hop();
    let lat = eng.latency_samples();
    let n_hops = (x.len() + lat).div_ceil(hop);
    let mut padded = vec![0.0f32; n_hops * hop];
    padded[..x.len()].copy_from_slice(x);
    let mut raw = vec![0.0f32; n_hops * hop];
    for (xi, yi) in padded.chunks_exact(hop).zip(raw.chunks_exact_mut(hop)) {
        eng.process(xi, yi);
    }
    raw[lat..lat + x.len()].to_vec()
}

/// 由测试集正本派生「全部任务关掉保命」的测试集：只改每条任务的 `safety`，别的不动。
pub fn suite_safety_off(suite: &serde_json::Value) -> Result<serde_json::Value, String> {
    let mut s = suite.clone();
    let tasks = s
        .get_mut("tasks")
        .and_then(|t| t.as_array_mut())
        .ok_or("测试集没有 tasks 数组")?;
    for t in tasks.iter_mut() {
        let o = t.as_object_mut().ok_or("任务不是对象")?;
        o.insert("safety".into(), serde_json::Value::Bool(false));
    }
    if let Some(o) = s.as_object_mut() {
        let name = o
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("suite")
            .to_string();
        o.insert("name".into(), format!("{name}-safety-off").into());
    }
    Ok(s)
}

/// 外部系统名：`<前缀>-o<open>-f<floor>`，负号写成 m（名字只许 `[a-z0-9-]`）。
pub fn system_name(prefix: &str, p: &MixParams) -> String {
    if p.off {
        return format!("{prefix}-nomix");
    }
    let c = p.config();
    let f = |v: f32| {
        let r = v.round() as i32;
        if r < 0 {
            format!("m{}", -r)
        } else {
            r.to_string()
        }
    };
    format!("{prefix}-o{}-f{}", f(c.open_db), f(c.floor_db))
}

#[cfg(test)]
mod tests;
