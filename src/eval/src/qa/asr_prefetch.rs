//! [`Asr::prefetch`]：一段要识别的几条音频并发发出去（大总管 2026-11-25：8 路），分两批——先发缺的前几次，
//! 前两次分词后不同的再补第 3 次（A，判法正本 `asr_vote`）。从 `asr.rs` 拆出（文件 ≤ 600 行）。

use std::path::PathBuf;

use super::{Asr, Backend, OPENROUTER, repeats_for, same_text};

impl Asr {
    /// 预取：把这批音频（路径, sha256, 语言, 片段 id）里走 OpenRouter、还没缓存的各次转写**并发**发出去，
    /// 结果按 sha256 写进缓存。之后 [`Asr::transcribe`] 照旧逐条取，都命中缓存，所以成绩与完成先后无关。
    /// 分两批：先发缺的前几次；前两次分词后不同的，再并发补第 3 次（A，`asr_vote`）。
    /// 预取失败的那几次不写缓存，[`Asr::transcribe`] 会再串行试一次并报出原因。本机后端不预取。
    pub fn prefetch(&mut self, items: &[(PathBuf, String, String, String)]) {
        use crate::qa::asr_vote::{TIE_BREAK_REP, Vote, vote_on};
        let model = crate::qa::asr_or::OR_MODEL;
        let key = |sha: &str, lang: &str, r: usize| format!("{sha}|{model}|{lang}|{r}");
        let mine: Vec<_> = items
            .iter()
            .filter(|(_, _, _, clip)| self.backend_for(clip) == OPENROUTER)
            .map(|(p, sha, lang, _)| (p, sha, lang, repeats_for(sha, self.single)))
            .collect();
        let first: Vec<_> = mine
            .iter()
            .flat_map(|&(p, sha, lang, n)| (1..=n).map(move |r| (p, sha, lang, r)))
            .collect();
        self.send(&first);
        let texts = |a: &Asr, sha: &str, lang: &str, upto: usize| -> Option<Vec<String>> {
            (1..=upto)
                .map(|r| a.cache.get(&key(sha, lang, r)).cloned())
                .collect()
        };
        let third: Vec<_> = mine
            .iter()
            .filter(|&&(_, sha, lang, n)| {
                n >= 2
                    && texts(self, sha, lang, 2)
                        .is_some_and(|t| vote_on(&t, lang) == Vote::NeedTieBreak)
            })
            .map(|&(p, sha, lang, _)| (p, sha, lang, TIE_BREAK_REP))
            .collect();
        self.send(&third);
        // 到手的当场计入复核与判法（评分那边可能因为参考转写为空就提前返回，不能靠它计；codex 第 5 轮 P2 #4）
        for &(_, sha, lang, n) in &mine {
            let Some(t) = texts(self, sha, lang, 2).filter(|_| n >= 2) else {
                continue;
            };
            let id = format!("{sha}|{model}");
            self.count_check(id.clone(), same_text(&t[1], &t[0], lang));
            if !same_text(&t[1], &t[0], lang) {
                // 前两次不同：先记「要补调」；第 3 次到手才改成 Some(_)，没到手（报错、停发）留 None
                let ok = texts(self, sha, lang, TIE_BREAK_REP)
                    .map(|t3| matches!(vote_on(&t3, lang), Vote::Use(_)));
                let slot = self.votes.entry(id).or_insert(None);
                if let Some(ok) = ok {
                    *slot = Some(ok);
                }
            }
        }
    }

    /// 并发发一批（路径, sha256, 语言, 第几次）：已缓存、已失败（codex 第 6 轮 P1 #1）、本批已排上的都不再发；停发后一个不发。
    pub(super) fn send(&mut self, want: &[(&PathBuf, &String, &String, usize)]) {
        let mut keys = Vec::new();
        let mut jobs = Vec::new();
        for &(path, sha, lang, rep) in want {
            let key = format!("{sha}|{}|{lang}|{rep}", crate::qa::asr_or::OR_MODEL);
            if self.cache.contains_key(&key)
                || self.failed.contains_key(&key)
                || keys.contains(&key)
            {
                continue;
            }
            let Ok((x, sr)) = crate::wav::read(path) else {
                continue;
            };
            keys.push(key);
            jobs.push((x, sr, lang.clone()));
        }
        let workers = self.workers;
        let results = match self.backends.get_mut(OPENROUTER) {
            Some(Backend::OpenRouter(c)) if !jobs.is_empty() && self.halted.is_none() => {
                c.transcribe_many(&jobs, workers)
            }
            _ => Vec::new(),
        };
        self.sync_review();
        let keys = if results.is_empty() { Vec::new() } else { keys };
        let mut any = false;
        for (k, r) in keys.into_iter().zip(results) {
            match r {
                Ok(t) => {
                    self.calls += 1;
                    self.cache.insert(k, t);
                    any = true;
                }
                Err(e) => {
                    self.fail(k, e);
                }
            }
        }
        if any {
            self.save();
        }
    }
}
