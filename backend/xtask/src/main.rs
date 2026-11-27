//! Workspace automation, run as `cargo run -p xtask -- <command>`

use std::process::ExitCode;

use xtask::{bindings, layers};

const USAGE: &str = "usage: cargo run -p xtask -- <command>

commands:
  check-layers       enforce [workspace.metadata.layers] from the root Cargo.toml
  gen-ts [--check]   regenerate frontend/src/types/generated, or fail if it is stale";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args[..] {
        ["check-layers"] => check_layers(),
        ["gen-ts"] => gen_ts(false),
        ["gen-ts", "--check"] => gen_ts(true),
        ["-h" | "--help" | "help"] => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn check_layers() -> ExitCode {
    match layers::metadata(xtask::WORKSPACE_MANIFEST).and_then(|m| layers::check(&m)) {
        Ok(report) if report.violations.is_empty() => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Ok(report) => {
            eprintln!("{report}");
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("check-layers: {e}");
            ExitCode::from(2)
        }
    }
}

fn gen_ts(check: bool) -> ExitCode {
    match bindings::generate(check) {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("gen-ts: {e}");
            ExitCode::FAILURE
        }
    }
}
