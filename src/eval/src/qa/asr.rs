//! 语音识别（吃字 V-EAT-ASR 用）。两个后端，**默认 OpenRouter**（D103 · D104），本机 42model 留作可选：
//!
//! - `openrouter`：云端 Qwen3-ASR 1.7B，见 [`super::asr_or`]（型号正本、`data_collection = deny`、费用上限都在那里）
//! - `local42`：本机 42model 服务里的 Qwen3-ASR-Plus 1.7B（`http://127.0.0.1:11520`），只有 macOS 版
//!
//! 选法：环境变量 `VDN_ASR_BACKEND`（`openrouter` 默认 · `local42`）定默认后端；`VDN_ASR_ROUTE` 指向一个 JSON 文件
//! `{"default": "openrouter", "clips": {"<片段 id>": "local42", …}}` 可按段单独指定（大总管 2026-11-25：人怎么定都不用返工）。
//! 哪些段实际用了哪个后端记在 [`Asr::used`]，写进成绩单 `meta.asr`。
//!
//! 每个文件转写 `REPEATS` 次（多次试验），缓存键 = 文件 sha256 | 后端型号 | 语言 | 第几次（按后端分开，不同后端的转写不混）。
//! 本机后端口径与 exp003 `score.py` 一致：temperature 0、response_format json、空白归一。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

pub const URL: &str = "http://127.0.0.1:11520/v1/audio/transcriptions";
/// 本机服务的固定占位 key，不是密钥（同 exp003 score.py）；可用环境变量 VDN_ASR_KEY 覆盖
const DEFAULT_KEY: &str = "42model";
pub const MODEL: &str = "qwen3-asr-plus:1.7b";
/// 每条音频转写几次（第 2 次是稳定性复核：两次不同就剔除这条，rubric V7）。基线用 2（大总管 2026-11-25）
pub const REPEATS: usize = 2;
/// 「以后的模型改成 1 次」时（`VDN_ASR_REPEATS=1`），按音频 sha256 固定抽这个比例照旧转 2 次复核
pub const SPOT_FRACTION: f64 = 0.10;
/// 基线量出的不一致率不超过它，以后的模型才可以改成 1 次 + 抽查（大总管 2026-11-25 定 0）
pub const MAX_UNSTABLE_RATE: f64 = 0.0;

/// 这条音频转几次：默认 [`REPEATS`]；`VDN_ASR_REPEATS=1` 时只转 1 次，但 sha256 落在前 [`SPOT_FRACTION`] 的照旧 2 次。
/// 抽中与否只看音频内容，同一条音频每次都一样。
pub fn repeats_for(sha: &str, single: bool) -> usize {
    if !single {
        return REPEATS;
    }
    let head = u64::from_str_radix(sha.get(..8).unwrap_or("ffffffff"), 16).unwrap_or(u64::MAX);
    if (head as f64) < SPOT_FRACTION * f64::from(u32::MAX) {
        REPEATS
    } else {
        1
    }
}

/// 后端名（`VDN_ASR_BACKEND`、路由文件、`meta.asr` 里用的写法）。
pub const OPENROUTER: &str = "openrouter";
pub const LOCAL42: &str = "local42";

/// 一个后端：本机服务，或 OpenRouter 客户端。
pub enum Backend {
    Local42,
    OpenRouter(super::asr_or::OpenRouter),
}

impl Backend {
    pub fn name(&self) -> &'static str {
        match self {
            Backend::Local42 => LOCAL42,
            Backend::OpenRouter(_) => OPENROUTER,
        }
    }
    pub fn model(&self) -> &'static str {
        match self {
            Backend::Local42 => MODEL,
            Backend::OpenRouter(_) => super::asr_or::OR_MODEL,
        }
    }
}

/// 按段选后端：默认一个，个别片段另指定。
#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize)]
pub struct Route {
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub clips: BTreeMap<String, String>,
}

impl Route {
    /// 这一段用哪个后端（片段 id 去掉 `#…` 的缓存后缀再查）。
    pub fn pick<'a>(&'a self, clip: &str, fallback: &'a str) -> &'a str {
        let id = clip.split('#').next().unwrap_or(clip);
        self.clips
            .get(id)
            .map(String::as_str)
            .or(self.default.as_deref())
            .unwrap_or(fallback)
    }
}

pub struct Asr {
    cache_path: PathBuf,
    cache: BTreeMap<String, String>,
    pub calls: usize,
    backends: BTreeMap<&'static str, Backend>,
    default: &'static str,
    route: Route,
    /// 后端名 → 实际用它转写过的片段 id（写进 `meta.asr`）
    pub used: BTreeMap<&'static str, std::collections::BTreeSet<String>>,
    /// 只转 1 次 + 抽查（`VDN_ASR_REPEATS=1`）
    pub single: bool,
    /// 做过多次转写的音频条数、其中不一致的条数（量不一致率用，写进 `meta.asr`）
    pub checked: usize,
    pub unstable: usize,
    /// OpenRouter 一批并发几路
    pub workers: usize,
    /// 已经失败过的转写（缓存键 → 原因）：再问到直接报这个错，不再发请求（codex 第 5 轮 P1 #3）
    failed: BTreeMap<String, String>,
    /// 遇到永久错误（数据政策、余额、上限、账本坏）后停发全部 OpenRouter 请求，原因在这里
    pub halted: Option<String>,
    /// 已计入 checked / unstable 的音频（sha256 | 型号），同一条只计一次
    counted: std::collections::BTreeSet<String>,
}

/// 这个错误是不是永久的：再发也一样（不重试的 4xx、到上限、账本坏或写不了）。
pub fn is_permanent(e: &str) -> bool {
    ["不重试", "上限", "账本"].iter().any(|k| e.contains(k))
}

impl Asr {
    /// 只用本机 42model（旧行为；测试与 `VDN_ASR_BACKEND=local42` 用）。
    pub fn new(cache_path: PathBuf) -> Asr {
        Asr::with_backends(
            cache_path,
            vec![Backend::Local42],
            LOCAL42,
            Route::default(),
        )
    }

    pub fn with_backends(
        cache_path: PathBuf,
        backends: Vec<Backend>,
        default: &'static str,
        route: Route,
    ) -> Asr {
        let cache = std::fs::read_to_string(&cache_path)
            .ok()
            .and_then(|t| serde_json::from_str::<BTreeMap<String, String>>(&t).ok())
            .unwrap_or_default();
        Asr {
            cache_path,
            cache,
            calls: 0,
            backends: backends.into_iter().map(|b| (b.name(), b)).collect(),
            default,
            route,
            used: BTreeMap::new(),
            single: false,
            checked: 0,
            unstable: 0,
            workers: super::asr_or::ASR_CONCURRENCY,
            failed: BTreeMap::new(),
            halted: None,
            counted: std::collections::BTreeSet::new(),
        }
    }

    /// 按环境变量组装（见文件头）。返回 `Err(原因)` 时这次成绩单的吃字整列判「不知道」。
    /// 默认后端 OpenRouter：要有密钥（`asr_or::load_key`）；路由文件里点名 `local42` 的段要本机服务在。
    pub fn from_env(cache_path: PathBuf, root: &Path) -> Result<Asr, String> {
        let default: &'static str = match std::env::var("VDN_ASR_BACKEND").as_deref() {
            Err(_) | Ok("openrouter") => OPENROUTER,
            Ok("local42") => LOCAL42,
            Ok(x) => {
                return Err(format!(
                    "VDN_ASR_BACKEND 只认 openrouter / local42，收到 {x}"
                ));
            }
        };
        let route: Route = match std::env::var("VDN_ASR_ROUTE") {
            Ok(p) => {
                let t =
                    std::fs::read_to_string(&p).map_err(|e| format!("VDN_ASR_ROUTE {p}：{e}"))?;
                serde_json::from_str(&t).map_err(|e| format!("VDN_ASR_ROUTE {p}：{e}"))?
            }
            Err(_) => Route::default(),
        };
        let mut wanted: std::collections::BTreeSet<&str> =
            route.clips.values().map(String::as_str).collect();
        wanted.insert(route.default.as_deref().unwrap_or(default));
        let mut backends = Vec::new();
        for w in wanted {
            match w {
                OPENROUTER => {
                    let key = super::asr_or::load_key(root)
                        .ok_or("没有 OPENROUTER_API_KEY（环境变量或仓库根 .env.local）")?;
                    let ledger = root.join(super::asr_or::LEDGER_REL);
                    backends.push(Backend::OpenRouter(super::asr_or::OpenRouter::new(
                        super::asr_or::OR_URL,
                        key,
                        ledger,
                    )));
                }
                LOCAL42 if Asr::alive() => backends.push(Backend::Local42),
                LOCAL42 => return Err("要用本机 42model，但服务不在（127.0.0.1:11520）".into()),
                x => {
                    return Err(format!(
                        "路由里的后端 {x} 不认识（只认 openrouter / local42）"
                    ));
                }
            }
        }
        let mut a = Asr::with_backends(cache_path, backends, default, route);
        a.single = match std::env::var("VDN_ASR_REPEATS").as_deref() {
            Err(_) | Ok("2") => false,
            Ok("1") => true,
            Ok(x) => return Err(format!("VDN_ASR_REPEATS 只认 1 或 2，收到 {x}")),
        };
        Ok(a)
    }

    /// 写进成绩单 `meta.asr` 的摘要：默认后端、每个后端的型号与实际用到的段数。
    pub fn summary(&self) -> Value {
        let b: serde_json::Map<String, Value> = self
            .backends
            .values()
            .map(|x| {
                let n = self.used.get(x.name()).map_or(0, |s| s.len());
                (
                    x.name().to_string(),
                    serde_json::json!({"model": x.model(), "clips": n}),
                )
            })
            .collect();
        let rate = (self.checked > 0).then(|| self.unstable as f64 / self.checked as f64);
        serde_json::json!({"default": self.default, "backends": b, "calls": self.calls,
            "repeats": if self.single { "1 + 抽查" } else { "2" }, "spot_fraction": SPOT_FRACTION,
            "checked": self.checked, "unstable": self.unstable, "unstable_rate": rate})
    }

    /// 服务在不在（GET 根路径能连上即可）。
    pub fn alive() -> bool {
        Command::new("curl")
            .args([
                "-s",
                "-m",
                "3",
                "-o",
                "/dev/null",
                "-w",
                "%{http_code}",
                "http://127.0.0.1:11520/",
            ])
            .output()
            .is_ok_and(|o| {
                let code = String::from_utf8_lossy(&o.stdout);
                o.status.success() && code.trim() != "000"
            })
    }

    fn once(
        &mut self,
        path: &Path,
        sha: &str,
        lang: &str,
        rep: usize,
        backend: &'static str,
    ) -> Result<String, String> {
        let b = self
            .backends
            .get(backend)
            .ok_or_else(|| format!("后端 {backend} 没装好"))?;
        let model = b.model();
        // 本机后端沿用旧的键（不带后端名），旧缓存还能用；OpenRouter 的键带型号，和本机的分开
        let key = format!("{sha}|{model}|{lang}|{rep}");
        if let Some(t) = self.cache.get(&key) {
            return Ok(t.clone());
        }
        if let Some(Backend::OpenRouter(c)) = self.backends.get_mut(backend) {
            let (x, sr) = crate::wav::read(path)?;
            let text = c.transcribe(&x, sr, lang)?;
            self.calls += 1;
            return Ok(self.remember(key, text));
        }
        let file_arg = format!("file=@{};type=audio/wav", path.display());
        let o = Command::new("curl")
            .args(["-sS", "-f", "-m", "600", "-X", "POST", URL, "-H"])
            .arg(format!(
                "Authorization: Bearer {}",
                std::env::var("VDN_ASR_KEY").unwrap_or_else(|_| DEFAULT_KEY.into())
            ))
            .args([
                "-F",
                &format!("model={MODEL}"),
                "-F",
                &format!("language={lang}"),
            ])
            .args([
                "-F",
                "temperature=0",
                "-F",
                "response_format=json",
                "-F",
                &file_arg,
            ])
            .output()
            .map_err(|e| format!("调不起 curl：{e}"))?;
        self.calls += 1;
        if !o.status.success() {
            return Err(format!(
                "ASR 请求失败（curl 退出码 {:?}）：{}",
                o.status.code(),
                String::from_utf8_lossy(&o.stderr).trim()
            ));
        }
        let v: Value =
            serde_json::from_slice(&o.stdout).map_err(|e| format!("ASR 返回不是 JSON：{e}"))?;
        let text = v["text"]
            .as_str()
            .ok_or_else(|| format!("ASR 返回缺 text：{v}"))?
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        Ok(self.remember(key, text))
    }

    fn remember(&mut self, key: String, text: String) -> String {
        self.cache.insert(key, text.clone());
        let _ = std::fs::create_dir_all(self.cache_path.parent().unwrap_or(Path::new(".")));
        let _ = std::fs::write(
            &self.cache_path,
            serde_json::to_string_pretty(&self.cache).unwrap_or_default(),
        );
        text
    }

    /// 转写 `REPEATS` 次，返回 (第一次文本, 是否每次逐字相同)。`clip` = 片段 id（按路由选后端，记进 [`Asr::used`]）。
    pub fn transcribe(
        &mut self,
        path: &Path,
        sha: &str,
        lang: &str,
        clip: &str,
    ) -> Result<(String, bool), String> {
        let backend = self.backend_for(clip);
        let first = self.once(path, sha, lang, 1, backend)?;
        let mut stable = true;
        let n = repeats_for(sha, self.single);
        for rep in 2..=n {
            stable &= self.once(path, sha, lang, rep, backend)? == first;
        }
        if n > 1 {
            self.checked += 1;
            self.unstable += usize::from(!stable);
        }
        let id = clip.split('#').next().unwrap_or(clip).to_string();
        self.used.entry(backend).or_default().insert(id);
        Ok((first, stable))
    }

    /// 预取：把这批音频（路径, sha256, 语言, 片段 id）里走 OpenRouter、还没缓存的各次转写**并发**发出去，
    /// 结果按 sha256 写进缓存。之后 [`Asr::transcribe`] 照旧逐条取，都命中缓存，所以成绩与完成先后无关。
    /// 预取失败的那几次不写缓存，[`Asr::transcribe`] 会再串行试一次并报出原因。本机后端不预取。
    pub fn prefetch(&mut self, items: &[(PathBuf, String, String, String)]) {
        let mut keys = Vec::new();
        let mut jobs = Vec::new();
        for (path, sha, lang, clip) in items {
            if self.backend_for(clip) != OPENROUTER {
                continue;
            }
            for rep in 1..=repeats_for(sha, self.single) {
                let key = format!("{sha}|{}|{lang}|{rep}", super::asr_or::OR_MODEL);
                if self.cache.contains_key(&key) || keys.contains(&key) {
                    continue;
                }
                let Ok((x, sr)) = crate::wav::read(path) else {
                    continue;
                };
                keys.push(key);
                jobs.push((x, sr, lang.clone()));
            }
        }
        if jobs.is_empty() {
            return;
        }
        let workers = self.workers;
        let Some(Backend::OpenRouter(c)) = self.backends.get_mut(OPENROUTER) else {
            return;
        };
        let results = c.transcribe_many(&jobs, workers);
        let mut any = false;
        for (k, r) in keys.into_iter().zip(results) {
            if let Ok(t) = r {
                self.calls += 1;
                self.cache.insert(k, t);
                any = true;
            }
        }
        if any {
            self.save();
        }
    }

    fn save(&self) {
        let _ = std::fs::create_dir_all(self.cache_path.parent().unwrap_or(Path::new(".")));
        let _ = std::fs::write(
            &self.cache_path,
            serde_json::to_string_pretty(&self.cache).unwrap_or_default(),
        );
    }

    /// 这一段用哪个后端（路由 → 默认）。路由点名了没装的后端时也照返回，`once` 会报「没装好」。
    pub fn backend_for(&self, clip: &str) -> &'static str {
        match self.route.pick(clip, self.default) {
            OPENROUTER => OPENROUTER,
            LOCAL42 => LOCAL42,
            _ => self.default,
        }
    }
}

/// 分词：中文按字，英文按词（小写、只留字母数字与撇号）。
pub fn tokens(text: &str, lang: &str) -> Vec<String> {
    if lang == "zh" {
        text.chars()
            .filter(|c| c.is_alphanumeric())
            .map(|c| c.to_lowercase().to_string())
            .collect()
    } else {
        text.split_whitespace()
            .map(|w| {
                w.chars()
                    .filter(|c| c.is_alphanumeric() || *c == '\'')
                    .collect::<String>()
                    .to_lowercase()
            })
            .filter(|w| !w.is_empty())
            .collect()
    }
}

/// Levenshtein 对齐，返回 (删除, 替换, 插入)。回溯优先级与 exp003 `edit_ops` 一致：对角 → 删除 → 插入。
pub fn edit_ops(r: &[String], h: &[String]) -> (usize, usize, usize) {
    let (n, m) = (r.len(), h.len());
    let mut d = vec![vec![0usize; m + 1]; n + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, v) in d[0].iter_mut().enumerate() {
        *v = j;
    }
    for i in 1..=n {
        for j in 1..=m {
            let sub = d[i - 1][j - 1] + usize::from(r[i - 1] != h[j - 1]);
            d[i][j] = (d[i - 1][j] + 1).min(d[i][j - 1] + 1).min(sub);
        }
    }
    let (mut i, mut j) = (n, m);
    let (mut del, mut sub, mut ins) = (0, 0, 0);
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && d[i][j] == d[i - 1][j - 1] + usize::from(r[i - 1] != h[j - 1]) {
            sub += usize::from(r[i - 1] != h[j - 1]);
            i -= 1;
            j -= 1;
        } else if i > 0 && d[i][j] == d[i - 1][j] + 1 {
            del += 1;
            i -= 1;
        } else {
            ins += 1;
            j -= 1;
        }
    }
    (del, sub, ins)
}

#[cfg(test)]
#[path = "asr_conc_tests.rs"]
mod conc_tests;

#[cfg(test)]
mod tests {
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
        let r: Route = serde_json::from_str(
            r#"{"default": "openrouter", "clips": {"cafe-c00-mba": "local42"}}"#,
        )
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
        let route: Route =
            serde_json::from_str(r#"{"clips": {"b-c00-ext": "openrouter"}}"#).unwrap();
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

    #[test]
    fn zh_tokens_are_chars() {
        assert_eq!(tokens("你好，世界", "zh").len(), 4);
    }
}
