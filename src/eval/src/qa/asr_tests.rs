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

/// 2026-11-25：OpenRouter 后端在装起来时就核账本；账本不存在，识别后端整个装不起来（原因写进 meta.asr），
/// 不等到每段都去撞一次。
#[test]
fn from_env_refuses_when_ledger_missing() {
    // 外面设了改后端、路由、次数的环境变量时，组装走的分支会变，这条测不准：跳过并说明（codex 本分支第 2 轮 P2 #1）
    let set: Vec<&str> = ["VDN_ASR_BACKEND", "VDN_ASR_ROUTE", "VDN_ASR_REPEATS"]
        .into_iter()
        .filter(|k| std::env::var_os(k).is_some())
        .collect();
    if !set.is_empty() {
        eprintln!("跳过 from_env_refuses_when_ledger_missing：环境里设了 {set:?}");
        return;
    }
    let root = std::env::temp_dir().join(format!("vdn-asr-fromenv-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(".env.local"),
        "OPENROUTER_API_KEY=placeholder-test-credential-0003\n",
    )
    .unwrap();
    // 显式给一个不存在的账本路径，不依赖外面有没有设 VDN_ASR_LEDGER（codex 本分支第 1 轮 P2 #3）
    let missing = root.join("nope/ledger.json");
    let e = Asr::from_env_with(root.join("cache.json"), &root, Some(missing))
        .err()
        .expect("账本不存在应装不起来");
    assert!(e.contains("不存在"), "{e}");
    // 显式指定一本存在的账本就装得起来（不发请求）
    let l = root.join("elsewhere/ledger.json");
    crate::qa::asr_ledger::init(&l, crate::qa::asr_ledger::Created::test()).unwrap();
    assert!(Asr::from_env_with(root.join("cache.json"), &root, Some(l.clone())).is_ok());
    // 封存了：装后端时就拒
    std::fs::write(crate::qa::asr_ledger::sealed_path(&l), "已封存").unwrap();
    let e = Asr::from_env_with(root.join("cache.json"), &root, Some(l))
        .err()
        .expect("封存了应装不起来");
    assert!(e.contains("封存"), "{e}");
    let _ = std::fs::remove_dir_all(&root);
}

/// 大总管 2026-11-25 冒烟：两次转写只差大小写、标点、空白时，评分用的词一样，不该判「不一致」。
/// 原先逐字比原文，5 段里 4/15 条被判不一致，吃字整段成了「不知道」。
#[test]
fn stability_ignores_differences_scoring_cannot_see() {
    assert!(same_text("Hello, world.", "hello world", "en"));
    assert!(same_text("你好，世界。", "你好 世界", "zh"));
    assert!(
        !same_text("hello world", "hello word", "en"),
        "真的有词不同仍判不一致"
    );
    assert!(!same_text("你好世界", "你好视界", "zh"));
    // 走转写入口：缓存里两次只差标点，判稳定
    let dir = std::env::temp_dir().join(format!("vdn-asr-norm-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let cache = dir.join("cache.json");
    let key = |rep: usize| format!("abc|{MODEL}|en|{rep}");
    std::fs::write(
        &cache,
        serde_json::json!({key(1): "Hi, there.", key(2): "hi there"}).to_string(),
    )
    .unwrap();
    let mut a = Asr::with_backends(cache, vec![Backend::Local42], LOCAL42, Route::default());
    let (_, stable) = a
        .transcribe(&dir.join("x.wav"), "abc", "en", "a-c00-ext#ff")
        .unwrap();
    assert!(stable);
    assert_eq!(a.summary()["unstable"], 0);
    let _ = std::fs::remove_dir_all(&dir);
}

/// 人工跑：数一份旧缓存里两次转写的不一致条数，旧判法（逐字）和新判法（[`same_text`]）各一个数。
/// `VDN_ASR_CACHE=<asr_cache.json> cargo test -p vdn-eval --lib count_unstable_in_cache -- --ignored --nocapture`
#[test]
#[ignore = "要一份真缓存，人工跑"]
fn count_unstable_in_cache() {
    let p = std::env::var("VDN_ASR_CACHE").expect("设 VDN_ASR_CACHE");
    let c: BTreeMap<String, String> =
        serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap();
    let (mut pairs, mut raw, mut norm) = (0, 0, 0);
    for (k, a) in &c {
        let Some(head) = k.strip_suffix("|1") else {
            continue;
        };
        let Some(b) = c.get(&format!("{head}|2")) else {
            continue;
        };
        let lang = head.rsplit('|').next().unwrap();
        pairs += 1;
        raw += usize::from(a != b);
        norm += usize::from(!same_text(a, b, lang));
    }
    println!("两次都有 {pairs} 条 · 逐字不一致 {raw} · 分词后不一致 {norm}");
}
