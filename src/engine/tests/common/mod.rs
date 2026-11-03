//! 引擎集成测试共用的小工具（各测试文件 `mod common;`）。

#![allow(dead_code)]

use vdn_engine::{Pipeline, StageChain};
use vdn_types::Model;

pub const SR: u32 = 48_000;
pub const HOP_IO: usize = 768;

/// 通带内多正弦 + 一个 15 kHz 分量（16 kHz 模型路径过不去，只有 48 kHz 原声里有）。
pub fn signal(n: usize, hf: f32) -> Vec<f32> {
    let fs = [220.0f32, 523.0, 1187.0, 2213.0, 3301.0, 4519.0, 6007.0];
    (0..n)
        .map(|i| {
            let t = i as f32 / SR as f32;
            let lo: f32 = fs
                .iter()
                .map(|f| 0.05 * (2.0 * std::f32::consts::PI * f * t + f * 0.01).sin())
                .sum();
            lo + hf * (2.0 * std::f32::consts::PI * 15_000.0 * t).sin()
        })
        .collect()
}

pub fn run<M: Model, S: StageChain>(p: &mut Pipeline<M, S>, x: &[f32], block: usize) -> Vec<f32> {
    let mut y = vec![0.0; x.len()];
    for (xi, yi) in x.chunks(block).zip(y.chunks_mut(block)) {
        p.process(xi, yi);
    }
    y
}

pub fn energy(v: &[f32]) -> f64 {
    v.iter().map(|s| f64::from(*s).powi(2)).sum()
}

