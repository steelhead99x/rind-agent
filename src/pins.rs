//! Every third-party action a workflow uses is pinned to a full commit SHA
//! with its version in a trailing comment, so a moved tag can't change what
//! CI runs and Dependabot can keep the pin current. Local actions (`./`)
//! change only through this repository and are exempt.

use std::fs;
use std::path::Path;

use crate::walk::{files, rel};

pub fn check(root: &Path, errors: &mut Vec<String>) {
    let github = root.join(".github");
    let yaml = |p: &Path| p.extension().is_some_and(|e| e == "yml" || e == "yaml");
    let workflows = files(&github, true, |p| {
        yaml(p)
            && (p.parent().is_some_and(|d| d.ends_with("workflows"))
                || p.file_stem().is_some_and(|s| s == "action"))
    });
    for path in workflows {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for (n, line) in text.lines().enumerate() {
            if let Some(problem) = check_line(line) {
                errors.push(format!("{}:{}: {problem}", rel(root, &path), n + 1));
            }
        }
    }
}

/// The problem with a workflow line's `uses:`, if it has one.
fn check_line(line: &str) -> Option<String> {
    let line = line.trim_start();
    let line = line.strip_prefix("- ").unwrap_or(line).trim_start();
    let rest = line.strip_prefix("uses:")?;
    let (value, comment) = match rest.split_once(" #") {
        Some((v, c)) => (v, c.trim()),
        None => (rest, ""),
    };
    let value = value.trim().trim_matches(|c| c == '"' || c == '\'');
    if value.starts_with("./") {
        return None;
    }
    let pinned = value.rsplit_once('@').is_some_and(|(_, sha)| {
        sha.len() == 40 && sha.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
    });
    if !pinned {
        Some(format!("{value} is not pinned to a full commit SHA"))
    } else if comment.is_empty() {
        Some(format!("{value} has no trailing version comment"))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHA: &str = "3d3c42e5aac5ba805825da76410c181273ba90b1";

    #[test]
    fn accepts_pins_and_local_actions() {
        let pinned = format!("      - uses: actions/checkout@{SHA} # v7.0.1");
        assert_eq!(check_line(&pinned), None);
        assert_eq!(check_line("    uses: ./.github/actions/setup"), None);
        assert_eq!(check_line("      - run: cargo test"), None);
    }

    #[test]
    fn rejects_tags_branches_and_bare_shas() {
        let tag = check_line("      - uses: actions/checkout@v4").unwrap();
        assert!(tag.contains("not pinned"), "{tag}");
        let short = check_line("      - uses: actions/checkout@3d3c42e # v7").unwrap();
        assert!(short.contains("not pinned"), "{short}");
        let bare = check_line(&format!("      - uses: actions/checkout@{SHA}")).unwrap();
        assert!(bare.contains("no trailing version comment"), "{bare}");
    }
}
