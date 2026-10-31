//! 本机 Qwen3-ASR（42model 服务，`http://127.0.0.1:11520`，D 记录：人选 Qwen3-ASR-Plus 1.7B，不用 whisper）。
//!
//! 调 `curl`（不经 shell，参数逐个传），不引 HTTP 依赖。每个文件转写 `REPEATS` 次（多次试验），
//! 缓存键 = 文件 sha256 | 模型 | 语言 | 第几次，缓存文件在 `_tmp/exp007/asr_cache.json`（仓外）。
//! 口径与 exp003 `score.py` 一致：temperature 0、response_format json、空白归一。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

pub const URL: &str = "http://127.0.0.1:11520/v1/audio/transcriptions";
/// 本机服务的固定占位 key，不是密钥（同 exp003 score.py）；可用环境变量 VDN_ASR_KEY 覆盖
const DEFAULT_KEY: &str = "42model";
pub const MODEL: &str = "qwen3-asr-plus:1.7b";
pub const REPEATS: usize = 2;

pub struct Asr {
    cache_path: PathBuf,
    cache: BTreeMap<String, String>,
    pub calls: usize,
}

impl Asr {
    pub fn new(cache_path: PathBuf) -> Asr {
        let cache = std::fs::read_to_string(&cache_path)
            .ok()
            .and_then(|t| serde_json::from_str::<BTreeMap<String, String>>(&t).ok())
            .unwrap_or_default();
        Asr {
            cache_path,
            cache,
            calls: 0,
        }
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

    fn once(&mut self, path: &Path, sha: &str, lang: &str, rep: usize) -> Result<String, String> {
        let key = format!("{sha}|{MODEL}|{lang}|{rep}");
        if let Some(t) = self.cache.get(&key) {
            return Ok(t.clone());
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
        self.cache.insert(key, text.clone());
        let _ = std::fs::create_dir_all(self.cache_path.parent().unwrap_or(Path::new(".")));
        let _ = std::fs::write(
            &self.cache_path,
            serde_json::to_string_pretty(&self.cache).unwrap_or_default(),
        );
        Ok(text)
    }

    /// 转写 `REPEATS` 次，返回 (第一次文本, 是否每次逐字相同)。
    pub fn transcribe(
        &mut self,
        path: &Path,
        sha: &str,
        lang: &str,
    ) -> Result<(String, bool), String> {
        let first = self.once(path, sha, lang, 1)?;
        let mut stable = true;
        for rep in 2..=REPEATS {
            stable &= self.once(path, sha, lang, rep)? == first;
        }
        Ok((first, stable))
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

    #[test]
    fn zh_tokens_are_chars() {
        assert_eq!(tokens("你好，世界", "zh").len(), 4);
    }
}
