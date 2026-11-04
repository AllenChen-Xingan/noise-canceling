//! R-FFT 的测试：负例必须红，正例不红，白名单只许缩短。

use super::*;

fn n(rel: &str, src: &str) -> usize {
    fft_problems(rel, src).len()
}

const OUT: &str = "src/speaker/src/a.rs";

#[test]
fn planner_outside_home_is_flagged() {
    for code in [
        "use realfft::RealFftPlanner;\n",
        "fn f() { let p = realfft::RealFftPlanner::<f32>::new(); }\n",
        "fn f(p: &mut P) { let _ = p.plan_fft_forward(512); }\n",
        "fn f(p: &mut P) { let _ = p.plan_fft_inverse(512); }\n",
        "use rustfft::FftPlanner;\n",
        "use rustfft::FftPlannerNeon as N;\n",
        "fn f() { m!(x.plan_fft_forward(8)); }\n",
        "#[test]\nfn t() { let _ = realfft::RealFftPlanner::<f64>::new(); }\n",
    ] {
        assert!(n(OUT, code) >= 1, "{code}");
    }
}

#[test]
fn home_comments_and_strings_pass() {
    assert_eq!(n("src/dsp/src/fft.rs", "use realfft::RealFftPlanner;\n"), 0);
    assert_eq!(
        n(
            "src/dsp/src/fft_tests.rs",
            "fn f(p: P) { p.plan_fft_forward(8); }\n"
        ),
        0
    );
    for code in [
        "// RealFftPlanner 在注释里\nfn f() {}\n",
        "/// 文档里提到 `plan_fft_forward`\nfn f() {}\n",
        "const S: &str = \"RealFftPlanner\";\n",
        "use vdn_dsp::{FrameFft, power_into};\n",
        "fn f() { let my_plan_fft_forward = 1; }\n",
    ] {
        assert_eq!(n(OUT, code), 0, "{code}");
    }
}

#[test]
fn reports_line_and_parse_error() {
    let p = fft_problems(OUT, "fn a() {}\n\nuse realfft::RealFftPlanner;\n");
    assert_eq!(p.len(), 1);
    assert!(p[0].starts_with("src/speaker/src/a.rs:3："), "{}", p[0]);
    assert!(fft_problems(OUT, "fn (")[0].contains("解析失败"));
}

/// 在临时目录里搭一棵小仓：白名单文件还在命中 → 不红；不在白名单的命中 → 红；白名单文件已干净 / 不存在 → 红。
#[test]
fn whitelist_only_shrinks() {
    let d = std::env::temp_dir().join(format!("vdn-fft-site-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    let w = |rel: &str, s: &str| {
        let p = d.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, s).unwrap();
    };
    w("src/dsp/src/fft.rs", "use realfft::RealFftPlanner;\n");
    let dirty = "use realfft::RealFftPlanner;\n";
    for (p, _, _) in ALLOW {
        w(p, dirty);
    }
    let mut probs = Vec::new();
    check(&d, &mut probs);
    assert!(probs.is_empty(), "{probs:?}");

    w("src/speaker/src/fbank.rs", dirty);
    let mut probs = Vec::new();
    check(&d, &mut probs);
    assert_eq!(probs.len(), 1, "{probs:?}");
    assert!(probs[0].contains("fbank.rs"));

    fs::remove_file(d.join("src/speaker/src/fbank.rs")).unwrap();
    w(ALLOW[0].0, "fn clean() {}\n");
    fs::remove_file(d.join(ALLOW[1].0)).unwrap();
    let mut probs = Vec::new();
    check(&d, &mut probs);
    assert_eq!(probs.len(), 2, "{probs:?}");
    assert!(probs.iter().any(|p| p.contains("已没有 FFT 计划")));
    assert!(probs.iter().any(|p| p.contains("不存在")));
    let _ = fs::remove_dir_all(&d);
}

/// 白名单只放行声明过的那几个标识符：场景文件只许逆 FFT，新加正向计划照样红；白名单文件解析失败也红。
#[test]
fn whitelist_is_scoped_per_identifier() {
    let scene = "src/scene/src/features.rs";
    let inv = "fn f(p: &mut realfft::RealFftPlanner<f32>) { p.plan_fft_inverse(8); }\n";
    assert_eq!(n(scene, inv), 0);
    let fwd = "fn f(p: &mut realfft::RealFftPlanner<f32>) { p.plan_fft_inverse(8); p.plan_fft_forward(8); }\n";
    let p = fft_problems(scene, fwd);
    assert_eq!(p.len(), 1, "{p:?}");
    assert!(p[0].contains("plan_fft_forward"));
    assert!(fft_problems(scene, "fn (")[0].contains("解析失败"));
    // 评测器只许正向
    assert_eq!(n("src/eval/src/verifiers/estoi.rs", inv), 1);
}
