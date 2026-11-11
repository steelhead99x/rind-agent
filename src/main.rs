//! Checks every skill's frontmatter and every relative link in the repo's
//! Markdown. Run with `cargo run`; exits non-zero on any error.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut errors = Vec::new();

    check_skills(&root, &mut errors);
    for doc in markdown_files(&root) {
        check_links(&root, &doc, &mut errors);
    }

    for error in &errors {
        eprintln!("{error}");
    }
    if errors.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn check_skills(root: &Path, errors: &mut Vec<String>) {
    let mut dirs: Vec<PathBuf> = match fs::read_dir(root.join("skills")) {
        Ok(entries) => entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect(),
        Err(e) => {
            errors.push(format!("skills/: {e}"));
            return;
        }
    };
    dirs.sort();

    for dir in dirs {
        let skill = dir.join("SKILL.md");
        let shown = rel(root, &skill);
        let Ok(text) = fs::read_to_string(&skill) else {
            errors.push(format!("{shown}: missing"));
            continue;
        };
        let Some(front) = frontmatter(&text) else {
            errors.push(format!("{shown}: missing YAML frontmatter"));
            continue;
        };
        let dir_name = dir.file_name().unwrap_or_default().to_string_lossy();
        if field(front, "name") != Some(dir_name.as_ref()) {
            errors.push(format!("{shown}: name must be {dir_name:?}"));
        }
        if field(front, "description").map_or(true, |d| d.trim().is_empty()) {
            errors.push(format!("{shown}: description is empty"));
        }
    }
}

/// The text between a leading `---` line and the next `---` line.
fn frontmatter(text: &str) -> Option<&str> {
    let body = text.strip_prefix("---\n")?;
    let end = body.find("\n---\n")?;
    Some(&body[..end])
}

fn field<'a>(front: &'a str, key: &str) -> Option<&'a str> {
    front.lines().find_map(|line| {
        let (k, v) = line.split_once(": ")?;
        (k == key).then_some(v)
    })
}

fn markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for path in entries.flatten().map(|e| e.path()) {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name.starts_with('.') || name == "target" {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "md") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

fn check_links(root: &Path, doc: &Path, errors: &mut Vec<String>) {
    let Ok(text) = fs::read_to_string(doc) else {
        return;
    };
    let base = doc.parent().unwrap_or(root);
    for target in link_targets(&text) {
        if target.starts_with('#') || has_scheme(target) {
            continue;
        }
        let file = target.split('#').next().unwrap_or(target);
        if !base.join(file).exists() {
            errors.push(format!("{}: broken link {target}", rel(root, doc)));
        }
    }
}

/// Targets of inline links: the text inside `](...)`, up to whitespace or `)`.
fn link_targets(text: &str) -> impl Iterator<Item = &str> {
    text.match_indices("](").filter_map(move |(i, _)| {
        let rest = &text[i + 2..];
        let end = rest.find(|c: char| c == ')' || c.is_whitespace())?;
        (end > 0 && rest[end..].starts_with(')')).then(|| &rest[..end])
    })
}

/// True for `https:`, `mailto:` and other lowercase URL schemes.
fn has_scheme(target: &str) -> bool {
    target
        .split_once(':')
        .is_some_and(|(s, _)| !s.is_empty() && s.bytes().all(|b| b.is_ascii_lowercase()))
}

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_frontmatter() {
        let text = "---\nname: x\ndescription: Use when y.\n---\n# Body\n";
        let front = frontmatter(text).unwrap();
        assert_eq!(field(front, "name"), Some("x"));
        assert_eq!(field(front, "description"), Some("Use when y."));
        assert_eq!(frontmatter("# no frontmatter\n"), None);
    }

    #[test]
    fn finds_link_targets() {
        let text = "[a](b.md) [c](https://x.y) [d](e.md#f) [g](h i) `](`";
        let found: Vec<_> = link_targets(text).collect();
        assert_eq!(found, ["b.md", "https://x.y", "e.md#f"]);
        assert!(has_scheme("mailto:a@b"));
        assert!(!has_scheme("skills/a/SKILL.md"));
    }
}
