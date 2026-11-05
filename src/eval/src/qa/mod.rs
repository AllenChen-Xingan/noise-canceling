//! QA 层：要本机模型的 verifier（V-EAT-ASR · V-DNSMOS · V-NRMOS-SCOREQ · V-NRMOS-DISTILL）。只在本机跑，CI 不跑。

pub mod asr;
pub mod dnsmos;
pub mod nrmos;
mod nrmos_v;

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::{Value, json};

use crate::rng::Rng;
use crate::runner::Extra;
use crate::suite::{Task, resolve};
use crate::synth::{Clip, SR};
use crate::verifiers::Params;
use crate::{Better, Score, wav};

/// 一个片段的吃字计数（集合层 bootstrap 用）。
#[derive(Clone, Debug)]
pub struct EatRow {
    pub system: String,
    pub task: String,
    pub ref_len: usize,
    pub mix_del: usize,
    pub mix_sub: usize,
    pub out_del: usize,
    pub out_sub: usize,
}

/// 参考分词 + 原声的 (删, 替, 插)。
type RefEntry = Result<(Vec<String>, (usize, usize, usize)), String>;

pub struct QaExtra {
    pub asr: Option<asr::Asr>,
    /// DNSMOS 模型路径。ort 一个进程只许一个 Environment（ort rc.15 environment.rs:515），
    /// 而 vdn-models 已建了一个，所以 DNSMOS 在子进程 `vdn-eval dnsmos` 里跑
    pub dnsmos: Option<PathBuf>,
    pub dnsmos_err: Option<String>,
    tmp: PathBuf,
    pub eat: Vec<EatRow>,
    /// 任务 → 参考分词与原声的 (删, 替, 插)，每个任务只转写一次
    refs: BTreeMap<String, RefEntry>,
    mix_mos: BTreeMap<String, f64>,
    /// 评分模型（SCOREQ / Distill-MOS）的模型路径或找不到的原因，及带噪原声的分（按任务 + 模型缓存）
    nrmos: nrmos_v::Models,
}

pub(crate) fn fnv(x: &[f32]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in x {
        for b in v.to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    }
    h
}

impl QaExtra {
    pub fn new(tmp: PathBuf) -> QaExtra {
        let asr = asr::Asr::alive().then(|| asr::Asr::new(tmp.join("asr_cache.json")));
        let (dnsmos, dnsmos_err) =
            match resolve("resources/DNS-Challenge/DNSMOS/DNSMOS/sig_bak_ovr.onnx") {
                Some(p) => (Some(p), None),
                None => (
                    None,
                    Some("找不到 resources/DNS-Challenge/DNSMOS/DNSMOS/sig_bak_ovr.onnx".into()),
                ),
            };
        QaExtra {
            asr,
            dnsmos,
            dnsmos_err,
            tmp,
            eat: Vec::new(),
            refs: BTreeMap::new(),
            mix_mos: BTreeMap::new(),
            nrmos: nrmos_v::Models::locate(),
        }
    }

    pub(crate) fn dump(&self, name: &str, x: &[f32]) -> Result<(PathBuf, String), String> {
        let p = self
            .tmp
            .join("asr-wav")
            .join(format!("{name}-{:016x}.wav", fnv(x)));
        if !p.exists() {
            wav::write(&p, x, SR)?;
        }
        let sha = wav::sha256(&p)?;
        Ok((p, sha))
    }

    fn transcribe(&mut self, name: &str, x: &[f32], lang: &str) -> Result<Vec<String>, String> {
        let (p, sha) = self.dump(name, x)?;
        let a = self.asr.as_mut().ok_or("ASR 服务不在（127.0.0.1:11520）")?;
        let (text, stable) = a.transcribe(&p, &sha, lang)?;
        if !stable {
            return Err(format!("ASR 对 {name} 两次转写不一致（剔除，rubric V7）"));
        }
        Ok(asr::tokens(&text, lang))
    }

    fn eat(&mut self, sys: &str, task: &Task, clip: &Clip, out: &[f32], p: &Params) -> Score {
        const ID: &str = "V-EAT-ASR";
        const U: &str = "百分点";
        if !self.refs.contains_key(&task.id) {
            let r = (|| {
                let rt = self.transcribe(&format!("{}-clean", task.id), &clip.clean, &task.lang)?;
                if rt.is_empty() {
                    return Err("参考转写为空".to_string());
                }
                let mt = self.transcribe(&format!("{}-mix", task.id), &clip.mix, &task.lang)?;
                let ops = asr::edit_ops(&rt, &mt);
                Ok((rt, ops))
            })();
            self.refs.insert(task.id.clone(), r);
        }
        let (rt, (md, ms, mi)) = match self.refs.get(&task.id) {
            Some(Ok(v)) => v.clone(),
            Some(Err(e)) => return Score::unknown(ID, U, Better::Lower, e.clone()),
            None => return Score::unknown(ID, U, Better::Lower, "内部错误"),
        };
        let ot = match self.transcribe(&format!("{}-{sys}", task.id), out, &task.lang) {
            Ok(t) => t,
            Err(e) => return Score::unknown(ID, U, Better::Lower, e),
        };
        let (od, os, oi) = asr::edit_ops(&rt, &ot);
        let l = rt.len() as f64;
        let pp = |a: usize, b: usize| (a as f64 - b as f64) / l * 100.0;
        self.eat.push(EatRow {
            system: sys.to_string(),
            task: task.id.clone(),
            ref_len: rt.len(),
            mix_del: md,
            mix_sub: ms,
            out_del: od,
            out_sub: os,
        });
        Score::judged(
            ID,
            pp(od + os, md + ms),
            U,
            Better::Lower,
            // 过关线只从任务参数取（成绩单标准给 1.0，正本 thresholds 表 V-EAT-ASR · max；025 E2），不在这里写默认值
            p.f("max"),
            &[],
        )
        .with_extra("d_del_pp", pp(od, md))
        .with_extra("d_sub_pp", pp(os, ms))
        .with_extra("d_ins_pp", pp(oi, mi))
        .with_extra("ref_len", l)
        .with_note("单片段点估计；集合层区间见报告「吃字（集合层）」")
    }

    fn mos(&mut self, task: &Task, clip: &Clip, out: &[f32]) -> Score {
        const ID: &str = "V-DNSMOS";
        if self.dnsmos.is_none() {
            return Score::unknown(
                ID,
                "MOS",
                Better::Higher,
                self.dnsmos_err.clone().unwrap_or_default(),
            );
        }
        // 原声分按带噪输入的内容哈希缓存（025 F1）：同一段输入只算一次，不同输入不会因任务 id 相同而串用
        let key = content_key(&clip.mix);
        if !self.mix_mos.contains_key(&key) {
            match self.dnsmos_sub(&format!("{}-mix", task.id), &clip.mix) {
                Ok(m) => {
                    self.mix_mos.insert(key.clone(), m.ovrl);
                }
                Err(e) => return Score::unknown(ID, "MOS", Better::Higher, format!("DNSMOS：{e}")),
            }
        }
        let mix = self.mix_mos.get(&key).copied();
        match self.dnsmos_sub(&format!("{}-out", task.id), out) {
            Ok(m) => Score::judged(ID, m.ovrl, "MOS", Better::Higher, mix, &[])
                .with_extra("sig", m.sig)
                .with_extra("bak", m.bak)
                .with_extra("ovrl_noisy", mix.unwrap_or(f64::NAN))
                .with_note("过线 = 不比带噪输入的 OVRL 差（L0 约束）"),
            Err(e) => Score::unknown(ID, "MOS", Better::Higher, format!("DNSMOS：{e}")),
        }
    }

    /// 在子进程里跑 DNSMOS（`vdn-eval dnsmos <模型> <wav>`，输出一行 JSON）。
    fn dnsmos_sub(&self, name: &str, x: &[f32]) -> Result<dnsmos::Mos, String> {
        let model = self.dnsmos.as_ref().ok_or("没有 DNSMOS 模型")?;
        let (wav_path, _) = self.dump(name, x)?;
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let o = std::process::Command::new(exe)
            .arg("dnsmos")
            .arg(model)
            .arg(&wav_path)
            .output()
            .map_err(|e| format!("起不来 DNSMOS 子进程：{e}"))?;
        if !o.status.success() {
            return Err(String::from_utf8_lossy(&o.stderr).trim().to_string());
        }
        let v: Value =
            serde_json::from_slice(&o.stdout).map_err(|e| format!("DNSMOS 子进程输出：{e}"))?;
        let f = |k: &str| v[k].as_f64().ok_or_else(|| format!("DNSMOS 缺 {k}"));
        Ok(dnsmos::Mos {
            sig: f("sig")?,
            bak: f("bak")?,
            ovrl: f("ovrl")?,
            windows: v["windows"].as_u64().unwrap_or(0) as usize,
        })
    }

    /// 集合层吃字：按片段配对 bootstrap（rubric V7，10000 次，种子 20261030）。
    pub fn eat_summary(&self) -> Value {
        let mut by: BTreeMap<&str, Vec<&EatRow>> = BTreeMap::new();
        for r in &self.eat {
            by.entry(r.system.as_str()).or_default().push(r);
        }
        let mut out = Vec::new();
        for (sys, rows) in by {
            let stat = |idx: &[usize], del_only: bool| -> f64 {
                let (mut c, mut m, mut l) = (0.0, 0.0, 0.0);
                for &i in idx {
                    let r = rows[i];
                    c += (r.out_del + if del_only { 0 } else { r.out_sub }) as f64;
                    m += (r.mix_del + if del_only { 0 } else { r.mix_sub }) as f64;
                    l += r.ref_len as f64;
                }
                (c - m) / l.max(1.0) * 100.0
            };
            let all: Vec<usize> = (0..rows.len()).collect();
            let mut rng = Rng::new(20_261_030);
            let mut bs_del = Vec::with_capacity(10_000);
            let mut bs_ds = Vec::with_capacity(10_000);
            for _ in 0..10_000 {
                let idx: Vec<usize> = (0..rows.len()).map(|_| rng.below(rows.len())).collect();
                bs_del.push(stat(&idx, true));
                bs_ds.push(stat(&idx, false));
            }
            let q = crate::analysis::quantile;
            let n = rows.len();
            let d_del = stat(&all, true);
            let lo = q(&bs_del, 0.025);
            let verdict = if n >= 30 {
                if d_del > 1.0 && lo > 0.0 {
                    "veto"
                } else {
                    "pass"
                }
            } else if d_del > 1.0 {
                "v7_suspect（片段 < 30，不否决）"
            } else {
                "pass（片段 < 30，只是点估计）"
            };
            out.push(json!({
                "system": sys, "clips": n,
                "d_del_pp": d_del, "d_del_ci95": [lo, q(&bs_del, 0.975)],
                "d_del_sub_pp": stat(&all, false), "d_del_sub_ci95": [q(&bs_ds, 0.025), q(&bs_ds, 0.975)],
                "rubric_v7": verdict,
            }));
        }
        json!(out)
    }
}

/// 样本内容的键：f32 逐位的 FNV-1a 64 位哈希 + 长度。
pub fn content_key(x: &[f32]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in x {
        for b in v.to_bits().to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    }
    format!("{h:016x}-{}", x.len())
}

impl Extra for QaExtra {
    fn score(
        &mut self,
        v: &str,
        sys: &str,
        task: &Task,
        clip: &Clip,
        out: &[f32],
        p: &Params,
    ) -> Option<Score> {
        match v {
            "V-EAT-ASR" => Some(self.eat(sys, task, clip, out, p)),
            "V-DNSMOS" => Some(self.mos(task, clip, out)),
            "V-NRMOS-SCOREQ" => Some(self.nrmos_score(nrmos::Kind::Scoreq, task, clip, out, p)),
            "V-NRMOS-DISTILL" => Some(self.nrmos_score(nrmos::Kind::Distill, task, clip, out, p)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::content_key;

    /// 同样的样本同一个键；差一个样本、差一位、差长度都换键（DNSMOS 原声分缓存，025 F1）。
    #[test]
    fn content_key_changes_with_content() {
        let a = vec![0.1f32, -0.2, 0.3];
        assert_eq!(content_key(&a), content_key(&a.clone()));
        let mut b = a.clone();
        b[1] = f32::from_bits(b[1].to_bits() ^ 1);
        assert_ne!(content_key(&a), content_key(&b));
        assert_ne!(content_key(&a), content_key(&a[..2]));
    }
}
