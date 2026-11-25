//! 识别费账本（codex 第 5 轮 P1 #1 · #2）：全部真调用共用一本，上限按它累计。
//!
//! - **跨进程加锁**：每次读改写都先锁旁边的 `<账本>.lock`（`std::fs::File::lock`，独占），几个进程同时跑也不会互相覆盖
//! - **原子落盘**：先写 `<账本>.tmp` 再改名；写失败返回 `Err`，调用方据此停发
//! - **账本坏了拒跑**：文件在但读不出、不是合法 JSON → `Err`，不当成从 0 开始
//! - **先预留、后结算**：发请求前按估算预留（`spent + reserved + 这次估算 > 上限` 就不发）；返回后按 `usage.cost` 实际金额结算、
//!   退掉预留。进程中途崩了，预留会留在账本里（偏保守：只会少发，不会超支）；要清就人工把 `reserved_usd` 改回 0

use std::path::{Path, PathBuf};

/// 账本内容。
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Ledger {
    pub spent_usd: f64,
    pub calls: u64,
    /// 已经发出、还没结算的请求预留的金额
    #[serde(default)]
    pub reserved_usd: f64,
    /// 待核：有请求的费用不知道（连不上、超时、返回读不出或没带金额）。设了就拒发，重启也一样，
    /// 要人核对 OpenRouter 后台的实际花费、改正 `spent_usd` 与 `reserved_usd`、再删掉这一项（codex 第 7 轮 P1 #2）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review: Option<String>,
}

fn lock_path(p: &Path) -> PathBuf {
    let mut s = p.as_os_str().to_owned();
    s.push(".lock");
    PathBuf::from(s)
}

/// 读账本（不加锁）：没有文件 = 空账本；读不出或格式坏了 = `Err`。
pub fn read(p: &Path) -> Result<Ledger, String> {
    match std::fs::read_to_string(p) {
        Ok(t) => serde_json::from_str(&t)
            .map_err(|e| format!("识别费账本 {} 坏了（{e}）：拒跑，先人工查", p.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Ledger::default()),
        Err(e) => Err(format!("读不了识别费账本 {}：{e}", p.display())),
    }
}

/// 在锁里读改写一次：`f` 改账本并返回结果；`f` 返回 `Err` 时不写盘。写盘失败返回 `Err`。
pub fn update<T>(p: &Path, f: impl FnOnce(&mut Ledger) -> Result<T, String>) -> Result<T, String> {
    if let Some(d) = p.parent() {
        std::fs::create_dir_all(d).map_err(|e| format!("建不了账本目录：{e}"))?;
    }
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(lock_path(p))
        .map_err(|e| format!("开不了账本锁：{e}"))?;
    lock.lock().map_err(|e| format!("锁不上账本：{e}"))?;
    let r = (|| {
        let mut l = read(p)?;
        let out = f(&mut l)?;
        let mut tmp = p.as_os_str().to_owned();
        tmp.push(".tmp");
        let tmp = PathBuf::from(tmp);
        let text = serde_json::to_string_pretty(&l).map_err(|e| e.to_string())?;
        std::fs::write(&tmp, text).map_err(|e| format!("写不了识别费账本：{e}（停发）"))?;
        std::fs::rename(&tmp, p).map_err(|e| format!("写不了识别费账本：{e}（停发）"))?;
        Ok(out)
    })();
    let _ = lock.unlock();
    r
}

/// 预留 `est` 美元：放得下（`spent + reserved + est ≤ budget`）就记上并返回 `Ok`，放不下返回 `Err`（不发）。
pub fn reserve(p: &Path, est: f64, budget: f64) -> Result<(), String> {
    update(p, |l| {
        if let Some(r) = &l.review {
            return Err(format!(
                "识别费账本待核（{r}）：停发，等人核对后清掉 review"
            ));
        }
        if l.spent_usd + l.reserved_usd + est > budget {
            return Err(format!(
                "识别费已花 {:.6} 美元、在途预留 {:.6} 美元，再发这一条（估 {est:.6}）会超过上限 {budget} 美元：停发",
                l.spent_usd, l.reserved_usd
            ));
        }
        l.reserved_usd += est;
        Ok(())
    })
}

/// 标「待核」：费用不知道的那条，预留不退，并让之后的预留一律拒绝。
pub fn flag_review(p: &Path, why: &str) -> Result<(), String> {
    update(p, |l| {
        if l.review.is_none() {
            l.review = Some(why.to_string());
        }
        Ok(())
    })
}

/// 结算：退掉预留 `est`；成功的请求（`actual = Some(实际金额)`）记进花费与次数。
pub fn settle(p: &Path, est: f64, actual: Option<f64>) -> Result<(), String> {
    update(p, |l| {
        l.reserved_usd = (l.reserved_usd - est).max(0.0);
        if l.reserved_usd < 1e-12 {
            l.reserved_usd = 0.0;
        }
        if let Some(c) = actual {
            l.spent_usd += c;
            l.calls += 1;
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 大总管 2026-11-25：账本文件不存在时拒发，不再当成空账本；不会顺手建出文件。
    #[test]
    fn missing_ledger_refuses_and_is_not_created() {
        let p = tmp("missing");
        let e = reserve(&p, 0.01, 1.0).unwrap_err();
        assert!(e.contains("不存在") && e.contains("--init-ledger"), "{e}");
        assert!(!p.exists(), "拒发时不建文件");
    }

    /// 有 `<账本>.sealed` 时，读、预留、新建都拒，提示正本在哪。
    #[test]
    fn sealed_marker_refuses_everything() {
        let p = tmp("sealed");
        init(&p, Created::test()).unwrap();
        std::fs::write(sealed_path(&p), "本机账本已封存；正本在云上\n").unwrap();
        assert!(reserve(&p, 0.01, 1.0).unwrap_err().contains("封存"));
        assert!(read(&p).unwrap_err().contains("封存"));
        let q = tmp("sealed2");
        std::fs::write(sealed_path(&q), "x").unwrap();
        assert!(
            init(&q, Created::test()).unwrap_err().contains("封存"),
            "封存了也不许另起一本"
        );
    }

    /// 只有显式新建才有账本；新建记 created（四个键），再新建报错，不覆盖。
    #[test]
    fn init_creates_once_and_records_origin() {
        let p = tmp("init");
        init(&p, Created::test()).unwrap();
        let l = read(&p).unwrap();
        assert_eq!((l.spent_usd, l.calls), (0.0, 0));
        let c = l.created.clone().expect("新建要记 created");
        assert_eq!(c.place, "local");
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        let mut keys: Vec<&str> = v["created"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(keys, ["at", "by", "machine", "place"]);
        assert!(init(&p, Created::test()).unwrap_err().contains("已存在"));
        reserve(&p, 0.01, 1.0).unwrap();
    }

    /// 账本快照会进公开仓库：账本里不能出现本机主机名（大总管 2026-11-25）。
    #[test]
    fn ledger_never_contains_hostname() {
        let p = tmp("host");
        init(&p, Created::here()).unwrap();
        reserve(&p, 0.01, 1.0).unwrap();
        settle(&p, 0.01, Some(0.001)).unwrap();
        let text = std::fs::read_to_string(&p).unwrap();
        let host = std::process::Command::new("hostname")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        for h in [host.as_str(), host.split('.').next().unwrap_or("")] {
            if h.len() >= 3 {
                assert!(!text.contains(h), "账本里出现了主机名：{text}");
            }
        }
    }

    /// 时间写成 UTC 的 ISO 8601（自写换算，核几个已知点）。
    #[test]
    fn utc_formatting() {
        assert_eq!(utc_iso(0), "1970-01-01T00:00:00Z");
        assert_eq!(utc_iso(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(utc_iso(1_793_000_000), "2026-10-26T07:33:20Z");
    }

    /// 账本路径：给了 `VDN_ASR_LEDGER` 就用它（读写都落在那个文件上），没给用主目录下的默认位置。
    #[test]
    fn ledger_path_override() {
        let root = Path::new("/r");
        assert_eq!(
            ledger_path_from(root, None),
            root.join(super::super::asr_or::LEDGER_REL)
        );
        assert_eq!(
            ledger_path_from(root, Some("/w/l.json".into())),
            PathBuf::from("/w/l.json")
        );
        assert_eq!(
            ledger_path_from(root, Some(String::new())),
            root.join(super::super::asr_or::LEDGER_REL)
        );
        let p = tmp("override");
        init(&p, Created::test()).unwrap();
        reserve(&p, 0.01, 1.0).unwrap();
        settle(&p, 0.01, Some(0.002)).unwrap();
        assert_eq!(read(&p).unwrap().calls, 1, "读写都落在指定的文件上");
    }

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("vdn-asr-ledger-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d.join("ledger.json")
    }

    #[test]
    fn reserve_settle_round_trip() {
        let p = tmp("rt");
        reserve(&p, 0.2, 1.0).unwrap();
        assert_eq!(read(&p).unwrap().reserved_usd, 0.2);
        settle(&p, 0.2, Some(0.15)).unwrap();
        let l = read(&p).unwrap();
        assert_eq!((l.spent_usd, l.calls, l.reserved_usd), (0.15, 1, 0.0));
        // 失败的请求只退预留，不记账
        reserve(&p, 0.2, 1.0).unwrap();
        settle(&p, 0.2, None).unwrap();
        assert_eq!(read(&p).unwrap().calls, 1);
    }

    #[test]
    fn reservation_counts_toward_budget() {
        let p = tmp("budget");
        update(&p, |l| {
            l.spent_usd = 0.9;
            Ok(())
        })
        .unwrap();
        reserve(&p, 0.05, 1.0).unwrap();
        reserve(&p, 0.05, 1.0).unwrap();
        assert!(
            reserve(&p, 0.01, 1.0).is_err(),
            "已花 + 在途预留 = 1.0，再预留就超"
        );
    }

    #[test]
    fn corrupt_ledger_refuses_and_is_not_overwritten() {
        let p = tmp("bad");
        std::fs::write(&p, "{ not json").unwrap();
        assert!(read(&p).is_err());
        assert!(reserve(&p, 0.01, 1.0).is_err());
        assert_eq!(
            std::fs::read_to_string(&p).unwrap(),
            "{ not json",
            "坏账本原样留着，等人查"
        );
    }

    #[test]
    fn concurrent_writers_do_not_lose_updates() {
        let p = tmp("conc");
        std::thread::scope(|s| {
            for _ in 0..8 {
                s.spawn(|| {
                    for _ in 0..25 {
                        reserve(&p, 0.001, 100.0).unwrap();
                        settle(&p, 0.001, Some(0.001)).unwrap();
                    }
                });
            }
        });
        let l = read(&p).unwrap();
        assert_eq!(l.calls, 200, "200 次都记上了");
        assert!((l.spent_usd - 0.2).abs() < 1e-9);
        assert_eq!(l.reserved_usd, 0.0);
    }
}
