//! Skill frontmatter, the README's skill table, and relative Markdown links.

use std::fs;
use std::path::{Path, PathBuf};

use crate::walk::{files, rel};

/// Checks each `skills/<name>/SKILL.md`: frontmatter with a `name` equal to
/// the directory and a `description` saying when to use it, and a row in
/// the README's skill table.
pub fn check(root: &Path, errors: &mut Vec<String>) {
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
    let readme = fs::read_to_string(root.join("README.md")).unwrap_or_default();

    for dir in dirs {
        let dir_name = dir.file_name().unwrap_or_default().to_string_lossy();
        let skill = dir.join("SKILL.md");
        let shown = rel(root, &skill);
        match fs::read_to_string(&skill) {
            Ok(text) => {
                for e in check_skill(&dir_name, &text) {
                    errors.push(format!("{shown}: {e}"));
                }
            }
            Err(_) => errors.push(format!("{shown}: missing")),
        }
        if !readme.contains(&format!("](skills/{dir_name}/SKILL.md)")) {
            errors.push(format!("README.md: skill table has no row for {dir_name}"));
        }
    }
}

/// The problems with one skill file whose directory is `dir_name`.
fn check_skill(dir_name: &str, text: &str) -> Vec<String> {
    let Some(front) = frontmatter(text) else {
        return vec!["missing YAML frontmatter".into()];
    };
    let mut problems = Vec::new();
    if !is_kebab(dir_name) {
        problems.push(format!(
            "directory {dir_name:?} is not lowercase kebab-case"
        ));
    }
    if field(front, "name") != Some(dir_name) {
        problems.push(format!("name must be {dir_name:?}"));
    }
    match field(front, "description").map(str::trim) {
        None | Some("") => problems.push("description is empty".into()),
        Some(d) if !d.starts_with("Use when") => {
            problems.push("description must start with \"Use when\"".into());
        }
        Some(_) => {}
    }
    problems
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

/// Lowercase ASCII words joined by single hyphens.
pub fn is_kebab(s: &str) -> bool {
    !s.is_empty()
        && s.split('-').all(|w| {
            !w.is_empty()
                && w.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

/// Checks that every relative link in the repo's Markdown resolves.
pub fn check_links(root: &Path, errors: &mut Vec<String>) {
    let docs = files(root, false, |p| p.extension().is_some_and(|e| e == "md"));
    for doc in docs {
        let Ok(text) = fs::read_to_string(&doc) else {
            continue;
        };
        let base = doc.parent().unwrap_or(root);
        for target in link_targets(&text) {
            if target.starts_with('#') || has_scheme(target) {
                continue;
            }
            let file = target.split('#').next().unwrap_or(target);
            if !base.join(file).exists() {
                errors.push(format!("{}: broken link {target}", rel(root, &doc)));
            }
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
        assert_eq!(frontmatter("---\nname: x\n"), None);
    }

    #[test]
    fn accepts_a_good_skill() {
        let text = "---\nname: rind-x\ndescription: Use when y.\n---\n";
        assert!(check_skill("rind-x", text).is_empty());
    }

    #[test]
    fn rejects_bad_skills() {
        let wrong_name = "---\nname: other\ndescription: Use when y.\n---\n";
        assert_eq!(
            check_skill("rind-x", wrong_name),
            ["name must be \"rind-x\""]
        );

        let no_description = "---\nname: rind-x\n---\n";
        assert_eq!(
            check_skill("rind-x", no_description),
            ["description is empty"]
        );

        let vague = "---\nname: rind-x\ndescription: Helps.\n---\n";
        assert_eq!(
            check_skill("rind-x", vague),
            ["description must start with \"Use when\""]
        );

        assert_eq!(check_skill("rind-x", "# x\n"), ["missing YAML frontmatter"]);
        let upper = "---\nname: Rind\ndescription: Use when y.\n---\n";
        assert_eq!(check_skill("Rind", upper).len(), 1);
    }

    #[test]
    fn kebab_case() {
        assert!(is_kebab("rind-crypto-review"));
        assert!(is_kebab("check2"));
        assert!(!is_kebab(""));
        assert!(!is_kebab("Rind"));
        assert!(!is_kebab("a--b"));
        assert!(!is_kebab("-a"));
        assert!(!is_kebab("a_b"));
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
