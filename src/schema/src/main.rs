//! `cargo run -p vdn-schema`：重新导出 `specs/schema/` 下的全部 schema（只写有变化的文件）。
//! `cargo run -p vdn-schema -- --check`：只比对，不一致退出码 1（测试里跑的是同一个函数）。

#![forbid(unsafe_code)]

use std::process::ExitCode;

fn main() -> ExitCode {
    let root = vdn_schema::root();
    if std::env::args().skip(1).any(|a| a == "--check") {
        let p = vdn_schema::check(&root);
        if p.is_empty() {
            println!("schema：与 Rust 正本一致");
            return ExitCode::SUCCESS;
        }
        eprintln!("schema：{} 处不一致", p.len());
        for x in &p {
            eprintln!("  - {x}");
        }
        return ExitCode::FAILURE;
    }
    match vdn_schema::write(&root) {
        Ok(w) if w.is_empty() => {
            println!("schema：没有变化");
            ExitCode::SUCCESS
        }
        Ok(w) => {
            for f in &w {
                println!("已写 {}/{f}", vdn_schema::DIR);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("schema：没写——{e}");
            ExitCode::FAILURE
        }
    }
}
