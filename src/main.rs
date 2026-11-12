//! The repository's checker. `cargo run` checks the tree; `cargo run -- pr`
//! checks a pull request's title and body, read from `PR_TITLE` and
//! `PR_BODY`, and skips the body when `PR_AUTHOR` is Dependabot. Either
//! exits non-zero on any error.

mod pins;
mod pr;
mod privacy;
mod skills;
mod walk;

use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: check [pr]";

fn main() -> ExitCode {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let args: Vec<String> = env::args().skip(1).collect();
    let errors = match args.as_slice() {
        [] => check_tree(&root),
        [cmd] if cmd == "pr" => check_pr(),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };

    for error in &errors {
        eprintln!("{error}");
    }
    if errors.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn check_tree(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    skills::check(root, &mut errors);
    skills::check_links(root, &mut errors);
    privacy::check(root, &mut errors);
    pins::check(root, &mut errors);
    errors
}

fn check_pr() -> Vec<String> {
    let title = env::var("PR_TITLE").unwrap_or_default();
    let body = env::var("PR_BODY").unwrap_or_default();
    let author = env::var("PR_AUTHOR").unwrap_or_default();
    let mut errors = Vec::new();
    for e in pr::check_title(&title) {
        errors.push(format!("PR title: {e}"));
    }
    // Dependabot writes its own body; its title still lands on main.
    if author == pr::DEPENDABOT {
        return errors;
    }
    for e in pr::check_body(&body) {
        errors.push(format!("PR body: {e}"));
    }
    errors
}
