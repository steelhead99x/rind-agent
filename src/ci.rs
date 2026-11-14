//! The local gate, `cargo xtask ci`.
//!
//! It runs the checks the thin `Check` workflow runs: format, clippy,
//! tests, an MSRV build, and the tree checker. The cargo alias builds this
//! binary into `target/xtask`, so the nested `cargo` processes below can
//! use `target/` without taking the lock that the parent `cargo run` holds
//! until this process exits.

use std::path::Path;
use std::process::{Command, ExitCode};

use crate::check_tree;

/// `rust-version` from `Cargo.toml`, via Cargo. The MSRV step checks with
/// this toolchain.
const MSRV: &str = env!("CARGO_PKG_RUST_VERSION");

/// Stable-toolchain cargo arguments, in the order the old `Check` job ran
/// them.
const STABLE_STEPS: &[&[&str]] = &[
    &["fmt", "--all", "--check"],
    &[
        "clippy",
        "--all-targets",
        "--locked",
        "--",
        "--deny",
        "warnings",
    ],
    &["test", "--locked"],
];

/// `cargo check` arguments for the MSRV toolchain. A separate target
/// directory so that compiler does not replace the stable build in `target/`.
const MSRV_ARGS: &[&str] = &[
    "check",
    "--all-targets",
    "--locked",
    "--target-dir",
    "target/msrv",
];

enum Fail {
    Exit(u8),
    Msg(String),
}

pub fn run(root: &Path) -> ExitCode {
    if let Err(fail) = gate(root) {
        return fail.into();
    }
    let errors = check_tree(root);
    for error in &errors {
        eprintln!("{error}");
    }
    if errors.is_empty() {
        eprintln!("ci: ok");
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn gate(root: &Path) -> Result<(), Fail> {
    install(root, "stable", Some("rustfmt,clippy"))?;
    install(root, MSRV, None)?;
    for args in STABLE_STEPS {
        cargo(root, "stable", args)?;
    }
    cargo(root, MSRV, MSRV_ARGS)?;
    Ok(())
}

fn install(dir: &Path, toolchain: &str, components: Option<&str>) -> Result<(), Fail> {
    let mut args = vec![
        "toolchain".to_string(),
        "install".to_string(),
        toolchain.to_string(),
        "--profile".to_string(),
        "minimal".to_string(),
    ];
    if let Some(components) = components {
        args.push("--component".to_string());
        args.push(components.to_string());
    }
    execute(dir, "rustup", &args)
}

fn cargo(dir: &Path, toolchain: &str, args: &[&str]) -> Result<(), Fail> {
    let mut full = Vec::with_capacity(args.len() + 1);
    full.push(format!("+{toolchain}"));
    full.extend(args.iter().map(|arg| (*arg).to_string()));
    execute(dir, "cargo", &full)
}

fn execute(dir: &Path, program: &str, args: &[String]) -> Result<(), Fail> {
    eprintln!("$ {program} {}", args.join(" "));
    // Drop `CARGO_TARGET_DIR`: `cargo xtask` sets it to `target/xtask`, and
    // a nested cargo that inherited it would wait on this process's lock.
    let status = Command::new(program)
        .args(args)
        .current_dir(dir)
        .env_remove("CARGO_TARGET_DIR")
        .status()
        .map_err(|err| Fail::Msg(format!("could not run {program}: {err}")))?;
    if status.success() {
        Ok(())
    } else {
        let code = status
            .code()
            .and_then(|code| u8::try_from(code).ok())
            .unwrap_or(1);
        Err(Fail::Exit(code))
    }
}

impl From<Fail> for ExitCode {
    fn from(fail: Fail) -> Self {
        match fail {
            Fail::Exit(code) => ExitCode::from(code),
            Fail::Msg(msg) => {
                eprintln!("ci: {msg}");
                ExitCode::FAILURE
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_steps_are_the_old_check_job() {
        assert_eq!(STABLE_STEPS[0], ["fmt", "--all", "--check"]);
        assert_eq!(
            STABLE_STEPS[1],
            [
                "clippy",
                "--all-targets",
                "--locked",
                "--",
                "--deny",
                "warnings"
            ]
        );
        assert_eq!(STABLE_STEPS[2], ["test", "--locked"]);
    }

    #[test]
    fn msrv_comes_from_cargo_toml() {
        let manifest = include_str!("../Cargo.toml");
        let line = manifest
            .lines()
            .find(|line| line.starts_with("rust-version"))
            .expect("rust-version");
        assert_eq!(line, format!("rust-version = \"{MSRV}\""));
        assert_eq!(
            MSRV_ARGS,
            [
                "check",
                "--all-targets",
                "--locked",
                "--target-dir",
                "target/msrv"
            ]
        );
    }

    #[test]
    fn check_workflow_runs_only_xtask() {
        let text = include_str!("../.github/workflows/check.yml");
        let runs: Vec<_> = text
            .lines()
            .filter(|line| line.trim_start().starts_with("run:"))
            .collect();
        assert_eq!(runs, ["        run: cargo xtask ci"]);
    }
}
