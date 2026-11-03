//! R-RATE 的测试：012 第五节第 2 表 R1–R8 的负例（必须红）与正例。

use super::*;

/// 包进一个函数里，返回问题数。
fn n(body: &str) -> usize {
    rate_problems("src/eval/src/a.rs", &format!("fn f() {{\n{body}\n}}\n")).len()
}

/// R1–R3：各种写法的字面量（区间、全部合法后缀、`.0`）
#[test]
fn r1_r3_literal_forms() {
    for code in [
        "let r = 48_000;",
        "let r = 16000u32;",
        "g(48000.0);",
        "let x = y * 16_000.0_f32;",
        "const SR: usize = 16_000usize;",
        "let t = (a, 48_000);",
        "for i in 0..16000 {}",
        "let s = &x[..48_000 - l];",
        "let r = 0..=48_000;",
        "let r = 48_000..96_000;",
        "let a = 48000u16;",
        "let b = 16000i16;",
        "let c = 48_000u128;",
        "let d = 16_000_i128;",
    ] {
        assert_eq!(n(code), 1, "{code}");
    }
}

/// R4：十六进制、八进制、二进制写的 48000 / 16000（文本规则查不到）
#[test]
fn r4_other_radixes() {
    for code in [
        "let a = 0xBB80;",
        "let b = 0o135600;",
        "let c = 0b11111010000000;",
        "let d = 0x3E80;",
    ] {
        assert_eq!(n(code), 1, "{code}");
    }
}

/// R5：两边都是字面量的算式算出采样率
#[test]
fn r5_folded_arithmetic() {
    for code in [
        "let a = 3 * 16_000;",
        "let b = 48 * 1000;",
        "let c = 144_000 / 3;",
        "let d = (8000 + 8000);",
        "let e = 2.0 * 8000.0;",
    ] {
        assert!(n(code) >= 1, "{code}");
    }
    // 整数除法截断（codex #71 第 1 轮 P2 #6）
    assert!(n("let f = 96001u32 / 2;") >= 1);
    assert_eq!(n("let g = 96001.0 / 2.0;"), 0, "浮点 48000.5 不是采样率");
    // 含变量、或算出来不是采样率的不算
    for code in ["let a = 3 * n;", "let b = 48 * 100;", "let c = x / 3;"] {
        assert_eq!(n(code), 0, "{code}");
    }
}

/// R6：宏参数里的字面量也查
#[test]
fn r6_inside_macros() {
    for code in [
        "let v = vec![0.0; 48_000];",
        "assert_eq!(r, 16000);",
        "println!(\"{}\", 48000);",
    ] {
        assert_eq!(n(code), 1, "{code}");
    }
}

/// R7：测试模块、`tests/`、`examples/`、底层 crate 里的字面量照查（不识别测试区）
#[test]
fn r7_test_code_is_checked_too() {
    let src =
        "fn f() -> u32 { 48_000 }\n#[cfg(test)]\nmod tests {\n    fn g() { let _ = 48_000; }\n}\n";
    assert_eq!(rate_problems("src/shm/src/wait.rs", src).len(), 2);
    for rel in [
        "src/shm/tests/ring.rs",
        "src/dsp/examples/x.rs",
        "src/eval/tests/a.rs",
        "src/shm/src/tests.rs",
    ] {
        assert_eq!(
            rate_problems(rel, "const R: u32 = 48_000;\n").len(),
            1,
            "{rel}"
        );
    }
}

/// R8：放行要写理由；写了理由才放行
#[test]
fn r8_allow_marker_needs_reason() {
    let ok = "const N: u32 = 48_000; // tidy: 非采样率：一分钟的毫秒数之类\n";
    assert!(rate_problems("src/eval/src/a.rs", ok).is_empty());
    for bad in [
        "const N: u32 = 48_000; // 非采样率\n",
        "const N: u32 = 48_000; // tidy: 非采样率：\n",
        "const M: u32 = 48_000; // tidy: 非采样率：   \n",
    ] {
        assert_eq!(rate_problems("src/eval/src/a.rs", bad).len(), 1, "{bad}");
    }
}

/// 正例：注释、字符串、文档注释里的不算；数值不同的不算；正本文件本身不查
#[test]
fn other_numbers_and_places_pass() {
    for code in [
        "// 48_000 Hz",
        "let s = \"16000\";",
        "let n = 160000;",
        "let n = 4800;",
        "let x = 48000.5;",
        "let x = 1.48000;",
        "let ns = 16_000_000;",
        "let r = IO_RATE;",
    ] {
        assert_eq!(n(code), 0, "{code}");
    }
    let doc = "//! 16000\n/// 采样率 48000 Hz\nconst A: u8 = 1;\n";
    assert!(rate_problems("src/eval/src/a.rs", doc).is_empty());
    assert!(rate_problems(RATE_FILE, "pub const IO_RATE: u32 = 48_000;\n").is_empty());
}

/// 解析不了的文件报红，不静默跳过
#[test]
fn parse_errors_are_reported() {
    let p = rate_problems("src/eval/src/a.rs", "fn broken( {\n");
    assert_eq!(p.len(), 1);
    assert!(p[0].contains("语法树解析失败"), "{p:?}");
}
