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

/// 每帧读输出张量失败的原因（`Copy`，不带 `String`，实时线程上可以直接返回）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CopyError {
    /// 不是 f32 张量
    Type,
    /// 元素个数与目标不符
    Len,
}

// 实时（B 档推理线程，每帧调用；规约 §5.3）
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
mod per_frame {
    use super::{CopyError, ort};

    /// 把一个 f32 输出张量的数据拷进 `dst`（缓存回灌、取模型输出用）。不分配；长度不符时不写 `dst`。
    pub fn copy_out(src: &ort::value::DynValue, dst: &mut [f32]) -> Result<(), CopyError> {
        let (_, data) = src
            .try_extract_tensor::<f32>()
            .map_err(|_| CopyError::Type)?;
        if data.len() != dst.len() {
            return Err(CopyError::Len);
        }
        dst.copy_from_slice(data);
        Ok(())
    }
}

pub use per_frame::copy_out;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_missing_file_returns_error_without_panic() {
        let p = Path::new("/nonexistent/vdn-backend-ort-test/no-such-model.onnx");
        let Err(err) = open(p) else {
            panic!("不存在的文件必须返回错误");
        };
        assert!(
            err.to_string().contains("no-such-model.onnx"),
            "错误信息应带上文件路径：{err}"
        );
    }

    #[test]
    fn zeros_has_requested_shape_and_all_zero_data() {
        let t = zeros(&[1, 3, 4]).expect("建张量");
        let (shape, data) = t.extract_tensor();
        assert_eq!(&**shape, &[1i64, 3, 4]);
        assert_eq!(data.len(), 12);
        assert!(data.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn copy_out_copies_when_len_matches_and_leaves_dst_untouched_otherwise() {
        let src = ort::value::Tensor::from_array((vec![2usize, 2], vec![1.0f32, 2.0, 3.0, 4.0]))
            .expect("建张量")
            .into_dyn();
        let mut dst = [0.0f32; 4];
        assert_eq!(copy_out(&src, &mut dst), Ok(()));
        assert_eq!(dst, [1.0, 2.0, 3.0, 4.0]);

        // 长度不符：返回 Len，且不写 dst
        let mut short = [9.0f32; 3];
        assert_eq!(copy_out(&src, &mut short), Err(CopyError::Len));
        assert_eq!(short, [9.0; 3]);
    }

    #[test]
    fn copy_out_rejects_non_f32_tensor() {
        let src = ort::value::Tensor::from_array((vec![2usize], vec![1i64, 2]))
            .expect("建张量")
            .into_dyn();
        let mut dst = [7.0f32; 2];
        assert_eq!(copy_out(&src, &mut dst), Err(CopyError::Type));
        assert_eq!(dst, [7.0; 2]);
    }
}
