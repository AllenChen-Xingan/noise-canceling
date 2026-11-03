//! R-MIRROR 的测试：旧规则的正负例（片段为合法 Rust）与 012 第五节第 3 表 M1–M3。

use super::*;

fn shapes(src: &str) -> Vec<Shape> {
    defs(&syn::parse_file(src).unwrap(), true)
        .into_iter()
        .map(|(s, _)| s)
        .collect()
}

const TYPES: &str = "pub struct FrameCtx { pub a: u8 }\npub trait Stage: Send {}\npub trait Model { fn a(&self) -> u8; fn b(&mut self, x: f32); fn c(&self); }\npub enum UserScene { A }\n\
    pub struct Frame { pub rate: u32, pub seq: u64, pub conf: crate::Confidence }\npub struct Hysteresis { pub x: f32 }\n";

fn n(src: &str) -> usize {
    mirror_problems("src/x/src/lib.rs", src, &shapes(TYPES)).len()
}

#[test]
fn types_shapes_are_read() {
    let names: Vec<String> = shapes(TYPES).into_iter().map(|s| s.name).collect();
    assert_eq!(
        names,
        [
            "FrameCtx",
            "Stage",
            "Model",
            "UserScene",
            "Frame",
            "Hysteresis"
        ]
    );
    assert!(!types_shapes(&crate::root()).unwrap().is_empty());
}

/// M1 与旧规则的负例：镜像文件、同名定义（任何可见性、带属性、类型别名、内联模块里）
#[test]
fn mirror_file_and_same_name_types_are_flagged() {
    assert_eq!(
        mirror_problems("src/polish/src/mirror.rs", "", &shapes(TYPES)).len(),
        1
    );
    for bad in [
        "pub struct FrameCtx {}\n",
        "pub(crate) trait Stage: Send {}\n",
        "#[derive(Clone)]\nenum UserScene { A }\n",
        "pub type FrameCtx = u8;\n",
        "mod inner {\n    pub struct Frame { rate: u32 }\n}\n",
    ] {
        assert_eq!(n(bad), 1, "{bad}");
    }
}

/// M2：改了名、结构相同（≥ 3 个字段；路径前缀与字段顺序不影响）→ 报；字段少于 3 个不比
#[test]
fn renamed_copies_are_flagged() {
    assert_eq!(
        n("struct FrameCopy { rate: u32, seq: u64, conf: vdn_types::Confidence }\n"),
        1
    );
    assert_eq!(
        n("pub struct F2 { conf: Confidence, seq: u64, rate: u32 }\n"),
        1
    );
    // 结构不同
    assert_eq!(n("struct F3 { rate: u32, seq: u64, conf: f32 }\n"), 0);
    // trait：方法名与签名都一样才算副本（codex #71 第 1 轮 P2 #7）
    assert_eq!(
        n("trait M2 { fn a(&self) -> u8; fn b(&mut self, x: f32); fn c(&self); }\n"),
        1
    );
    assert_eq!(
        n("trait M3 { fn a(&self) -> u16; fn b(&mut self, x: f32); fn c(&self); }\n"),
        0
    );
    // 字段太少（门槛 3）
    assert_eq!(n("struct H2 { x: f32 }\n"), 0);
}

/// M3 与旧规则的正例：不同名、引用、写了理由的同名（同行、上方注释、块注释）、注释与字符串里的
#[test]
fn distinct_names_and_reasoned_same_names_pass() {
    for good in [
        "pub struct VadHysteresis {}\n",
        "use vdn_types::Stage;\nimpl Stage for X {}\n",
        "/// VAD 概率的滞回\n// tidy: 同名不同物：这是 VAD 概率滞回，vdn-types 的是场景滞回\n#[derive(Clone, Copy)]\npub struct Hysteresis {}\n",
        "pub struct Hysteresis {} // tidy: 同名不同物：VAD 概率滞回\n",
        "// pub struct Stage {}\n",
        "/* pub struct Stage {} */\n",
        "const S: &str = \"pub struct Stage {}\";\n",
        "/* tidy: 同名不同物：块注释里写理由也算 */\npub struct Hysteresis {}\n",
        "// tidy: 同名不同物：改名副本也能放行\nstruct FrameCopy { rate: u32, seq: u64, conf: Confidence }\n",
    ] {
        assert_eq!(n(good), 0, "{good}");
    }
    // 只写标记、不写理由不算；写在字符串里也不算
    for bad in [
        "// tidy: 同名不同物：\npub struct Hysteresis {}\n",
        "pub struct Hysteresis {} const S: &str = \"tidy: 同名不同物：假的\";\n",
    ] {
        assert_eq!(n(bad), 1, "{bad}");
    }
}

/// 解析不了的文件报红
#[test]
fn parse_errors_are_reported() {
    assert_eq!(n("struct broken {\n"), 1);
}

/// 整个仓库当前过这条规则。
#[test]
fn repo_passes() {
    let mut p = Vec::new();
    check(&crate::root(), &mut p);
    assert!(p.is_empty(), "{p:#?}");
}

/// codex #71 第 4 轮 P2 #2：`r#` 名字照样认（同名、改名副本）
#[test]
fn raw_identifiers_are_unrawed() {
    assert_eq!(n("struct r#FrameCtx;\n"), 1);
    assert_eq!(
        n("struct F4 { r#rate: u32, seq: u64, conf: r#Confidence }\n"),
        1
    );
}
