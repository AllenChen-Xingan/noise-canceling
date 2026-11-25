//! 识别费账本（codex 第 5 轮 P1 #1 · #2）：全部真调用共用一本，上限按它累计。
//!
//! - **跨进程加锁**：每次读改写都先锁旁边的 `<账本>.lock`（`std::fs::File::lock`，独占），几个进程同时跑也不会互相覆盖
//! - **原子落盘**：先写 `<账本>.tmp` 再改名；写失败返回 `Err`，调用方据此停发
//! - **账本坏了拒跑**：文件在但读不出、不是合法 JSON → `Err`，不当成从 0 开始
//! - **账本不存在也拒跑**（大总管 2026-11-25）：不再当空账本；只有显式新建（`--init-ledger` 或 `VDN_ASR_INIT_LEDGER=1`，
//!   入口 [`init`]）才有账本，新建时把来历记进 `created`（时间 · 本机或云上 · 机器 id · 哪个会话；**不记主机名**，账本快照进公开仓库）
//! - **封存**：账本旁边有 `<账本>.sealed`（本机那份合并进云上正本后放的说明）时，读、预留、新建一律拒，提示正本在哪
//! - **路径可配**：`VDN_ASR_LEDGER` 指向正本（云上是网络盘 `/workspace/asr-ledger/…`）；临时文件与锁都放在账本同一目录，
//!   改名在同一个文件系统里完成
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
    /// 这本账是怎么建的（[`init`] 写；旧账本没有）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<Created>,
    /// 手动更正记录（清 review 时由 exp025 baseline/clear_review.py 追加）：程序原样保留，不解读
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub adjustments: Vec<serde_json::Value>,
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

/// 新建来历。四个键，不含主机名。
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Created {
    /// UTC 时间，ISO 8601
    pub at: String,
    /// `local` 或 `cloud`
    pub place: String,
    /// 云上 = RunPod 机器 id（`RUNPOD_POD_ID`）；本机 = `local`（没有不带人名的固定编号，不另造）
    pub machine: String,
    /// 新建它的会话名（`VDN_SESSION`），没有为 `unknown`
    pub by: String,
}

impl Created {
    /// 按当前环境记来历。
    pub fn here() -> Created {
        let pod = std::env::var("RUNPOD_POD_ID")
            .ok()
            .filter(|s| !s.trim().is_empty());
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let by = std::env::var("VDN_SESSION")
            .ok()
            .filter(|s| !s.trim().is_empty());
        Created {
            at: utc_iso(secs),
            place: if pod.is_some() { "cloud" } else { "local" }.into(),
            machine: pod.unwrap_or_else(|| "local".into()),
            by: by.unwrap_or_else(|| "unknown".into()),
        }
    }

    /// 测试用的固定来历。
    pub fn test() -> Created {
        Created {
            at: utc_iso(0),
            place: "local".into(),
            machine: "local".into(),
            by: "test".into(),
        }
    }
}

/// Unix 秒 → `YYYY-MM-DDTHH:MM:SSZ`（UTC；公历换算照 Howard Hinnant 的 civil_from_days）。
pub fn utc_iso(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

/// 账本路径：`VDN_ASR_LEDGER` 非空就用它，否则 `<主目录>/` + [`super::asr_or::LEDGER_REL`]。
pub fn ledger_path(root: &Path) -> PathBuf {
    ledger_path_from(root, std::env::var("VDN_ASR_LEDGER").ok())
}

/// [`ledger_path`] 的纯函数部分（测试不用动环境变量）。
pub fn ledger_path_from(root: &Path, env: Option<String>) -> PathBuf {
    match env.filter(|s| !s.trim().is_empty()) {
        Some(p) => PathBuf::from(p),
        None => root.join(super::asr_or::LEDGER_REL),
    }
}

/// 错误信息里只写账本文件名（不写目录：本机目录含用户名，而错误会进成绩单 `meta.asr`，成绩单进公开仓库）。
fn fname(p: &Path) -> String {
    p.file_name()
        .map_or_else(|| "账本".into(), |n| n.to_string_lossy().into_owned())
}

/// 逐笔明细：`<账本>.calls.jsonl`，每次 HTTP 尝试一行，只追加（大总管 2026-11-25：以后逐笔核后台用量）。
pub fn details_path(p: &Path) -> PathBuf {
    let mut s = p.as_os_str().to_owned();
    s.push(".calls.jsonl");
    PathBuf::from(s)
}

/// 在账本锁里往明细追加一行（几个进程同时写也不会串行）。
pub fn append_detail(p: &Path, row: &serde_json::Value) -> Result<(), String> {
    use std::io::Write;
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(lock_path(p))
        .map_err(|e| format!("开不了账本锁：{e}"))?;
    lock.lock().map_err(|e| format!("锁不上账本：{e}"))?;
    let r = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(details_path(p))
        .and_then(|mut f| f.write_all(format!("{row}\n").as_bytes()))
        .map_err(|e| format!("写不了识别费明细 {}：{e}", fname(&details_path(p))));
    let _ = lock.unlock();
    r
}

/// 封存说明文件：`<账本>.sealed`。
pub fn sealed_path(p: &Path) -> PathBuf {
    let mut s = p.as_os_str().to_owned();
    s.push(".sealed");
    PathBuf::from(s)
}

fn check_sealed(p: &Path) -> Result<(), String> {
    // 不把封存说明的原文带出来（可能写了机器名、地址）；正本在哪写在说明文件里，人去看
    if sealed_path(p).exists() {
        return Err(format!(
            "识别费账本 {} 已封存：拒发。正本在云上，先取回再发（见旁边的 .sealed 说明）",
            fname(p)
        ));
    }
    Ok(())
}

/// 显式新建一本空账本（记 `created`）。已存在、或已封存，都报错，不覆盖。
pub fn init(p: &Path, created: Created) -> Result<(), String> {
    check_sealed(p)?;
    if let Some(d) = p.parent() {
        std::fs::create_dir_all(d).map_err(|e| format!("建不了账本目录：{e}"))?;
    }
    let l = Ledger {
        created: Some(created),
        ..Ledger::default()
    };
    let text = serde_json::to_string_pretty(&l).map_err(|e| e.to_string())?;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(p)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                format!("识别费账本 {} 已存在：不新建、不覆盖", fname(p))
            } else {
                format!("建不了识别费账本 {}：{e}", fname(p))
            }
        })?;
    use std::io::Write;
    f.write_all(text.as_bytes())
        .map_err(|e| format!("写不了识别费账本：{e}"))
}

/// 命令行要求新建时才建（`scorecard --init-ledger` = `flag`，或环境变量 `VDN_ASR_INIT_LEDGER` = `env` 为 `1`）。
/// 返回是否新建了；要求了但已存在或已封存，返回 `Err`（不覆盖）。
pub fn init_if_requested(p: &Path, flag: bool, env: Option<String>) -> Result<bool, String> {
    if !(flag || env.as_deref() == Some("1")) {
        return Ok(false);
    }
    init(p, Created::here())?;
    Ok(true)
}

/// 读账本（不加锁）：没有文件、已封存、读不出、格式坏了，都是 `Err`。
pub fn read(p: &Path) -> Result<Ledger, String> {
    check_sealed(p)?;
    match std::fs::read_to_string(p) {
        Ok(t) => serde_json::from_str(&t)
            .map_err(|e| format!("识别费账本 {} 坏了（{e}）：拒跑，先人工查", fname(p))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(format!(
            "识别费账本 {} 不存在：拒发（不当成空账本）。要新建请带 --init-ledger 或 VDN_ASR_INIT_LEDGER=1",
            fname(p)
        )),
        Err(e) => Err(format!("读不了识别费账本 {}：{e}", fname(p))),
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

    /// `scorecard --init-ledger` 或 `VDN_ASR_INIT_LEDGER=1` 才新建；别的值、没给都不建。
    #[test]
    fn init_only_when_explicitly_requested() {
        let p = tmp("req");
        assert!(!init_if_requested(&p, false, None).unwrap());
        assert!(!init_if_requested(&p, false, Some("0".into())).unwrap());
        assert!(!p.exists(), "没要求就不建");
        assert!(init_if_requested(&p, false, Some("1".into())).unwrap());
        assert!(read(&p).unwrap().created.is_some());
        let q = tmp("req2");
        assert!(init_if_requested(&q, true, None).unwrap());
        assert!(
            init_if_requested(&q, true, None)
                .unwrap_err()
                .contains("已存在"),
            "已有账本时要求新建 = 报错，不覆盖"
        );
    }

    /// 手动更正（清 review 的脚本写的 `adjustments`）在程序之后每次写账本时都要留着，不能被丢掉。
    #[test]
    fn manual_adjustments_survive_updates() {
        let p = tmp("adj");
        init(&p, Created::test()).unwrap();
        let mut v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        v["adjustments"] = serde_json::json!([{"kind": "review-cleared", "worst_usd": 0.000096}]);
        std::fs::write(&p, v.to_string()).unwrap();
        reserve(&p, 0.01, 1.0).unwrap();
        settle(&p, 0.01, Some(0.001)).unwrap();
        let back: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        assert_eq!(back["adjustments"][0]["kind"], "review-cleared", "{back}");
    }

    /// codex 第 1 轮（本分支）P1 #2 + 自审：错误会进 `meta.asr.error`（成绩单进公开仓库）。错误信息里只许出现账本文件名，
    /// 不许带目录（本机目录含用户名）、不许带封存说明的原文。
    #[test]
    fn errors_carry_no_directory_or_seal_note() {
        let p = tmp("leak");
        let dir = p.parent().unwrap().display().to_string();
        let msgs = vec![read(&p).unwrap_err(), reserve(&p, 0.01, 1.0).unwrap_err()];
        std::fs::write(
            sealed_path(&p),
            "迁去 secret-host-xyz.local · https://user:pw@example.com",
        )
        .unwrap();
        let msgs = [
            msgs,
            vec![
                read(&p).unwrap_err(),
                init(&p, Created::test()).unwrap_err(),
            ],
        ]
        .concat();
        std::fs::remove_file(sealed_path(&p)).unwrap();
        std::fs::write(&p, "{ bad").unwrap();
        let msgs = [msgs, vec![read(&p).unwrap_err()]].concat();
        for m in &msgs {
            assert!(!m.contains(&dir), "带了目录：{m}");
            assert!(
                !m.contains("secret-host") && !m.contains("user:pw"),
                "带了封存说明原文：{m}"
            );
            assert!(m.contains("ledger.json"), "要说清是哪个文件：{m}");
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
        init(&p, Created::test()).unwrap(); // 账本要显式新建（2026-11-25 起不再当空账本）
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
        init(&p, Created::test()).unwrap(); // 账本要显式新建（2026-11-25 起不再当空账本）
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
        init(&p, Created::test()).unwrap(); // 账本要显式新建（2026-11-25 起不再当空账本）
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
