//! 离线跑修音链：读对齐好的带噪原声 `dry` 与模型输出 `wet`（16 kHz 单声道），过 `PolishChain`，写结果。
//!
//! 频谱节点要吃「模型自己的 STFT」。离线这里对 dry、wet 各做一次 512/256 周期 sqrt-Hann STFT（与 GTCRN 适配器同口径），
//! 在同一帧网格上处理后合成——这是对「挂在模型 STFT 上」的模拟。这次额外的 STFT 让离线输出晚 256 样本，
//! 写文件前平移掉；**实时接入时它不存在**（频谱来自模型适配器），所以不进延迟账。
//!
//! 用法：polish_offline <dry.wav> <wet.wav> <out.wav> <tone 或 时间线文件> [--ramp 秒] [--params s,floor,comfort,demuffle,agc]
//! 时间线文件每行 `<起始秒> <tone>`（scene 的决策），按时间顺序；给一个整数就是全程固定预设。

use std::time::{Duration, Instant};

use vdn_dsp::StreamingStft;
use vdn_dsp::window::sqrt_hann_periodic;
use vdn_polish::{PolishChain, PolishParams};
use vdn_types::FrameCtx;

const N_FFT: usize = 512;
const HOP: usize = 256;
const SR: f32 = 16000.0;

fn read(path: &str) -> Vec<f32> {
    let mut r = hound::WavReader::open(path).unwrap_or_else(|e| panic!("读不了 {path}：{e}"));
    let s = r.spec();
    assert_eq!(
        (s.sample_rate, s.channels),
        (16000, 1),
        "{path} 要 16 kHz 单声道"
    );
    match s.sample_format {
        hound::SampleFormat::Int => {
            let k = (1i64 << (s.bits_per_sample - 1)) as f32;
            r.samples::<i32>().map(|v| v.unwrap() as f32 / k).collect()
        }
        hound::SampleFormat::Float => r.samples::<f32>().map(|v| v.unwrap()).collect(),
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    assert!(
        a.len() >= 5,
        "用法：polish_offline <dry> <wet> <out> <tone|时间线> [--ramp 秒]"
    );
    let dry = read(&a[1]);
    let wet = read(&a[2]);
    let n = dry.len().min(wet.len());
    let timeline: Vec<(f32, u8)> = match a[4].parse::<u8>() {
        Ok(t) => vec![(0.0, t)],
        Err(_) => std::fs::read_to_string(&a[4])
            .expect("读不了时间线")
            .lines()
            .filter_map(|l| {
                let mut it = l.split_whitespace();
                Some((it.next()?.parse().ok()?, it.next()?.parse().ok()?))
            })
            .collect(),
    };
    let ramp = a
        .iter()
        .position(|x| x == "--ramp")
        .map(|i| a[i + 1].parse::<f32>().unwrap())
        .unwrap_or(vdn_types::desc::POLISH_RAMP_S.default);
    let mut chain = PolishChain::new(
        N_FFT,
        HOP,
        SR,
        PolishParams::preset(timeline.first().map(|t| t.1).unwrap_or(0)),
    );
    chain.set_ramp_s(ramp);
    // --params smooth,floor_db,comfort_db,demuffle_max_db,agc：全程固定这组参数（对照实验用，忽略 tone）
    let fixed: Option<PolishParams> = a.iter().position(|x| x == "--params").map(|i| {
        let v: Vec<f32> = a[i + 1].split(',').map(|x| x.parse().unwrap()).collect();
        PolishParams {
            smooth: v[0],
            floor_db: v[1],
            comfort_db: v[2],
            demuffle_max_db: v[3],
            agc: v[4],
            ..PolishParams::DEFAULT
        }
    });
    // 范围的正本是 vdn_types::desc：越界直接报错退出，不悄悄夹值
    if let Some(p) = &fixed {
        p.validate()
            .unwrap_or_else(|e| panic!("--params 越界：{e}"));
    }
    vdn_types::desc::POLISH_RAMP_S
        .check(ramp)
        .unwrap_or_else(|e| panic!("--ramp 越界：{e}"));
    if let Some(p) = fixed {
        chain.set_ramp_s(0.0);
        chain.set_target(p);
    }
    let mut sd = StreamingStft::new(N_FFT, HOP, sqrt_hann_periodic(N_FFT));
    let mut sw = StreamingStft::new(N_FFT, HOP, sqrt_hann_periodic(N_FFT));
    let lat = sw.latency_samples();
    let total = n + lat + HOP;
    let mut pad_d = dry[..n].to_vec();
    let mut pad_w = wet[..n].to_vec();
    pad_d.resize(total.div_ceil(HOP) * HOP, 0.0);
    pad_w.resize(total.div_ceil(HOP) * HOP, 0.0);
    let mut out = vec![0.0f32; pad_w.len()];
    let mut dry_spec = vec![realfft::num_complex::Complex32::new(0.0, 0.0); N_FFT / 2 + 1];
    let mut ti = 0;
    let (mut t_spec, mut t_time, mut hops) = (Duration::ZERO, Duration::ZERO, 0u64);
    for (k, (d, w)) in pad_d
        .as_chunks::<HOP>()
        .0
        .iter()
        .zip(pad_w.as_chunks::<HOP>().0)
        .enumerate()
    {
        let t = (k * HOP) as f32 / SR;
        while fixed.is_none() && ti < timeline.len() && timeline[ti].0 <= t {
            chain.set_target(PolishParams::preset(timeline[ti].1));
            ti += 1;
        }
        dry_spec.copy_from_slice(sd.analyze(d));
        let ws = sw.analyze(w);
        let t0 = Instant::now();
        chain.process_spectrum(ws, &dry_spec, FrameCtx::default());
        t_spec += t0.elapsed();
        let o = &mut out[k * HOP..(k + 1) * HOP];
        sw.synthesize(o);
        let t0 = Instant::now();
        chain.process_time(o);
        t_time += t0.elapsed();
        hops += 1;
    }
    let y = &out[lat..lat + n];
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut wr = hound::WavWriter::create(&a[3], spec).expect("写不了输出");
    for v in y {
        wr.write_sample(*v).unwrap();
    }
    wr.finalize().unwrap();
    let us = |d: Duration| d.as_secs_f64() * 1e6 / hops.max(1) as f64;
    eprintln!(
        "{{\"spectral_us_per_hop\":{:.2},\"agc_us_per_hop\":{:.2},\"hops\":{hops}}}",
        us(t_spec),
        us(t_time)
    );
}
