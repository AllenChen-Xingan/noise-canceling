//! 离线跑场景分析：读一批 16 kHz WAV，逐 hop 喂 `SceneAnalyzer`，每 31 hop（≈0.5 s）判断一次，每次判断写一行 JSON 到标准输出。
//! 标准错误输出算力统计（逐帧 / 每次判断的耗时）。
//!
//! 用法：cargo run --release -p vdn-scene --example scene_offline -- <清单文件> [--ema S] [--confirm S] [--minconf X] [--dwell S]
//! 清单文件每行：`<id> <wav 路径>`。
//! 只用 hound 读文件（dev 依赖，不进库的依赖闭包）；JSON 手写，不引 serde。

use std::fmt::Write as _;
use std::io::{BufWriter, Write};
use std::time::{Duration, Instant};

use vdn_scene::features::NF;
use vdn_scene::{SceneAnalyzer, route, weights};
use vdn_types::{Hysteresis, MODEL_RATE};

const HOP: usize = vdn_types::FRONTEND.hop;
const TICK_HOPS: usize = 31;

fn read_wav(path: &str) -> Vec<f32> {
    let mut r = hound::WavReader::open(path).unwrap_or_else(|e| panic!("读不了 {path}：{e}"));
    let spec = r.spec();
    assert_eq!(spec.sample_rate, MODEL_RATE, "{path} 不是 16 kHz");
    assert_eq!(spec.channels, 1, "{path} 不是单声道");
    match spec.sample_format {
        hound::SampleFormat::Int => {
            let s = (1i64 << (spec.bits_per_sample - 1)) as f32;
            r.samples::<i32>().map(|v| v.unwrap() as f32 / s).collect()
        }
        hound::SampleFormat::Float => r.samples::<f32>().map(|v| v.unwrap()).collect(),
    }
}

fn arr(buf: &mut String, v: &[f32]) {
    buf.push('[');
    for (i, x) in v.iter().enumerate() {
        if i > 0 {
            buf.push(',');
        }
        let _ = write!(buf, "{x:.5}");
    }
    buf.push(']');
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let list = args.get(1).expect("要给清单文件");
    let mut h: Hysteresis = weights::HYSTERESIS;
    let mut ema = weights::EMA_S;
    let mut i = 2;
    while i + 1 < args.len() {
        let v: f32 = args[i + 1].parse().expect("参数要是数字");
        match args[i].as_str() {
            "--ema" => ema = v,
            "--confirm" => h.confirm_s = v,
            "--minconf" => h.min_confidence = v,
            "--dwell" => h.min_dwell_s = v,
            other => panic!("不认识的参数 {other}"),
        }
        i += 2;
    }
    // 范围的正本是 vdn_types::desc：越界直接报错退出，不悄悄夹值
    h.validate().unwrap_or_else(|e| panic!("滞回参数越界：{e}"));
    vdn_types::desc::SCENE_EMA_S
        .check(ema)
        .unwrap_or_else(|e| panic!("--ema 越界：{e}"));
    let out = std::io::stdout();
    let mut out = BufWriter::new(out.lock());
    let (mut t_push, mut n_push, mut t_judge, mut n_judge) =
        (Duration::ZERO, 0u64, Duration::ZERO, 0u64);
    let mut max_push = Duration::ZERO;
    let mut max_judge = Duration::ZERO;
    let mut line = String::new();
    let mut nm = String::new();
    for row in std::fs::read_to_string(list).expect("读不了清单").lines() {
        let mut it = row.split_whitespace();
        let (Some(id), Some(path)) = (it.next(), it.next()) else {
            continue;
        };
        let x = read_wav(path);
        let mut sa = SceneAnalyzer::new(MODEL_RATE, HOP);
        sa.set_tuning(h, ema);
        for (k, hop) in x.as_chunks::<HOP>().0.iter().enumerate() {
            let t0 = Instant::now();
            sa.push_hop(hop);
            let d = t0.elapsed();
            t_push += d;
            max_push = max_push.max(d);
            n_push += 1;
            if (k + 1) % TICK_HOPS != 0 {
                continue;
            }
            let t0 = Instant::now();
            let j = sa.judge();
            let d = t0.elapsed();
            t_judge += d;
            max_judge = max_judge.max(d);
            n_judge += 1;
            line.clear();
            let t = ((k + 1) * HOP) as f32 / MODEL_RATE as f32;
            route::name(j.committed.1, &mut nm);
            let _ = write!(
                line,
                "{{\"id\":\"{id}\",\"t\":{t:.3},\"label_fine\":{},\"label_conf\":{:.4},\"device\":\"{}\",\"committed_fine\":{},\"committed_name\":\"{nm}\",\"committed_user\":\"{}\",\"switched\":{},\"model\":\"{}\",\"tone\":{},",
                j.label.fine.0,
                j.label.confidence.get(),
                j.device.as_str(),
                j.committed.1.0,
                j.committed.0.as_str(),
                j.switched.is_some(),
                j.params.model.as_str(),
                j.params.tone.0,
            );
            line.push_str("\"raw\":");
            arr(&mut line, &j.raw);
            line.push_str(",\"smooth\":");
            arr(&mut line, &j.smooth);
            line.push_str(",\"feat\":");
            arr(&mut line, &j.features[..NF]);
            line.push('}');
            writeln!(out, "{line}").unwrap();
        }
    }
    out.flush().unwrap();
    let us = |d: Duration, n: u64| d.as_secs_f64() * 1e6 / n.max(1) as f64;
    eprintln!(
        "{{\"weights\":\"{}\",\"push_hop_us_mean\":{:.2},\"push_hop_us_max\":{:.1},\"judge_us_mean\":{:.1},\"judge_us_max\":{:.1},\"hops\":{n_push},\"judges\":{n_judge},\"cpu_share_pct\":{:.4}}}",
        weights::TRAINED_ON,
        us(t_push, n_push),
        max_push.as_secs_f64() * 1e6,
        us(t_judge, n_judge),
        max_judge.as_secs_f64() * 1e6,
        // 占一个核的百分比：逐帧耗时 / 16 ms + 判断耗时 / 496 ms
        100.0 * (us(t_push, n_push) / 16000.0 + us(t_judge, n_judge) / 496000.0), // tidy: 非采样率：16000 µs = 16 ms 一帧、496000 µs = 496 ms 一次判断，是微秒时长
    );
}
