//! Pull request titles and bodies. PRs squash onto `main`, so the title
//! becomes the commit subject and must be a conventional commit.

use crate::skills::is_kebab;

/// Commit types this repository uses.
const TYPES: [&str; 8] = [
    "feat", "fix", "docs", "chore", "ci", "test", "refactor", "revert",
];

/// The longest title, so `git log --oneline` stays readable.
const MAX_TITLE: usize = 72;

/// The author whose PRs are exempt from the body sections.
pub const DEPENDABOT: &str = "dependabot[bot]";

/// Sections every PR body has, as `## <name>` headings.
pub const SECTIONS: [&str; 6] = [
    "Problem",
    "Approach",
    "Impact",
    "Testing",
    "Alternatives",
    "Follow-ups",
];

/// The problems with a PR title: `type(scope)!: subject`, scope and `!`
/// optional.
pub fn check_title(title: &str) -> Vec<String> {
    let mut problems = Vec::new();
    if title != title.trim() {
        problems.push("has leading or trailing whitespace".into());
    }
    if title.chars().count() > MAX_TITLE {
        problems.push(format!("is longer than {MAX_TITLE} characters"));
    }
    let Some((head, subject)) = title.trim().split_once(": ") else {
        problems.push("must look like `type(scope): subject`".into());
        return problems;
    };
    let head = head.strip_suffix('!').unwrap_or(head);
    let (kind, scope) = match head.split_once('(') {
        Some((kind, rest)) => (kind, Some(rest.strip_suffix(')').unwrap_or("("))),
        None => (head, None),
    };
    if !TYPES.contains(&kind) {
        problems.push(format!("type {kind:?} is not one of {}", TYPES.join(", ")));
    }
    if scope.is_some_and(|s| !is_kebab(s)) {
        problems.push("scope must be lowercase kebab-case in parentheses".into());
    }
    let subject = subject.trim();
    if subject.is_empty() {
        problems.push("subject is empty".into());
    } else if subject.ends_with('.') {
        problems.push("subject ends with a period".into());
    }
    problems
}

/// The problems with a PR body: each of [`SECTIONS`] as a `## ` heading,
/// with text that isn't only an HTML comment from the template.
pub fn check_body(body: &str) -> Vec<String> {
    let text = strip_comments(&body.replace("\r\n", "\n"));
    let sections = split_sections(&text);
    SECTIONS
        .iter()
        .filter_map(|&want| match sections.iter().find(|(h, _)| *h == want) {
            None => Some(format!("missing a `## {want}` section")),
            Some((_, text)) if text.trim().is_empty() => Some(format!("`## {want}` is empty")),
            Some(_) => None,
        })
        .collect()
}

/// `text` without `<!-- ... -->` comments.
fn strip_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("<!--") {
        out.push_str(&rest[..start]);
        rest = match rest[start..].find("-->") {
            Some(end) => &rest[start + end + 3..],
            None => "",
        };
    }
    out.push_str(rest);
    out
}

/// Each `## ` heading's name and the text up to the next `## ` heading.
fn split_sections(text: &str) -> Vec<(&str, String)> {
    let mut out: Vec<(&str, String)> = Vec::new();
    for line in text.lines() {
        if let Some(name) = line.strip_prefix("## ") {
            out.push((name.trim(), String::new()));
        } else if let Some((_, body)) = out.last_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_conventional_titles() {
        for title in [
            "docs(crypto-review): require a differential test",
            "feat(check): refuse unpinned actions",
            "fix: broken link in NOTES.md",
            "chore!: drop the old checker",
            "ci(deps): bump actions/checkout",
        ] {
            assert!(
                check_title(title).is_empty(),
                "{title}: {:?}",
                check_title(title)
            );
        }
    }

    #[test]
    fn rejects_bad_titles() {
        assert_eq!(
            check_title("Update README"),
            ["must look like `type(scope): subject`"]
        );
        assert_eq!(
            check_title("skills: new notes"),
            ["type \"skills\" is not one of feat, fix, docs, chore, ci, test, refactor, revert"]
        );
        assert_eq!(
            check_title("docs(Crypto Review): x"),
            ["scope must be lowercase kebab-case in parentheses"]
        );
        assert_eq!(
            check_title("docs(crypto-review: x"),
            ["scope must be lowercase kebab-case in parentheses"]
        );
        assert_eq!(
            check_title("docs: fix a link."),
            ["subject ends with a period"]
        );
        assert_eq!(
            check_title(" docs: x"),
            ["has leading or trailing whitespace"]
        );
        let long = format!("docs: {}", "x".repeat(MAX_TITLE));
        assert_eq!(
            check_title(&long),
            [format!("is longer than {MAX_TITLE} characters")]
        );
    }

    fn body(skip: &str) -> String {
        SECTIONS
            .iter()
            .filter(|s| **s != skip)
            .map(|s| format!("## {s}\n\nSomething.\n\n"))
            .collect()
    }

    #[test]
    fn accepts_a_full_body() {
        assert!(check_body(&body("")).is_empty());
        assert!(check_body(&body("").replace('\n', "\r\n")).is_empty());
    }

    #[test]
    fn rejects_missing_and_empty_sections() {
        assert_eq!(
            check_body(&body("Testing")),
            ["missing a `## Testing` section"]
        );
        let placeholder = body("").replace(
            "## Impact\n\nSomething.",
            "## Impact\n\n<!-- One bullet per change. -->",
        );
        assert_eq!(check_body(&placeholder), ["`## Impact` is empty"]);
        assert_eq!(check_body("").len(), SECTIONS.len());
    }
}
