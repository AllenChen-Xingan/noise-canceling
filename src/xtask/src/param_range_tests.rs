//! R-PARAM 的测试：旧文本规则的正负例（片段包成合法的 Rust）与 012 第五节第 3 表 P1–P3。

use super::*;

fn toks() -> Vec<String> {
    param_tokens(&[
        "floor_db".into(),
        "polish.smooth".into(),
        "gate.confirm_n".into(),
    ])
}

/// 包进一个函数，返回问题数（生产文件）。
fn n(body: &str) -> usize {
    range_problems(
        "x.rs",
        &format!("fn f() {{\n{body}\n}}\n"),
        &toks(),
        &defs(),
    )
    .len()
}

/// floor_db 的描述表默认值 −12，smooth 0.5
fn defs() -> Vec<(String, f64)> {
    vec![("floor_db".into(), -12.0), ("smooth".into(), 0.5)]
}

fn desc(src: &str) -> (Vec<String>, usize) {
    desc_keys(&[std::rc::Rc::new(syn::parse_file(src).unwrap())])
}

#[test]
fn desc_keys_are_parsed_even_when_split_by_rustfmt() {
    let src = "pub const A: ParamDesc = p(1, \"floor_db\", Unit::Db, -60.0, 0.0, -12.0, \"x\");\n\
               pub const B: ParamDesc = p(\n    202,\n    \"polish.floor_db\",\n    Unit::Db,\n);\n\
               const fn p(id: u16, key: &'static str) {}\n";
    assert_eq!(
        desc(src),
        (vec!["floor_db".into(), "polish.floor_db".into()], 2)
    );
}

#[test]
fn real_desc_file_parses() {
    let files = read_desc(&crate::root()).unwrap();
    let (keys, consts) = desc_keys(&files);
    assert_eq!(keys.len(), consts, "{keys:?}");
    assert!(keys.iter().any(|k| k == "floor_db"));
}

/// 条目里夹注释也能解析；解析不出时条数对不上（`check` 据此报错）。
#[test]
fn desc_keys_survive_comments_and_count_mismatch_is_detected() {
    let src = "pub const A: ParamDesc = p(106, /* 编号说明 */ \"gate.confirm_n\", Unit::Count);\n\
               pub const B: ParamDesc = p(\n    2, // 衰减下限\n    \"floor_db\",\n);\n";
    assert_eq!(
        desc(src),
        (vec!["gate.confirm_n".into(), "floor_db".into()], 2)
    );
    let (k, c) = desc("pub const A: ParamDesc = make(\"x\");\n");
    assert_ne!(k.len(), c);
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

/// 负例（该报）：旧规则认的写法照样认
#[test]
fn literal_param_ranges_are_flagged() {
    for body in [
        "let ok = (-30.0..=0.0).contains(&p.floor_db);",
        "if !(-60.0..0.0).contains(&floor_db) {}",
        "self.smooth = s.smooth.clamp(0.0, 1.0);",
        "let f = floor_db.max(-60.0).min(0.0);",
        "let n = confirm_n.min(20).max(1);",
        "let f = floor_db.clamp(-60f32, 0f32);",
        "let f = floor_db.clamp(-6e1, 0.0);",
        "let ok = (-6E+1..=0e0).contains(&floor_db);",
        "let f = /* 夹一下 */ floor_db.clamp(-60.0, 0.0);",
        "let n = confirm_n.clamp(1u32, 20u32);",
        "let n = (1usize..=20usize).contains(&confirm_n);",
        "let f = floor_db.clamp(-60., 0.);",
        // 字符串里写放行标记不算放行
        "let s = \"tidy: 非参数范围：x\"; let f = floor_db.clamp(-60.0, 0.0);",
        "let f = floor_db.clamp(-60.0, 0.0,);",
        // 宏参数里的（旧规则按行能查到，新规则解析宏参数后照查）
        "assert!((-60.0..=0.0).contains(&floor_db));",
        // else if 的条件单独算（codex #71 第 2 轮 P2 #4）
        "if ready {} else if (-60.0..=0.0).contains(&floor_db) {}",
        // r# 前缀（codex #71 第 3 轮 P2 #5）
        "let f = r#floor_db.clamp(-60.0, 0.0);",
        // 方法名的 r#（codex #71 第 6 轮 P2 #3）
        "let f = floor_db.r#clamp(-60.0, 0.0);",
        "let f = floor_db.r#max(-60.0).r#min(0.0);",
        "let s = br#\"tidy: 非参数范围：x\"#; let f = floor_db.clamp(-60.0, 0.0);",
    ] {
        assert_eq!(n(body), 1, "{body}");
    }
    let arm = "fn g(x: ParamField) -> (f64, f64) {\n    match x {\n        ParamField::FloorDb => (-60.0, 0.0),\n        _ => (0.0, 1.0),\n    }\n}\n";
    assert_eq!(range_problems("x.rs", arm, &toks(), &defs()).len(), 1);
}

/// P1：rustfmt 拆成几行的范围也查得到（旧规则查不到）
#[test]
fn p1_ranges_split_across_lines() {
    assert_eq!(n("let f = x.floor_db.clamp(\n    -60.0,\n    0.0,\n);"), 1);
    assert_eq!(
        n("let ok = (-60.0..=0.0)\n    .contains(\n        &floor_db,\n    );"),
        1
    );
}

/// P2：范围先赋给不带参数名的变量、在别的语句里再用——仍查不到（记录限制）
#[test]
#[ignore = "已知限制：只在同一条语句里找参数名（012 第五节 P2），这条预期查不到"]
fn p2_range_in_other_statement() {
    assert_eq!(n("let r = -60.0..0.0;\nlet ok = r.contains(&floor_db);"), 1);
}

/// P3：非测试代码里结构体字面量的参数字段写的数**等于描述表默认值** → 报；
/// 和默认值不同的预设不报；测试代码里造输入不报
#[test]
fn p3_default_literals_in_production() {
    assert_eq!(n("let p = Polish { smooth: 0.5, floor_db: -20.0 };"), 1);
    assert_eq!(n("let p = Polish { r#smooth: 0.5 };"), 1, "r# 字段名也认");
    assert_eq!(
        n("let p = Polish { floor_db: -20.0, smooth: 0.3 };"),
        0,
        "预设值不报"
    );
    let files = read_desc(&crate::root()).unwrap();
    assert_eq!(
        desc_defaults(&files).len(),
        desc_keys(&files).0.len(),
        "每条描述都读出默认值"
    );
    assert_eq!(
        n("let p = Polish { floor_db: -12.0, ..Default::default() };"),
        1
    );
    // 放行：本意是预设、碰巧等于默认；必须写理由
    assert_eq!(
        n("let p = Polish { floor_db: -12.0 }; // tidy: 预设值，不随默认变：exp006 标定"),
        0
    );
    assert_eq!(
        n("let p = Polish { floor_db: -12.0 }; // tidy: 预设值，不随默认变："),
        1,
        "没写理由就红"
    );
    assert_eq!(
        n("let p = Polish { floor_db: -12.0 }; // tidy: 非参数范围：x"),
        1,
        "范围的放行标记不放行默认值"
    );
    let test = "#[cfg(test)]\nmod t {\n    fn g() { let p = Polish { floor_db: -12.0 }; }\n}\n";
    assert!(range_problems("x.rs", test, &toks(), &defs()).is_empty());
    let src = "fn g() { let p = Polish { floor_db: -12.0 }; }\n";
    assert!(
        range_problems("x.rs", src, &toks(), &[]).is_empty(),
        "整份测试文件不查默认值"
    );
}

/// 正例（不该报）
#[test]
fn non_param_ranges_and_values_pass() {
    for body in [
        // 范围来自描述表
        "let f = FLOOR_DB.clamp(p.floor_db);",
        // 不是参数的范围
        "if (1500.0..2000.0).contains(&f) {}",
        "let a = alpha.clamp(0.0, 1.0);",
        // 参数的单个值（测试输入），不是范围
        "assert_eq!((c.floor_db, c.detect_db), (-20.0, 12.0));",
        // 注释里的范围
        "// floor_db 原来写成 -60.0..=0.0，现在在 desc.rs",
        // 元组下标 / 切片 / 整数方法调用
        "let s = &floor_db_buf[x.0..n];",
        "for i in 0..floor_db.len() {}",
        "let n = 3.max(confirm_n);",
        // 放行
        "let r = (0.0..1.0).contains(&smooth); // tidy: 非参数范围：这是归一化后的插值位置",
        // 字符串里的负例（测试数据）不是代码
        "let s = \"    let ok = (-30.0..=0.0).contains(&p.floor_db);\";",
        "let s = r#\"floor_db.clamp(-60.0, 0.0)\"#;",
        "let s = br\"floor_db.clamp(-60.0, 0.0)\";",
        "/* floor_db.clamp(-60.0, 0.0) */ let x = 1;",
        // for 循环头的范围不把循环体算进同一条语句
        "for i in 0..10 {\n    let a = floor_db;\n}",
        // let 初始化、常量、分支值里带块的也只算头部（codex #71 第 4 轮 P2 #3）
        "let _ = for i in 0..10 { let _ = floor_db; };",
        "let _ = (for i in 0..10 { let _ = floor_db; });",
        "let _ = Some(for i in 0..10 { let _ = floor_db; });",
        "let _ = (1, for i in 0..10 { let _ = floor_db; });",
        "let _ = match x { _ => for i in 0..10 { let _ = floor_db; } };",
    ] {
        assert_eq!(n(body), 0, "{body}");
    }
}

/// 解析不了的文件报红
#[test]
fn parse_errors_are_reported() {
    assert_eq!(
        range_problems("x.rs", "fn broken( {\n", &toks(), &defs()).len(),
        1
    );
}

/// 整个仓库当前过这条规则。
#[test]
fn repo_passes() {
    let mut p = Vec::new();
    check(&crate::root(), &mut p);
    assert!(p.is_empty(), "{p:#?}");
}
