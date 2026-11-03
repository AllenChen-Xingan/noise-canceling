//! 不需要权重的单元测试。要权重的端到端对拍在 main.rs 的 tests 里（#[ignore]）。
use super::tests_support::rnd;
use super::*;

#[test]
fn exp_fast_close_to_libm() {
    let mut worst = 0.0f32;
    let mut x = -80.0f32;
    while x <= 80.0 {
        let rel = ((exp_fast(x) - x.exp()) / x.exp()).abs();
        worst = worst.max(rel);
        x += 0.01;
    }
    assert!(worst < 1e-6, "exp_fast 最大相对误差 {worst}");
}

#[test]
fn tanh_sigmoid_fast_close_to_libm() {
    let (mut wt, mut ws) = (0.0f32, 0.0f32);
    let mut x = -20.0f32;
    while x <= 20.0 {
        wt = wt.max((tanh_fast(x) - x.tanh()).abs());
        ws = ws.max((sig_fast(x) - 1.0 / (1.0 + (-x).exp())).abs());
        x += 0.001;
    }
    assert!(
        wt < 1e-6 && ws < 1e-6,
        "tanh 最大绝对误差 {wt}，sigmoid {ws}"
    );
    assert!(
        exp_fast(-1000.0).is_finite() && exp_fast(1000.0).is_finite(),
        "越界输入要被钳住"
    );
}

/// 原来逐元素判断边界的写法，作参照
fn conv_ref(c: &ConvF, x: &[f32], y: &mut [f32]) {
    let (cig, cog) = (c.cin / c.groups, c.cout / c.groups);
    for o in 0..c.cout {
        let g = o / cog;
        let yo = &mut y[o * c.fout..(o + 1) * c.fout];
        yo.fill(c.b[o]);
        for ci in 0..cig {
            let xi = &x[(g * cig + ci) * c.fin..(g * cig + ci + 1) * c.fin];
            for t in 0..K {
                let wv = c.w[(o * cig + ci) * K + t];
                for (j, a) in yo.iter_mut().enumerate() {
                    let idx = (j * 2 + t) as isize - 2;
                    if idx >= 0 && (idx as usize) < c.fin {
                        *a += wv * xi[idx as usize];
                    }
                }
            }
        }
        for v in yo.iter_mut() {
            *v = prelu(*v, c.alpha);
        }
    }
}

fn deconv_ref(d: &DeconvF, x: &[f32], y: &mut [f32]) {
    let (cig, cog) = (d.cin / d.groups, d.cout / d.groups);
    for o in 0..d.cout {
        y[o * d.fout..(o + 1) * d.fout].fill(d.b[o]);
    }
    for ic in 0..d.cin {
        let g = ic / cig;
        let xi = &x[ic * d.fin..(ic + 1) * d.fin];
        for ol in 0..cog {
            let o = g * cog + ol;
            let yo = &mut y[o * d.fout..(o + 1) * d.fout];
            for k in 0..K {
                let wv = d.w[(ic * cog + ol) * K + k];
                for (i, &xv) in xi.iter().enumerate() {
                    let j = (i * 2 + k) as isize - 2;
                    if j >= 0 && (j as usize) < d.fout {
                        yo[j as usize] += wv * xv;
                    }
                }
            }
        }
    }
    for v in y[..d.cout * d.fout].iter_mut() {
        *v = if d.tanh { v.tanh() } else { prelu(*v, d.alpha) };
    }
}

/// 去分支后的卷积与原写法逐位相同（GTCRN 实际用到的三种尺寸）
#[test]
fn conv_branch_free_bitwise_equal() {
    for (cin, cout, groups, fin) in [(9, 16, 1, F0), (16, 16, 2, F1), (4, 4, 2, 7)] {
        let cig = cin / groups;
        let fout = (fin + 2 * 2 - K) / 2 + 1;
        let mut c = ConvF {
            w: rnd(cout * cig * K, 1),
            b: rnd(cout, 2),
            cin,
            cout,
            groups,
            fin,
            fout,
            alpha: 0.25,
            col: vec![0.0; cig * K * fout],
            ph: vec![0.0; cin * 2 * v2::conv_phase_len(fout)],
        };
        let x = rnd(cin * fin, 3);
        let (mut y1, mut y2) = (vec![0.0; cout * fout], vec![0.0; cout * fout]);
        conv_ref(&c, &x, &mut y1);
        BLAS.store(false, std::sync::atomic::Ordering::Relaxed);
        c.run(&x, &mut y2);
        assert!(
            y1.iter().zip(&y2).all(|(a, b)| a.to_bits() == b.to_bits()),
            "ConvF {cin}->{cout}/{groups} @{fin}"
        );
    }
}

#[test]
fn deconv_branch_free_bitwise_equal() {
    for (cin, cout, groups, fin, tanh) in [
        (16, 16, 2, F2, false),
        (16, 2, 1, F1, true),
        (4, 4, 2, 5, false),
    ] {
        let cog = cout / groups;
        let fout = (fin - 1) * 2 - 2 * 2 + K;
        let d = DeconvF {
            w: rnd(cin * cog * K, 4),
            b: rnd(cout, 5),
            cin,
            cout,
            groups,
            fin,
            fout,
            alpha: 0.1,
            tanh,
            xp: vec![0.0; cin * v2::deconv_pad_len(fin)],
        };
        let x = rnd(cin * fin, 6);
        let (mut y1, mut y2) = (vec![0.0; cout * fout], vec![0.0; cout * fout]);
        deconv_ref(&d, &x, &mut y1);
        d.run(&x, &mut y2);
        assert!(
            y1.iter().zip(&y2).all(|(a, b)| a.to_bits() == b.to_bits()),
            "DeconvF {cin}->{cout}/{groups} @{fin}"
        );
    }
}

// ── 研究 020：计算量统计的交叉校验（工作量公式 ⟷ 模型实际持有的数组 / 参照实现的循环次数）──

fn gru_params(g: &Gru) -> usize {
    g.wi.len() + g.wh.len() + g.bi.len() + g.bh.len()
}

/// 模型实际持有的参数个数：直接数各结构体里的数组长度（与 `Cost` 公式无关）
fn param_count(n: &GtcrnNative) -> usize {
    let sp = |s: &Sparse| s.rows.iter().map(|(_, g)| g.len()).sum::<usize>();
    let pt = |p: &Point| p.w.len() + p.b.len();
    let gt = |b: &GtBlock| {
        pt(&b.p1) + b.dw.len() + b.db.len() + pt(&b.p2) + gru_params(&b.gru) + b.fc.w.len() + b.fc.b.len()
    };
    let dp = |d: &Dp| {
        d.intra.iter().map(gru_params).sum::<usize>()
            + d.inter.iter().map(gru_params).sum::<usize>()
            + d.intra_fc.w.len()
            + d.intra_fc.b.len()
            + d.inter_fc.w.len()
            + d.inter_fc.b.len()
            + d.intra_lnw.len()
            + d.intra_lnb.len()
            + d.inter_lnw.len()
            + d.inter_lnb.len()
    };
    sp(&n.erb)
        + sp(&n.ierb)
        + n.en0.w.len()
        + n.en0.b.len()
        + n.en1.w.len()
        + n.en1.b.len()
        + n.en_gt.iter().chain(&n.de_gt).map(gt).sum::<usize>()
        + n.dp.iter().map(dp).sum::<usize>()
        + n.de3.w.len()
        + n.de3.b.len()
        + n.de4.w.len()
        + n.de4.b.len()
}

struct Collect(Vec<(&'static str, &'static str, Cost)>);
impl Probe for Collect {
    fn mark(&mut self, b: &'static str, o: &'static str, c: impl FnOnce() -> Cost) {
        self.0.push((b, o, c()));
    }
}

#[test]
fn cost_weight_bytes_match_model_params() {
    let mut net = GtcrnNative::from_weights(&Weights::fake(0.01)).unwrap();
    let (mut re, mut im) = (rnd(BINS, 7), rnd(BINS, 8));
    let mut p = Collect(Vec::new());
    net.infer_probed(&mut re, &mut im, &mut p);
    let total = p.0.iter().fold(Cost::default(), |a, (_, _, c)| a + c.clone());
    assert_eq!(total.w_bytes, 4 * param_count(&net) as u64, "Cost 的权重字节 ≠ 模型实际参数");
    // 段名不重复（下界表按 (blk, op) 对行）
    let mut names: Vec<_> = p.0.iter().map(|(b, o, _)| (*b, *o)).collect();
    let n = names.len();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), n, "打点段名重复");
    // 只查量级：真权重下实测 23 838 个（BN 已折、ERB 只存非零段）；假权重全非零，ERB 为稠密
    assert!((20_000..60_000).contains(&param_count(&net)), "参数个数 {}", param_count(&net));
}

#[test]
fn gru_cost_macs_equal_weight_count() {
    let w = Weights::fake(0.1);
    for (i, h) in [(8, 4), (8, 8), (8, 16)] {
        let g = Gru::load(&w, "g", "", i, h).unwrap();
        assert_eq!(g.step_cost().macs as usize, g.wi.len() + g.wh.len());
        assert_eq!(g.step_cost().w_bytes as usize, 4 * gru_params(&g));
    }
}

/// 卷积 / 转置卷积的乘加次数 = 参照实现里真正做乘加的次数（逐元素判断边界的老写法）
#[test]
fn conv_cost_macs_equal_reference_loop_count() {
    for (cin, cout, groups, fin) in [(9, 16, 1, F0), (16, 16, 2, F1), (4, 4, 2, 7)] {
        let cig = cin / groups;
        let fout = (fin + 2 * 2 - K) / 2 + 1;
        let c = ConvF {
            w: vec![0.0; cout * cig * K],
            b: vec![0.0; cout],
            cin,
            cout,
            groups,
            fin,
            fout,
            alpha: 0.0,
            col: Vec::new(),
            ph: Vec::new(),
        };
        let mut n = 0u64;
        for _o in 0..cout {
            for _ci in 0..cig {
                for t in 0..K {
                    for j in 0..fout {
                        let idx = (j * 2 + t) as isize - 2;
                        n += u64::from(idx >= 0 && (idx as usize) < fin);
                    }
                }
            }
        }
        assert_eq!(c.cost().macs, n, "ConvF {cin}->{cout}/{groups} @{fin}");
    }
    for (cin, cout, groups, fin) in [(16, 16, 2, F2), (16, 2, 1, F1), (4, 4, 2, 5)] {
        let cog = cout / groups;
        let fout = (fin - 1) * 2 - 2 * 2 + K;
        let d = DeconvF {
            w: vec![0.0; cin * cog * K],
            b: vec![0.0; cout],
            cin,
            cout,
            groups,
            fin,
            fout,
            alpha: 0.0,
            tanh: false,
            xp: Vec::new(),
        };
        let mut n = 0u64;
        for _ic in 0..cin {
            for _ol in 0..cog {
                for k in 0..K {
                    for i in 0..fin {
                        let j = (i * 2 + k) as isize - 2;
                        n += u64::from(j >= 0 && (j as usize) < fout);
                    }
                }
            }
        }
        assert_eq!(d.cost().macs, n, "DeconvF {cin}->{cout}/{groups} @{fin}");
    }
}

/// 打点不改变输出：infer 与 infer_probed（收集工作量）逐位相同
#[test]
fn probed_infer_bitwise_equal() {
    BLAS.store(false, std::sync::atomic::Ordering::Relaxed);
    let w = Weights::fake(0.02);
    let (mut a, mut b) = (
        GtcrnNative::from_weights(&w).unwrap(),
        GtcrnNative::from_weights(&w).unwrap(),
    );
    for f in 0..5 {
        let (mut r1, mut i1) = (rnd(BINS, 10 + f), rnd(BINS, 20 + f));
        let (mut r2, mut i2) = (r1.clone(), i1.clone());
        a.infer(&mut r1, &mut i1);
        b.infer_probed(&mut r2, &mut i2, &mut Collect(Vec::new()));
        assert!(r1.iter().chain(&i1).zip(r2.iter().chain(&i2)).all(|(x, y)| x.to_bits() == y.to_bits()));
    }
}

/// 预取枚举的权重 = 模型实际持有的参数（交叉校验）；预取不改输出（逐位相同）
#[test]
fn prefetch_covers_all_params_and_keeps_output() {
    let w = Weights::fake(0.02);
    let (mut a, mut b) = (
        GtcrnNative::from_weights(&w).unwrap(),
        GtcrnNative::from_weights(&w).unwrap(),
    );
    let mut n = 0;
    a.for_each_v2_weight(|s| n += s.len());
    assert_eq!(n, param_count(&a), "预取枚举漏了或多了参数");
    a.set_kind(Kind::V2Fast);
    b.set_kind(Kind::V2Fast);
    b.set_prefetch(true);
    for f in 0..3 {
        let (mut r1, mut i1) = (rnd(BINS, 30 + f), rnd(BINS, 40 + f));
        let (mut r2, mut i2) = (r1.clone(), i1.clone());
        a.infer(&mut r1, &mut i1);
        b.infer(&mut r2, &mut i2);
        assert!(r1.iter().chain(&i1).zip(r2.iter().chain(&i2)).all(|(x, y)| x.to_bits() == y.to_bits()));
    }
}

#[test]
fn clog2_values() {
    assert_eq!([1, 2, 3, 4, 5, 8, 9, 33, 528].map(clog2), [0, 1, 2, 2, 3, 3, 4, 6, 10]);
}

#[test]
fn weights_reject_mismatch() {
    let dir = std::env::temp_dir().join(format!("exp010-w-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join("w");
    std::fs::write(p.with_extension("bin"), [0u8; 8]).unwrap(); // 2 个 f32
    std::fs::write(p.with_extension("txt"), "a 3 0 3\n").unwrap(); // 声称 3 个
    assert!(Weights::load(&p).is_err(), "总数对不上要报错");
    std::fs::write(p.with_extension("txt"), "a 2 0 2\n").unwrap();
    let w = Weights::load(&p).unwrap();
    assert!(w.get("a", 3).is_err() && w.get("b", 2).is_err() && w.get("a", 2).is_ok());
    std::fs::write(p.with_extension("bin"), [0u8; 7]).unwrap();
    assert!(Weights::load(&p).is_err(), "bin 长度不是 4 的倍数要报错");
    std::fs::remove_dir_all(&dir).ok();
}
