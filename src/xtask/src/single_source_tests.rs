//! `single_source.rs` 的测试（R-MIRROR · R-PARAM 的正例与负例），分出来让每个文件 ≤ 600 行。

use super::*;

fn toks() -> Vec<String> {
    param_tokens(&[
        "floor_db".into(),
        "polish.smooth".into(),
        "gate.confirm_n".into(),
    ])
}

#[test]
fn desc_keys_are_parsed_even_when_split_by_rustfmt() {
    let src = "pub const A: ParamDesc = p(1, \"floor_db\", Unit::Db, -60.0, 0.0, -12.0, \"x\");\n\
               pub const B: ParamDesc = p(\n    202,\n    \"polish.floor_db\",\n    Unit::Db,\n);\n\
               const fn p(id: u16, key: &'static str) {}\n";
    assert_eq!(desc_keys(src), ["floor_db", "polish.floor_db"]);
}

#[test]
fn real_desc_file_parses() {
    let src = read_desc(&crate::root()).unwrap();
    let keys = desc_keys(&src);
    assert_eq!(keys.len(), desc_const_count(&src), "{keys:?}");
    assert!(keys.iter().any(|k| k == "floor_db"));
}

/// 条目里夹注释也能解析；解析不出时条数对不上（`check` 据此报错）。
#[test]
fn desc_keys_survive_comments_and_count_mismatch_is_detected() {
    let src = "pub const A: ParamDesc = p(106, /* 编号说明 */ \"gate.confirm_n\", Unit::Count);\n\
               pub const B: ParamDesc = p(\n    2, // 衰减下限\n    \"floor_db\",\n);\n";
    assert_eq!(desc_keys(src), ["gate.confirm_n", "floor_db"]);
    assert_eq!(desc_const_count(src), 2);
    let broken = "pub const A: ParamDesc = make(\"x\");\n";
    assert_ne!(desc_keys(broken).len(), desc_const_count(broken));
}

#[test]
fn tokens_include_short_and_camel_forms() {
    let t = toks();
    for want in [
        "floor_db",
        "FloorDb",
        "smooth",
        "Smooth",
        "confirm_n",
        "ConfirmN",
    ] {
        assert!(t.iter().any(|x| x == want), "{want}");
    }
}

// ── R-PARAM 负例（该报）──
#[test]
fn literal_param_ranges_are_flagged() {
    let bad = [
        "        let ok = (-30.0..=0.0).contains(&p.floor_db);",
        "    if !(-60.0..0.0).contains(&floor_db) {",
        "    self.smooth = s.smooth.clamp(0.0, 1.0);",
        "    let f = floor_db.max(-60.0).min(0.0);",
        "    let n = confirm_n.min(20).max(1);",
        "            ParamField::FloorDb => (-60.0, 0.0),",
        "    let f = floor_db.clamp(-60f32, 0f32);",
        "    let f = floor_db.clamp(-6e1, 0.0);",
        "    let ok = (-6E+1..=0e0).contains(&floor_db);",
        "    let f = /* 夹一下 */ floor_db.clamp(-60.0, 0.0);",
        "    let n = confirm_n.clamp(1u32, 20u32);",
        "    let n = (1usize..=20usize).contains(&confirm_n);",
        "    let f = floor_db.clamp(-60., 0.);",
        // 字符串里写放行标记不算放行
        "    let s = \"tidy: 非参数范围：x\"; let f = floor_db.clamp(-60.0, 0.0);",
        // 尾逗号
        "    let f = floor_db.clamp(-60.0, 0.0,);",
        "            ParamField::FloorDb => (-60.0, 0.0,),",
        // 字节原始字符串里的放行标记不算放行
        "    let s = br#\"tidy: 非参数范围：x\"#; let f = floor_db.clamp(-60.0, 0.0);",
    ];
    for line in bad {
        assert_eq!(range_problems("x.rs", line, &toks()).len(), 1, "{line}");
    }
}

// ── R-PARAM 正例（不该报）──
#[test]
fn non_param_ranges_and_values_pass() {
    let good = [
        // 范围来自描述表
        "    let f = FLOOR_DB.clamp(p.floor_db);",
        // 不是参数的范围
        "    } else if (1500.0..2000.0).contains(&f) {",
        "    let a = alpha.clamp(0.0, 1.0);",
        // 参数的单个值（默认值 / 测试输入），不是范围
        "        floor_db: -12.0,",
        "    assert_eq!((c.floor_db, c.detect_db), (-20.0, 12.0));",
        // 注释里的范围
        "    // floor_db 原来写成 -60.0..=0.0，现在在 desc.rs",
        // 元组下标 / 切片 / 整数方法调用
        "    let s = &floor_db_buf[x.0..n];",
        "    for i in 0..floor_db.len() {",
        "    let n = 3.max(confirm_n);",
        // 放行
        "    let r = (0.0..1.0).contains(&smooth); // tidy: 非参数范围：这是归一化后的插值位置",
        // 字符串里的负例（测试数据）不是代码
        "        \"    let ok = (-30.0..=0.0).contains(&p.floor_db);\",",
        "    let s = r#\"floor_db.clamp(-60.0, 0.0)\"#;",
        "    let s = br#\"\" floor_db.clamp(-60.0, 0.0) \"#;",
        "    let s = br\"floor_db.clamp(-60.0, 0.0)\";",
        // 块注释里的
        "    /* floor_db.clamp(-60.0, 0.0) */ let x = 1;",
        // 生命周期与字符字面量不打乱剥离
        "    fn f<'a>(x: &'a str) -> char { let _ = floor_db; '\"' }",
        "    let q = ['\\'', '\\\\', '\"']; let s = \"floor_db.clamp(-60.0, 0.0)\";",
    ];
    for line in good {
        assert!(range_problems("x.rs", line, &toks()).is_empty(), "{line}");
    }
}

#[test]
fn code_lines_strip_strings_and_comments_across_lines() {
    let src = "let a = \"x\n\\\n  floor_db.clamp(-60.0, 0.0)\";\n/* 开始\nfloor_db.clamp(-60.0, 0.0)\n结束 */ let b = 1;\nlet c = r#\"\na \"q\" floor_db\n\"#;\n";
    let c = code_lines(src);
    assert!(c.iter().all(|l| !l.contains("floor_db")), "{c:#?}");
    assert!(c.iter().any(|l| l.contains("let b = 1;")));
    assert!(c.iter().any(|l| l.contains("let c = \"")));
    assert!(range_problems("x.rs", src, &toks()).is_empty());
}

// ── R-MIRROR ──
#[test]
fn mirror_file_and_same_name_types_are_flagged() {
    let names = types_names(
        ["pub struct FrameCtx {\n}\npub trait Stage: Send {}\npub enum UserScene {}\n"]
            .into_iter(),
    );
    assert_eq!(names, ["FrameCtx", "Stage", "UserScene"]);
    assert_eq!(
        mirror_problems("src/polish/src/mirror.rs", "", &names).len(),
        1
    );
    for bad in [
        "pub struct FrameCtx {\n",
        "pub(crate) trait Stage: Send {}\n",
        "#[derive(Clone)]\nenum UserScene { A }\n",
        "pub type FrameCtx = u8;\n",
    ] {
        assert_eq!(
            mirror_problems("src/x/src/lib.rs", bad, &names).len(),
            1,
            "{bad}"
        );
    }
}

#[test]
fn distinct_names_and_reasoned_same_names_pass() {
    let names = vec!["Hysteresis".to_string(), "Stage".to_string()];
    for good in [
        "pub struct VadHysteresis {}\n",
        "use vdn_types::Stage;\nimpl Stage for X {}\n",
        "/// VAD 概率的滞回\n// tidy: 同名不同物：这是 VAD 概率滞回，vdn-types 的是场景滞回\n#[derive(Clone, Copy)]\npub struct Hysteresis {}\n",
        "pub struct Hysteresis {} // tidy: 同名不同物：VAD 概率滞回\n",
        "// pub struct Stage {}\n",
        "/* pub struct Stage {} */\n",
        "let s = \"pub struct Stage {}\";\n",
        "/* tidy: 同名不同物：块注释里写理由也算 */\npub struct Hysteresis {}\n",
    ] {
        assert!(
            mirror_problems("src/x/src/lib.rs", good, &names).is_empty(),
            "{good}"
        );
    }
    // 只写标记、不写理由不算；写在字符串里也不算
    for bad in [
        "// tidy: 同名不同物：\npub struct Hysteresis {}\n",
        "pub struct Hysteresis {} const S: &str = \"tidy: 同名不同物：假的\";\n",
    ] {
        assert_eq!(
            mirror_problems("src/x/src/lib.rs", bad, &names).len(),
            1,
            "{bad}"
        );
    }
}

/// 整个仓库当前过这两条规则（新加的违规会在这里和 `tidy` 里同时红）。
#[test]
fn repo_passes() {
    let mut p = Vec::new();
    check(&crate::root(), &mut p);
    assert!(p.is_empty(), "{p:#?}");
}
