//! 成绩单输出：一张 Markdown 表（场景 × 项 → 分数 · 过不过 · 拖后腿的声音）+ 一份 JSON（同样的内容，机读）。

use serde_json::{Value, json};

use super::ClipRow;
use super::rules::{SceneCard, CardStatus, summarize, worst};
use crate::files::serde_name;

/// 整张成绩单。
pub struct Card {
    pub cards: Vec<SceneCard>,
    pub worst: Vec<usize>,
    pub skipped: Vec<(String, String)>,
    pub clips: usize,
}

impl Card {
    pub fn build(rows: &[ClipRow], skipped: Vec<(String, String)>) -> Card {
        let cards = summarize(rows);
        let worst = worst(&cards);
        Card {
            cards,
            worst,
            skipped,
            clips: rows.len(),
        }
    }

    pub fn to_json(&self, meta: Value) -> Value {
        json!({
            "doc": "场景成绩单（vdn-eval scorecard）。规则见 specs/rubric/scorecard.md；感知项全部 L0（只作约束，不当奖励）",
            "meta": meta,
            "clips_scored": self.clips,
            "scenes": self.cards,
            "worst": self.worst.iter().map(|&i| key(&self.cards[i])).collect::<Vec<_>>(),
            "skipped": self.skipped.iter().map(|(k, w)| json!({"what": k, "why": w})).collect::<Vec<_>>(),
        })
    }

    pub fn to_markdown(&self, title: &str) -> String {
        let mut s = format!("# {title}\n\n");
        let n = |st: CardStatus| self.cards.iter().filter(|c| c.status == st).count();
        s += &format!(
            "共 {} 个 场景 × 档 × 设备 × 系统 组合、{} 段：过 {} · 不过 {} · 待测 {}。感知项全部 L0（只作约束，不当奖励）。\n\n",
            self.cards.len(),
            self.clips,
            n(CardStatus::Pass),
            n(CardStatus::Fail),
            n(CardStatus::Unknown)
        );
        s += "## 最差的场景\n\n";
        if self.worst.is_empty() {
            s += "（没有不过或待测的场景）\n\n";
        }
        for &i in &self.worst {
            let c = &self.cards[i];
            let bad: Vec<&str> = c
                .items
                .iter()
                .filter(|x| x.status == CardStatus::Fail)
                .map(|x| x.v)
                .collect();
            s += &format!(
                "- **{}** · {}：{}{}\n",
                key(c),
                c.status.zh(),
                if c.veto_failed.is_empty() {
                    String::new()
                } else {
                    format!("否决项不过 {} · ", c.veto_failed.join(" "))
                },
                if bad.is_empty() {
                    "没有不过的项（有待测）".to_string()
                } else {
                    format!("不过的项 {}", bad.join(" "))
                }
            );
        }
        s += "\n## 每个场景\n\n";
        for c in &self.cards {
            s += &format!("### {} — {}（{} 段）\n\n", key(c), c.status.zh(), c.clips);
            s += "| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |\n|---|---|---|---|---|---|---|\n";
            for i in &c.items {
                let f = |v: Option<f64>| v.map_or("—".into(), |x| format!("{x:.3}"));
                s += &format!(
                    "| {} | {} | {} / {} / {} | {} {} | {}{} | {} | {} |\n",
                    i.v,
                    if i.veto { "是" } else { "" },
                    i.pass,
                    i.fail,
                    i.unknown,
                    f(i.mean),
                    i.unit,
                    f(i.worst),
                    i.worst_clip
                        .as_ref()
                        .map_or(String::new(), |w| format!("（{w}）")),
                    f(i.threshold),
                    i.status.zh()
                );
            }
            let l = &c.listen;
            s += &format!(
                "| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | {} / {} / {} | | | Δ ≥ 0 | {} |\n\n",
                l.pass,
                l.fail,
                l.unknown,
                l.status.map_or("待测", CardStatus::zh)
            );
            if !l.to_human.is_empty() {
                s += "送人耳：\n";
                for (clip, why) in &l.to_human {
                    s += &format!("- {clip}：{why}\n");
                }
                s += "\n";
            }
            if !c.drags.is_empty() {
                s += "拖后腿的声音（有这类声音的段比没有的段差多少）：\n";
                for d in &c.drags {
                    s += &match d.gap {
                        Some(g) => format!(
                            "- {} · `{}`（{}）：差 {g:+.3}（有 {} 段 {:.3} · 没有 {} 段 {:.3}）\n",
                            d.v,
                            d.cls,
                            serde_name(&d.group),
                            d.n_with,
                            d.mean_with,
                            d.n_without,
                            d.mean_without
                        ),
                        None => format!(
                            "- {} · `{}`：样本不够（有 {} 段 · 没有 {} 段，每组要 ≥ {}）\n",
                            d.v,
                            d.cls,
                            d.n_with,
                            d.n_without,
                            super::rules::MIN_SPLIT
                        ),
                    };
                }
                s += "\n";
            }
        }
        if !self.skipped.is_empty() {
            s += "## 没跑的\n\n";
            for (k, w) in &self.skipped {
                s += &format!("- {k}：{w}\n");
            }
        }
        s
    }
}

fn key(c: &SceneCard) -> String {
    format!(
        "{} · {} · {} · {}",
        c.scene.as_str(),
        serde_name(&c.tier),
        c.device,
        c.system
    )
}
