//! ONNX Runtime 会话：单线程、顺序执行、不自旋，图优化同 Python 默认（ORT_ENABLE_ALL），利于和 exp003 对拍。

use std::path::Path;
use std::sync::OnceLock;

use ort::environment::Environment;
use ort::session::Session;
use ort::session::builder::GraphOptimizationLevel;

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
    let s = Session::builder(env()?)?
        .with_optimization_level(GraphOptimizationLevel::All)?
        .with_intra_threads(1)?
        .with_inter_threads(1)?
        .with_parallel_execution(false)?
        .with_intra_op_spinning(false)?
        .commit_from_file(path)?;
    Ok(s)
}

/// 全零 f32 张量。
pub fn zeros(shape: &[usize]) -> Result<ort::value::Tensor<f32>, BoxError> {
    let n = shape.iter().product();
    Ok(ort::value::Tensor::from_array((shape.to_vec(), vec![0.0f32; n]))?)
}
