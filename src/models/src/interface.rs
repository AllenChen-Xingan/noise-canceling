//! 模型接口的声明（输入输出张量的名字与形状），以及从 ONNX 文件实读接口、与声明比对。
//!
//! 三方一致（CLAUDE.md 第五节第 5 条）：ONNX 文件实读 ↔ 这里的声明 ↔ `specs/schema/models.interface.json`。
//! - 声明 ↔ schema：`vdn-schema` 导出并逐字比对（CI）
//! - 文件 ↔ 声明：适配器 `open` 时调 [`check_file`]，不一致不让打开；`tests/interface.rs` 有一条要仓外权重的测试
//!
//! 形状照文件原样写：固定维写数，动态维写 `-1`（ONNX 的写法）。全部张量都是 [`DTYPE`]（f32），文件里的元素类型也核对。

use ort::session::Session;
use ort::value::TensorElementType;
use vdn_backend_ort::{BoxError, ort};

/// 全部张量的元素类型（适配器按 f32 预分配、读写）。导出文件的 `dtype` 也取它。
pub const DTYPE: &str = "f32";

/// 一个张量：名字 + 形状（`-1` = 动态维）。元素类型一律 [`DTYPE`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TensorDecl {
    pub name: &'static str,
    pub shape: &'static [i64],
}

/// 一个模型的输入输出，按会话里的顺序。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Interface {
    pub inputs: &'static [TensorDecl],
    pub outputs: &'static [TensorDecl],
}

const fn t(name: &'static str, shape: &'static [i64]) -> TensorDecl {
    TensorDecl { name, shape }
}

/// GTCRN simple：`mix` 是一帧复数谱（257 个频点 × 实部虚部），三个缓存每帧回灌。
pub const GTCRN: Interface = Interface {
    inputs: &[
        t("mix", &[1, 257, 1, 2]),
        t("conv_cache", &[2, 1, 16, 16, 33]),
        t("tra_cache", &[2, 3, 1, 1, 16]),
        t("inter_cache", &[2, 1, 33, 16]),
    ],
    outputs: &[
        t("enh", &[1, 257, 1, 2]),
        t("conv_cache_out", &[2, 1, 16, 16, 33]),
        t("tra_cache_out", &[2, 3, 1, 1, 16]),
        t("inter_cache_out", &[2, 1, 33, 16]),
    ],
};

/// FastEnhancer-M wav2wav：`wav_in` 一个 hop 的波形，六个缓存每帧回灌（`cache_in_i` ↔ `cache_out_i`）。
pub const FASTENHANCER_M: Interface = Interface {
    inputs: &[
        t("wav_in", &[1, 160]),
        t("cache_in_0", &[1, 352]),
        t("cache_in_1", &[1, 352]),
        t("cache_in_2", &[1, 48, 72]),
        t("cache_in_3", &[1, 48, 72]),
        t("cache_in_4", &[1, 48, 72]),
        t("cache_in_5", &[1, 48, 72]),
    ],
    outputs: &[
        t("wav_out", &[1, 160]),
        t("cache_out_0", &[1, 352]),
        t("cache_out_1", &[1, 352]),
        t("cache_out_2", &[1, 48, 72]),
        t("cache_out_3", &[1, 48, 72]),
        t("cache_out_4", &[1, 48, 72]),
        t("cache_out_5", &[1, 48, 72]),
    ],
};

/// 从文件实读的一个张量。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TensorRead {
    pub name: String,
    /// `None` = 不是张量（序列、映射等）
    pub shape: Option<Vec<i64>>,
    /// 元素类型：f32 写 [`DTYPE`]，别的写 ort 的名字；`None` = 不是张量
    pub dtype: Option<String>,
}

fn dtype_name(t: TensorElementType) -> String {
    if t == TensorElementType::Float32 {
        DTYPE.to_string()
    } else {
        format!("{t:?}")
    }
}

/// 读已打开会话的输入输出（构造期，非实时）。
pub fn read_session(s: &Session) -> (Vec<TensorRead>, Vec<TensorRead>) {
    let one = |name: &str, t: &ort::value::ValueType| TensorRead {
        name: name.to_string(),
        shape: t.tensor_shape().map(|d| d.iter().copied().collect()),
        dtype: t.tensor_type().map(dtype_name),
    };
    let ins = s
        .inputs()
        .iter()
        .map(|i| one(i.name(), i.dtype()))
        .collect();
    let outs = s
        .outputs()
        .iter()
        .map(|o| one(o.name(), o.dtype()))
        .collect();
    (ins, outs)
}

type Row<'a> = (&'a str, Option<&'a [i64]>, Option<&'a str>);

fn diff(side: &str, decl: &[TensorDecl], got: &[TensorRead]) -> Option<String> {
    let want: Vec<Row> = decl
        .iter()
        .map(|d| (d.name, Some(d.shape), Some(DTYPE)))
        .collect();
    let have: Vec<Row> = got
        .iter()
        .map(|g| (g.name.as_str(), g.shape.as_deref(), g.dtype.as_deref()))
        .collect();
    (want != have).then(|| format!("{side}不符：声明 {want:?}，文件 {have:?}"))
}

/// 会话的输入输出与声明逐个比对（名字、顺序、形状）。不一致返回说明。
pub fn check_session(decl: &Interface, s: &Session) -> Result<(), BoxError> {
    let (ins, outs) = read_session(s);
    check_read(decl, &ins, &outs)
}

/// 实读结果与声明逐个比对（名字、顺序、形状、元素类型）。不一致返回说明。
pub fn check_read(
    decl: &Interface,
    ins: &[TensorRead],
    outs: &[TensorRead],
) -> Result<(), BoxError> {
    let mut why = Vec::new();
    why.extend(diff("输入", decl.inputs, ins));
    why.extend(diff("输出", decl.outputs, outs));
    if why.is_empty() {
        Ok(())
    } else {
        Err(why.join("；").into())
    }
}

/// 形状里的固定维转成 usize（动态维按 1，用于预分配）。
pub fn dims(shape: &[i64]) -> Vec<usize> {
    shape.iter().map(|&d| d.max(1) as usize).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ModelKind;

    /// 每个模型：输入输出一样多、按位置配对的形状相同（状态进出同形）、名字不重复、没有空形状。
    #[test]
    fn declarations_are_well_formed() {
        for k in ModelKind::ALL {
            let io = k.interface();
            assert_eq!(io.inputs.len(), io.outputs.len(), "{k:?}");
            for (a, b) in io.inputs.iter().zip(io.outputs) {
                assert_eq!(a.shape, b.shape, "{k:?} {} / {}", a.name, b.name);
                assert!(!a.shape.is_empty() && a.shape.iter().all(|&d| d >= 1 || d == -1));
            }
            let mut names: Vec<&str> = io.inputs.iter().chain(io.outputs).map(|t| t.name).collect();
            let n = names.len();
            names.sort_unstable();
            names.dedup();
            assert_eq!(names.len(), n, "{k:?} 名字重复");
        }
    }

    fn read(ts: &[TensorDecl]) -> Vec<TensorRead> {
        ts.iter()
            .map(|t| TensorRead {
                name: t.name.into(),
                shape: Some(t.shape.to_vec()),
                dtype: Some(DTYPE.into()),
            })
            .collect()
    }

    /// 与声明完全相同才过；名字、顺序、形状、元素类型任一不同都拒（codex 第 1 轮 #1：同名同形但不是 f32）。
    #[test]
    fn check_read_rejects_any_difference() {
        for k in ModelKind::ALL {
            let io = k.interface();
            let (ins, outs) = (read(io.inputs), read(io.outputs));
            check_read(&io, &ins, &outs).unwrap();
            let mut bad = ins.clone();
            bad[0].dtype = Some(dtype_name(TensorElementType::Float16));
            assert!(check_read(&io, &bad, &outs).is_err(), "{k:?} 元素类型");
            let mut bad = outs.clone();
            bad[1].dtype = None;
            assert!(check_read(&io, &ins, &bad).is_err(), "{k:?} 不是张量");
            let mut bad = ins.clone();
            bad.swap(1, 2);
            assert!(check_read(&io, &bad, &outs).is_err(), "{k:?} 顺序");
            let mut bad = ins.clone();
            bad[0].shape = Some(vec![1, 1]);
            assert!(check_read(&io, &bad, &outs).is_err(), "{k:?} 形状");
            let mut bad = outs.clone();
            bad[0].name = "x".into();
            assert!(check_read(&io, &ins, &bad).is_err(), "{k:?} 名字");
        }
    }

    #[test]
    fn dims_maps_dynamic_to_one() {
        assert_eq!(dims(&[1, -1, 3]), [1, 1, 3]);
    }
}
