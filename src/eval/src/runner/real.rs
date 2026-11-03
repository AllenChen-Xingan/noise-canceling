//! 读真实片段（QA 层）：核哈希、读 WAV、对齐干净参考。

use super::*;

/// 读一个真实片段（QA）。
pub fn load_real(src: &RealSource) -> Result<Clip, String> {
    let check = |p: &std::path::Path, want: Option<&str>| -> Result<(), String> {
        if let Some(w) = want {
            let got = wav::sha256(p)?;
            if got != w {
                return Err(format!(
                    "{} 的 sha256 对不上（清单 {w}，实际 {got}）",
                    p.display()
                ));
            }
        }
        Ok(())
    };
    let load = |p: &std::path::Path| -> Result<Vec<f32>, String> {
        let (x, sr) = wav::read(p)?;
        if sr != SR {
            return Err(format!("{} 采样率 {sr}，只支持 16 kHz", p.display()));
        }
        Ok(x)
    };
    match src {
        RealSource::Exp005 {
            clips,
            audio_dir,
            clip,
        } => {
            let cp = resolve(clips).ok_or_else(|| format!("找不到 {clips}（exp005 未合并？）"))?;
            let dir = resolve(audio_dir).ok_or_else(|| format!("找不到音频目录 {audio_dir}"))?;
            let j: Value =
                serde_json::from_str(&std::fs::read_to_string(&cp).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            let c = j["clips"]
                .as_array()
                .and_then(|a| a.iter().find(|c| c["id"] == clip.as_str()))
                .ok_or_else(|| format!("{clips} 里没有 {clip}"))?;
            let mut tr = BTreeMap::new();
            for k in ["mix", "owner", "interf", "noise"] {
                let f = &c["files"][k];
                let p = dir.join(f["file"].as_str().ok_or("缺文件名")?);
                check(&p, f["sha256"].as_str())?;
                tr.insert(k, load(&p)?);
            }
            let n = tr.values().map(Vec::len).min().unwrap_or(0);
            let take = |k: &str| tr.get(k).map(|v| v[..n].to_vec()).unwrap_or_default();
            let (mix, clean, interf, noise) =
                (take("mix"), take("owner"), take("interf"), take("noise"));
            Ok(Clip {
                sr: SR,
                truth: Truth {
                    owner: truth_from_clean(&clean),
                    interf: truth_from_clean(&interf),
                },
                clean,
                interf,
                noise,
                mix,
                tracks: true,
            })
        }
        RealSource::Exp003 { clips, clip } => {
            let cp = resolve(clips).ok_or_else(|| format!("找不到 {clips}"))?;
            let j: Value =
                serde_json::from_str(&std::fs::read_to_string(&cp).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            let c = j["clips"]
                .as_array()
                .and_then(|a| a.iter().find(|c| c["id"] == clip.as_str()))
                .ok_or_else(|| format!("{clips} 里没有 {clip}"))?;
            let get = |k: &str, h: &str| -> Result<Vec<f32>, String> {
                let rel = c[k]
                    .as_str()
                    .ok_or_else(|| format!("{clip} 缺 {k}（不是现成片段）"))?;
                let p = resolve(rel).ok_or_else(|| format!("找不到 {rel}"))?;
                check(&p, c[h].as_str())?;
                load(&p)
            };
            let mix = get("noisy_src", "noisy_sha256")?;
            let clean = get("clean_src", "clean_sha256")?;
            let n = mix.len().min(clean.len());
            let (mix, clean) = (mix[..n].to_vec(), clean[..n].to_vec());
            let noise: Vec<f32> = mix.iter().zip(&clean).map(|(a, b)| a - b).collect();
            Ok(Clip {
                sr: SR,
                truth: Truth {
                    owner: truth_from_clean(&clean),
                    interf: Vec::new(),
                },
                interf: vec![0.0; n],
                clean,
                noise,
                mix,
                tracks: false,
            })
        }
    }
}
