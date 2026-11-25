//! asr.rs 的单元测试（分词、对齐、按段路由、缺后端）。
use super::*;

fn t(s: &str) -> Vec<String> {
    tokens(s, "en")
}

#[test]
fn edit_ops_counts() {
    assert_eq!(edit_ops(&t("a b c d"), &t("a c d")), (1, 0, 0));
    assert_eq!(edit_ops(&t("a b c"), &t("a x c")), (0, 1, 0));
    assert_eq!(edit_ops(&t("a b"), &t("a b c")), (0, 0, 1));
    assert_eq!(edit_ops(&t("Hello, World."), &t("hello world")), (0, 0, 0));
}

/// 按段选后端：片段 id 去掉 `#…` 后查路由，查不到用路由的默认，再查不到用总默认。
#[test]
fn route_picks_per_clip_then_defaults() {
    let r: Route =
        serde_json::from_str(r#"{"default": "openrouter", "clips": {"cafe-c00-mba": "local42"}}"#)
            .unwrap();
    assert_eq!(r.pick("cafe-c00-mba#1a2b3c4d", LOCAL42), LOCAL42);
    assert_eq!(r.pick("cafe-c01-mba#00", LOCAL42), OPENROUTER);
    assert_eq!(Route::default().pick("x", LOCAL42), LOCAL42);
}

/// 没装的后端：用到它的段转写报「没装好」，不悄悄换成别的后端；用到的段与型号记进摘要。
#[test]
fn missing_backend_fails_and_summary_counts_used_clips() {
    let dir = std::env::temp_dir().join(format!("vdn-asr-route-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let cache = dir.join("cache.json");
    // 缓存里放好本机后端的两次转写，免得真去连服务
    let key = |rep: usize| format!("abc|{MODEL}|en|{rep}");
    std::fs::write(
        &cache,
        serde_json::json!({key(1): "hi there", key(2): "hi there"}).to_string(),
    )
    .unwrap();
    let route: Route = serde_json::from_str(r#"{"clips": {"b-c00-ext": "openrouter"}}"#).unwrap();
    let mut a = Asr::with_backends(cache, vec![Backend::Local42], LOCAL42, route);
    let wav = dir.join("x.wav");
    let (t, stable) = a.transcribe(&wav, "abc", "en", "a-c00-ext#ff").unwrap();
    assert_eq!((t.as_str(), stable), ("hi there", true));
    let e = a.transcribe(&wav, "abc", "en", "b-c00-ext#ff").unwrap_err();
    assert!(e.contains("没装好"), "{e}");
    let s = a.summary();
    assert_eq!(s["default"], LOCAL42);
    assert_eq!(s["backends"][LOCAL42]["clips"], 1);
    assert_eq!(s["backends"][LOCAL42]["model"], MODEL);
    let _ = std::fs::remove_dir_all(&dir);
}

/// codex 第 6 轮 P2 #4：`VDN_ASR_ROUTE` 指向一个 JSON **文件**（README 第一节同样写法）。
#[test]
fn route_loads_from_file_path() {
    let p = std::env::temp_dir().join(format!("vdn-asr-route-file-{}.json", std::process::id()));
    std::fs::write(
        &p,
        r#"{"default": "openrouter", "clips": {"cafe-c00-mba": "local42"}}"#,
    )
    .unwrap();
    let r = Route::load(&p).unwrap();
    assert_eq!(r.pick("cafe-c00-mba", OPENROUTER), LOCAL42);
    assert!(Route::load(std::path::Path::new("/no/such/route.json")).is_err());
    let _ = std::fs::remove_file(&p);
}

#[test]
fn zh_tokens_are_chars() {
    assert_eq!(tokens("你好，世界", "zh").len(), 4);
}

/// codex 第 9 轮 P2 #2：摘要里的默认后端是路由覆盖后真正生效的那个，环境变量给的另列。
#[test]
fn summary_default_is_effective_route_default() {
    let route: Route = serde_json::from_str(r#"{"default": "local42"}"#).unwrap();
    let dir = std::env::temp_dir().join(format!("vdn-asr-sumdef-{}", std::process::id()));
    let a = Asr::with_backends(
        dir.join("c.json"),
        vec![Backend::Local42],
        OPENROUTER,
        route,
    );
    assert_eq!(a.backend_for("x-c00-ext"), LOCAL42);
    let s = a.summary();
    assert_eq!(s["default"], LOCAL42, "生效的默认");
    assert_eq!(s["env_default"], OPENROUTER, "环境变量给的默认另列");
}
