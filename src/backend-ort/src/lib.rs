//! vdn-backend-ort：ONNX Runtime 推理后端（规约 §2.1「适配·推理」、§2.3「第一次要接第二个推理后端」）。
//!
//! 从 `vdn-models/src/session.rs` 原样搬来：单线程 CPU 会话的打开（`open`）、全零张量（`zeros`），并 `pub use ort`，
//! 让 `vdn-models` 不再直接依赖 `ort`。会话只在构造期（非实时线程）打开；推理调用（`Session::run*`）由模型适配器在
//! 推理线程上发起，按 B 档管（ORT 内部分配按命名空间登记，规约 §4）。

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::unreachable,
        clippy::todo
    )
)]

pub use ort;

use std::path::Path;
use std::sync::OnceLock;

use ort::environment::Environment;
use ort::session::Session;
use ort::session::builder::{AutoDevicePolicy, GraphOptimizationLevel};

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

static ENV: OnceLock<Environment> = OnceLock::new();

fn env() -> Result<&'static Environment, BoxError> {
    if let Some(e) = ENV.get() {
        return Ok(e);
    }
    let e = ort::init().with_name("vdn").build()?;
    Ok(ENV.get_or_init(|| e))
}

/// 打开一个单线程 CPU 会话。
pub fn open(path: &Path) -> Result<Session, BoxError> {
    if !path.is_file() {
        return Err(format!("找不到模型文件：{}", path.display()).into());
    }
    // 建会话各步的错误类型 `ort::Error<SessionBuilder>` 不是 Send + Sync，统一转成文字
    let b =
        |e: ort::Error<ort::session::builder::SessionBuilder>| -> BoxError { e.to_string().into() };
    // ort rc.15 在 macOS 上默认按 MaxEfficiency 自动挂 CoreML（ort src/session/builder/impl_options.rs:354），
    // 对 GTCRN 直接编译失败；P1 只用 CPU，和 Python 的 CPUExecutionProvider 对齐
    let s = Session::builder(env()?)?
        .with_auto_device(AutoDevicePolicy::PreferCPU)
        .map_err(b)?
        .with_optimization_level(GraphOptimizationLevel::All)
        .map_err(b)?
        .with_intra_threads(1)
        .map_err(b)?
        .with_inter_threads(1)
        .map_err(b)?
        .with_parallel_execution(false)
        .map_err(b)?
        .with_intra_op_spinning(false)
        .map_err(b)?
        .commit_from_file(path)?;
    Ok(s)
}

/// 全零 f32 张量。
pub fn zeros(shape: &[usize]) -> Result<ort::value::Tensor<f32>, BoxError> {
    let n = shape.iter().product();
    Ok(ort::value::Tensor::from_array((
        shape.to_vec(),
        vec![0.0f32; n],
    ))?)
}
