//! 成绩单输出：一张 Markdown 表（场景 × 项 → 分数 · 过不过 · 拖后腿的声音）+ 一份 JSON（同样的内容，机读）。

use serde_json::{Value, json};

use super::rules::{CardStatus, ItemCell, SceneCard, summarize, worst};
use super::{ClipRow, Missing};
use crate::files::serde_name;

/// 成绩单的两个文件名。
pub const FILES: [&str; 2] = ["scorecard.json", "scorecard.md"];

/// 输出目录里已有成绩单就报错：不覆盖旧结果（CLAUDE.md 第五节第 1 条；codex 第 5 轮 P1 #1）。先查再跑，免得白跑。
pub fn ensure_fresh(dir: &std::path::Path) -> Result<(), String> {
    for f in FILES {
        let p = dir.join(f);
        if p.exists() {
            return Err(format!("{} 已存在，不覆盖：换一个 --out 目录", p.display()));
        }
    }
    Ok(())
}

/// 只新建写一个文件：已存在就报错，不覆盖。
pub fn write_new_file(p: &std::path::Path, text: &str) -> Result<(), String> {
    use std::io::Write;
    let mut h = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(p)
        .map_err(|e| format!("{}：{e}", p.display()))?;
    h.write_all(text.as_bytes()).map_err(|e| e.to_string())
}

/// 写成绩单：两个文件都用「只新建」打开，已存在就报错（与 [`ensure_fresh`] 之间被别的进程抢先写入也不会覆盖）。
pub fn write_new(dir: &std::path::Path, json: &str, md: &str) -> Result<(), String> {
    ensure_fresh(dir)?;
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    for (f, text) in FILES.iter().zip([json, md]) {
        write_new_file(&dir.join(f), text)?;
    }
    Ok(())
}

/// 汇总规则版本：`v1` = 旧规则（任一段不过即不过）；`v2` = 人定 D85 的新规则（默认）。人改主意时用 `--rules v1` 切回。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rules {
    V1,
    V2,
}

impl Rules {
    pub fn parse(s: &str) -> Option<Rules> {
        match s {
            "v1" => Some(Rules::V1),
            "v2" => Some(Rules::V2),
            _ => None,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Rules::V1 => "v1",
            Rules::V2 => "v2",
        }
    }
}

/// 整张成绩单。
#[derive(Debug)]
pub struct Card {
    pub cards: Vec<SceneCard>,
    pub worst: Vec<usize>,
    pub missing: Vec<Missing>,
    pub clips: usize,
    pub rules: Rules,
}

impl Card {
    /// 按新规则（v2）建表。
    pub fn build(rows: &[ClipRow], missing: Vec<Missing>) -> Card {
        Card::build_with(rows, missing, Rules::V2)
    }

    pub fn build_with(rows: &[ClipRow], missing: Vec<Missing>, rules: Rules) -> Card {
        let cards = match rules {
            Rules::V1 => summarize(rows, &missing),
            Rules::V2 => super::rules_v2::summarize(rows, &missing),
        };
        let worst = worst(&cards);
        Card {
            cards,
            worst,
            missing,
            clips: rows.len(),
            rules,
        }
    }

    pub fn to_json(&self, meta: Value) -> Value {
        json!({
            "doc": "场景成绩单（vdn-eval scorecard）。规则见 specs/rubric/scorecard.md；感知项全部 L0（只作约束，不当奖励）",
            "meta": meta,
            "rules": self.rules.as_str(),
            "clips_scored": self.clips,
            "scenes": self.cards,
            "worst": self.worst.iter().map(|&i| key(&self.cards[i])).collect::<Vec<_>>(),
            "missing": self.missing,
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
        s += &match self.rules {
            Rules::V1 => "汇总规则 v1（旧）：每项任一段不过即不过。\n\n".to_string(),
            Rules::V2 => "汇总规则 v2（人定 D85）：物理硬事实项逐段否决；吃字单段多错 ≥ 5 字否决；\
                其余项按组合均值的 95% 区间判（区间全在线内 = 过，全在线外 = 不过且经 Holm 校正，跨线 = 待测）；\
                听感允许比原声差 0.1 分。「判法」列：逐段 · ≥5字 · 区间 · 只报 · 只报·旧口径。\n\n\
                人声忽大忽小（V-PUMPV）· 音色偏差（V-TIMBRE）· 弱音丢失（V-WEAK）在「干净人声单独过一遍模型」的输出上量（人定 D85 题 2）：\
                只量得到安静时的人声损伤，嘈杂时模型对人声的伤害这三项量不到。\n\n"
                .to_string(),
        };
        s += "## 最差的场景\n\n";
        if self.worst.is_empty() {
            s += "（没有不过或待测的场景）\n\n";
        }
        for &i in &self.worst {
            let c = &self.cards[i];
            let bad = c.failed_items();
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
            if !c.missing.is_empty() {
                s += &format!(
                    "缺 {} 段没打分（见文末「没跑成的」），本场景最多判「待测」。\n\n",
                    c.missing.len()
                );
            }
            s += "| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 95% 区间 | 最差（段） | 过线 | 判法 | 判定 |\n|---|---|---|---|---|---|---|---|---|\n";
            for i in &c.items {
                let f = |v: Option<f64>| v.map_or("—".into(), |x| format!("{x:.3}"));
                s += &format!(
                    "| {} | {} | {} / {} / {} | {} {} | {} | {}{} | {} | {} | {} |\n",
                    i.v,
                    if i.veto { "是" } else { "" },
                    i.pass,
                    i.fail,
                    i.unknown,
                    f(i.mean),
                    i.unit,
                    ci_text(i),
                    f(i.worst),
                    i.worst_clip
                        .as_ref()
                        .map_or(String::new(), |w| format!("（{w}）")),
                    f(i.threshold),
                    rule_short(i.rule),
                    if i.report_only {
                        format!("{}（只报不判）", i.status.zh())
                    } else {
                        i.status.zh().to_string()
                    }
                );
            }
            let l = &c.listen;
            s += &format!(
                "| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | {} / {} / {} | SCOREQ Δ {} · Distill-MOS Δ {} | | | {} | {} | {} |\n\n",
                l.pass,
                l.fail,
                l.unknown,
                l.mean_scoreq.map_or("—".into(), |x| format!("{x:+.3}")),
                l.mean_distill.map_or("—".into(), |x| format!("{x:+.3}")),
                l.models
                    .first()
                    .and_then(|m| m.threshold)
                    .map_or("Δ ≥ 0".into(), |t| format!("Δ ≥ {t}")),
                l.models.first().map_or("逐段", |m| rule_short(m.rule)),
                l.status.map_or("待测", CardStatus::zh)
            );
            // 两个评分模型各自的明细（只给人看，不单独决定场景）
            if !l.models.is_empty() {
                s += "| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 95% 区间 | 最差（段） | 判定 |\n|---|---|---|---|---|---|\n";
                for m in &l.models {
                    let f = |v: Option<f64>| v.map_or("—".into(), |x| format!("{x:+.3}"));
                    s += &format!(
                        "| {} | {} / {} / {} | {} | {} | {}{} | {} |\n",
                        m.v,
                        m.pass,
                        m.fail,
                        m.unknown,
                        f(m.mean),
                        ci_text(m),
                        f(m.worst),
                        m.worst_clip
                            .as_ref()
                            .map_or(String::new(), |w| format!("（{w}）")),
                        if m.ci.is_some() { m.status.zh() } else { "—" }
                    );
                }
                s += "\n";
            }
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
        if !self.missing.is_empty() {
            s += "## 没跑成的（所在场景最多判「待测」）\n\n";
            for m in &self.missing {
                s += &format!("- {} · {}：{}\n", m.what, m.system, m.why);
            }
        }
        s
    }
}

fn ci_text(i: &ItemCell) -> String {
    i.ci.as_ref().map_or("—".into(), |c| {
        format!("[{:.3}, {:.3}]（{} 组）", c.lo, c.hi, c.groups)
    })
}

/// 判法的短名（表头图例见 v2 说明行）。
fn rule_short(r: &str) -> &'static str {
    use super::rules_v2 as v2;
    match r {
        x if x == v2::RULE_PHYS => "逐段",
        x if x == v2::RULE_EAT => "≥5字",
        x if x == v2::RULE_MEAN => "区间",
        x if x == v2::RULE_REPORT => "只报",
        x if x == v2::RULE_OLD_INPUT => "只报·旧口径",
        _ => "逐段",
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

#[cfg(test)]
mod header_tests {
    use super::*;

    /// 人定 D85 题 2：v2 成绩单自己要写明三项在干净人声那一路上量、只量得到安静时的人声损伤（收尾第 2 轮 P2 #2）。
    #[test]
    fn v2_markdown_states_clean_pass_limit() {
        let v2 = Card::build_with(&[], Vec::new(), Rules::V2).to_markdown("t");
        assert!(v2.contains("只量得到安静时的人声损伤"), "{v2}");
        assert!(v2.contains("V-PUMPV") && v2.contains("V-TIMBRE") && v2.contains("V-WEAK"));
        let v1 = Card::build_with(&[], Vec::new(), Rules::V1).to_markdown("t");
        assert!(!v1.contains("只量得到安静时的人声损伤"));
    }
}
