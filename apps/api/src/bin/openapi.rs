//! Writes the api OpenAPI document: `cargo run -p api --bin openapi -- --out <file>`.
//! With `--check`, fails instead when the file differs (used in CI).

use std::{fs, path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    let mut out: Option<PathBuf> = None;
    let mut check = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" => out = args.next().map(PathBuf::from),
            "--check" => check = true,
            other => {
                eprintln!("unknown argument {other:?}. Usage: openapi --out <file> [--check]");
                return ExitCode::from(2);
            }
        }
    }
    let Some(out) = out else {
        eprintln!("Usage: openapi --out <file> [--check]");
        return ExitCode::from(2);
    };

    let mut json = api::openapi()
        .to_pretty_json()
        .expect("OpenAPI document serializes");
    json.push('\n');

    if check {
        let current = fs::read_to_string(&out)
            .unwrap_or_default()
            .replace("\r\n", "\n");
        if current == json {
            return ExitCode::SUCCESS;
        }
        eprintln!(
            "{} is out of date. Run: cargo run -p api --bin openapi -- --out {}",
            out.display(),
            out.display()
        );
        return ExitCode::FAILURE;
    }

    if let Err(err) = fs::write(&out, json) {
        eprintln!("writing {}: {err}", out.display());
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
