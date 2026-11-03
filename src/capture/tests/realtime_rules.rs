//! 文本检查（规约 §3.4 / R2）：vdn-capture 的音频线程代码里没有锁。
//! 实时模块 = lib 的全部源文件（Forwarder · Feeder · shm 适配）；bin 的音频回调只调 lib 里的 `Feeder::feed` /
//! `HalSource::render`，bin 本身（进程编排）也一并查，免得有人把锁写进回调闭包。

use std::path::Path;

const BANNED: [&str; 7] = [
    "Mutex",
    "RwLock",
    "Condvar",
    "try_lock",
    ".lock(",
    "parking_lot",
    "std::sync::mpsc",
];

#[test]
fn no_locks_in_capture_sources() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut hits = Vec::new();
    let mut files = 0;
    for entry in std::fs::read_dir(&src).unwrap() {
        let p = entry.unwrap().path();
        if p.extension().is_some_and(|e| e == "rs") {
            files += 1;
            let text = std::fs::read_to_string(&p).unwrap();
            for (i, line) in text.lines().enumerate() {
                let code = line.split("//").next().unwrap_or("");
                for b in BANNED {
                    if code.contains(b) {
                        hits.push(format!("{}:{}：{b}", p.display(), i + 1));
                    }
                }
            }
        }
    }
    assert!(files >= 5, "只找到 {files} 个源文件，路径不对？");
    assert!(
        hits.is_empty(),
        "音频线程代码里不许有锁：\n{}",
        hits.join("\n")
    );
}
