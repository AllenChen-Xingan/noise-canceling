//! Core ML 后端（objc2-core-ml）。
//! 做法：所有 MLMultiArray 都包住我们自己的 f32 缓冲（`initWithDataPointer`，不交出所有权）；
//! 输出走 `outputBackings` 写进我们的缓冲；缓存两组 A/B 交替（A 进 B 出），零拷贝。
//! 若 Core ML 忽略 outputBackings，我们读的缓冲不会更新，`BE-PARITY` 会直接失败，所以对拍同时是这项的检查。
use std::ptr::NonNull;

use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::AnyThread;
use objc2_core_ml::{
    MLComputeUnits, MLDictionaryFeatureProvider, MLFeatureProvider, MLFeatureValue, MLModel, MLModelConfiguration,
    MLMultiArray, MLMultiArrayDataType, MLPredictionOptions, MLState,
};
use objc2_foundation::{NSArray, NSDictionary, NSNumber, NSString, NSURL};

use crate::{Backend, ModelKind};

pub fn units(s: &str) -> Result<MLComputeUnits, String> {
    Ok(match s {
        "cpu" => MLComputeUnits::CPUOnly,
        "gpu" => MLComputeUnits::CPUAndGPU,
        "ane" => MLComputeUnits::CPUAndNeuralEngine,
        "all" => MLComputeUnits::All,
        _ => return Err("计算单元只收 cpu|gpu|ane|all".into()),
    })
}

/// mlpackage 编译成 mlmodelc，返回编译产物路径和耗时。已是 mlmodelc 就原样返回。
pub fn compile(path: &str) -> Result<(String, f64), String> {
    if path.trim_end_matches('/').ends_with(".mlmodelc") {
        return Ok((path.into(), 0.0));
    }
    let t = std::time::Instant::now();
    let url = NSURL::fileURLWithPath(&NSString::from_str(path));
    #[allow(deprecated)]
    let out = unsafe { MLModel::compileModelAtURL_error(&url) }.map_err(|e| format!("编译失败：{e:?}"))?;
    let p = out.path().map(|s| s.to_string()).ok_or("编译产物没有路径")?;
    Ok((p, t.elapsed().as_secs_f64() * 1e3))
}

/// 包住一段我们自己持有的 f32 缓冲（连续行主序）。
fn wrap(buf: &mut [f32], shape: &[usize]) -> Result<Retained<MLMultiArray>, String> {
    assert_eq!(buf.len(), shape.iter().product::<usize>());
    let mut strides = vec![1usize; shape.len()];
    for i in (0..shape.len().saturating_sub(1)).rev() {
        strides[i] = strides[i + 1] * shape[i + 1];
    }
    let nums = |v: &[usize]| NSArray::from_retained_slice(&v.iter().map(|&x| NSNumber::new_isize(x as isize)).collect::<Vec<_>>());
    unsafe {
        MLMultiArray::initWithDataPointer_shape_dataType_strides_deallocator_error(
            MLMultiArray::alloc(),
            NonNull::new(buf.as_mut_ptr().cast()).unwrap(),
            &nums(shape),
            MLMultiArrayDataType::Float32,
            &nums(&strides),
            None,
        )
    }
    .map_err(|e| format!("MLMultiArray：{e:?}"))
}

fn dict(pairs: &[(&str, &AnyObject)]) -> Retained<NSDictionary<NSString, AnyObject>> {
    let keys: Vec<Retained<NSString>> = pairs.iter().map(|(k, _)| NSString::from_str(k)).collect();
    let krefs: Vec<&NSString> = keys.iter().map(|k| &**k).collect();
    let vals: Vec<&AnyObject> = pairs.iter().map(|(_, v)| *v).collect();
    NSDictionary::from_slices(&krefs, &vals)
}

pub struct CoreMl {
    model: Retained<MLModel>,
    state: Option<Retained<MLState>>,
    provs: Vec<Retained<MLDictionaryFeatureProvider>>,
    opts: Vec<Retained<MLPredictionOptions>>,
    _arrays: Vec<Retained<MLMultiArray>>,
    xin: Box<[f32]>,
    yout: Box<[f32]>,
    _caches: Vec<Box<[f32]>>,
    kind: ModelKind,
    flip: usize,
    pub compile_ms: f64,
}

impl CoreMl {
    /// `cache_shapes`：各缓存形状（顺序同 `kind.cache_names()`）；`stateful`：缓存在模型里（`MLState`），只进输入出输出。
    pub fn open(path: &str, unit: &str, kind: ModelKind, cache_shapes: &[Vec<usize>], stateful: bool) -> Result<Self, String> {
        let (compiled, compile_ms) = compile(path)?;
        let cfg = unsafe { MLModelConfiguration::new() };
        unsafe { cfg.setComputeUnits(units(unit)?) };
        let url = NSURL::fileURLWithPath(&NSString::from_str(&compiled));
        let model = unsafe { MLModel::modelWithContentsOfURL_configuration_error(&url, &cfg) }.map_err(|e| format!("加载失败：{e:?}"))?;

        let mut xin = vec![0f32; kind.in_len()].into_boxed_slice();
        let mut yout = vec![0f32; kind.out_len()].into_boxed_slice();
        let a_in = wrap(&mut xin, &kind.in_shape())?;
        let a_out = wrap(&mut yout, &kind.out_shape())?;
        let v_in = unsafe { MLFeatureValue::featureValueWithMultiArray(&a_in) };
        let mut arrays = vec![a_in.clone(), a_out.clone()];
        let mut caches: Vec<Box<[f32]>> = Vec::new();
        let (mut provs, mut opts) = (Vec::new(), Vec::new());
        let (cin, cout) = kind.cache_names();
        let state = if stateful {
            let p = unsafe {
                MLDictionaryFeatureProvider::initWithDictionary_error(MLDictionaryFeatureProvider::alloc(), &dict(&[(kind.input_name(), &v_in)]))
            }
            .map_err(|e| format!("{e:?}"))?;
            let o = unsafe { MLPredictionOptions::new() };
            unsafe { o.setOutputBackings(&dict(&[(kind.output_name(), &a_out)])) };
            provs.push(p);
            opts.push(o);
            Some(unsafe { model.newState() })
        } else {
            // sets[k][i]：第 k 组的第 i 个缓存
            let mut sets: [Vec<Retained<MLMultiArray>>; 2] = [Vec::new(), Vec::new()];
            for shape in cache_shapes {
                for set in sets.iter_mut() {
                    let mut b = vec![0f32; shape.iter().product()].into_boxed_slice();
                    let a = wrap(&mut b, shape)?;
                    caches.push(b);
                    arrays.push(a.clone());
                    set.push(a);
                }
            }
            for k in 0..2 {
                let vals: Vec<Retained<MLFeatureValue>> = sets[k].iter().map(|a| unsafe { MLFeatureValue::featureValueWithMultiArray(a) }).collect();
                let mut pin: Vec<(&str, &AnyObject)> = vec![(kind.input_name(), &v_in)];
                for (i, v) in vals.iter().enumerate() {
                    pin.push((&cin[i], v));
                }
                let p = unsafe { MLDictionaryFeatureProvider::initWithDictionary_error(MLDictionaryFeatureProvider::alloc(), &dict(&pin)) }
                    .map_err(|e| format!("{e:?}"))?;
                let mut pout: Vec<(&str, &AnyObject)> = vec![(kind.output_name(), &a_out)];
                for (i, a) in sets[1 - k].iter().enumerate() {
                    pout.push((&cout[i], a));
                }
                let o = unsafe { MLPredictionOptions::new() };
                unsafe { o.setOutputBackings(&dict(&pout)) };
                provs.push(p);
                opts.push(o);
            }
            None
        };
        Ok(Self { model, state, provs, opts, _arrays: arrays, xin, yout, _caches: caches, kind, flip: 0, compile_ms })
    }
}

impl Backend for CoreMl {
    fn run(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), String> {
        self.kind.pack_input(input, &mut self.xin);
        let k = if self.state.is_some() { 0 } else { self.flip };
        let r = autoreleasepool(|_| unsafe {
            let p = ProtocolObject::<dyn MLFeatureProvider>::from_ref(&*self.provs[k]);
            match &self.state {
                Some(s) => self.model.predictionFromFeatures_usingState_options_error(p, s, &self.opts[k]).map(|_| ()),
                None => self.model.predictionFromFeatures_options_error(p, &self.opts[k]).map(|_| ()),
            }
        });
        r.map_err(|e| format!("预测失败：{e:?}"))?;
        self.flip ^= 1;
        self.kind.unpack_output(&self.yout, output);
        Ok(())
    }
}
