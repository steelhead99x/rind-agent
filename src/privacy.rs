//! Refuses details of a contributor's own machine or accounts: home
//! directory paths, UUIDs (machine, device, and session IDs look like
//! these), and credential-shaped strings.
//!
//! The patterns are assembled with `concat!` so this file does not match
//! itself.

use std::fs;
use std::path::Path;

use crate::walk::{files, rel};

/// Path prefixes that only appear in a path on someone's own machine.
const HOME_PATHS: [&str; 3] = [
    concat!("/", "Users/"),
    concat!("/", "home/"),
    concat!("C:", "\\", "Users", "\\"),
];

/// Prefixes of GitHub tokens; a match needs 20 more token characters.
const TOKEN_PREFIXES: [&str; 5] = [
    concat!("gh", "p_"),
    concat!("gh", "o_"),
    concat!("gh", "s_"),
    concat!("gh", "u_"),
    concat!("github", "_pat_"),
];

const PRIVATE_KEY: &str = concat!("PRIVATE ", "KEY-----");

/// Checks every UTF-8 file in the tree, dot-directories included.
pub fn check(root: &Path, errors: &mut Vec<String>) {
    for path in files(root, true, |_| true) {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for (n, line) in text.lines().enumerate() {
            for found in scan_line(line) {
                errors.push(format!("{}:{}: {found}", rel(root, &path), n + 1));
            }
        }
    }
}

/// What `line` leaks, described without repeating the match.
fn scan_line(line: &str) -> Vec<&'static str> {
    let mut found = Vec::new();
    if HOME_PATHS.iter().any(|p| line.contains(p)) {
        found.push("home directory path");
    }
    if has_uuid(line) {
        found.push("UUID (machine, device, or session ID)");
    }
    if TOKEN_PREFIXES.iter().any(|p| has_token(line, p)) {
        found.push("access token");
    }
    if line.contains(PRIVATE_KEY) {
        found.push("private key");
    }
    found
}

/// True if `line` has 8-4-4-4-12 hex digits not inside a longer word.
fn has_uuid(line: &str) -> bool {
    const GROUPS: [usize; 5] = [8, 4, 4, 4, 12];
    const LEN: usize = 36;
    let b = line.as_bytes();
    let word = |c: u8| c.is_ascii_alphanumeric() || c == b'-' || c == b'_';
    (0..b.len().saturating_sub(LEN - 1)).any(|start| {
        if start > 0 && word(b[start - 1]) {
            return false;
        }
        if b.get(start + LEN).is_some_and(|&c| word(c)) {
            return false;
        }
        let mut i = start;
        GROUPS.iter().enumerate().all(|(g, &len)| {
            let ok = b[i..i + len].iter().all(u8::is_ascii_hexdigit)
                && (g == GROUPS.len() - 1 || b[i + len] == b'-');
            i += len + 1;
            ok
        })
    })
}

/// True if `prefix` is followed by at least 20 token characters.
fn has_token(line: &str, prefix: &str) -> bool {
    line.match_indices(prefix).any(|(i, _)| {
        line[i + prefix.len()..]
            .bytes()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == b'_')
            .count()
            >= 20
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_home_paths() {
        let mac = concat!("see /", "Users/someone/rind");
        let linux = concat!("cd /", "home/someone");
        let windows = concat!("C:", "\\", "Users", "\\", "someone");
        for line in [mac, linux, windows] {
            assert_eq!(scan_line(line), ["home directory path"], "{line}");
        }
        assert!(scan_line("cp -r skills/* ~/.claude/skills/").is_empty());
    }

    #[test]
    fn finds_uuids() {
        let id = concat!("machine 0123abcd-", "4567-89ab-cdef-0123456789AB here");
        assert_eq!(scan_line(id), ["UUID (machine, device, or session ID)"]);
        // A commit SHA is not a UUID.
        assert!(!has_uuid("3c0ea6e3e56907b6cddd502fe90326855c570052"));
        // Inside a longer hyphenated run.
        assert!(!has_uuid(concat!(
            "x-0123abcd-",
            "4567-89ab-cdef-0123456789ab"
        )));
        assert!(!has_uuid(concat!(
            "0123abcd-",
            "4567-89ab-cdef-0123456789abc"
        )));
        assert!(!has_uuid(concat!(
            "0123abcd-",
            "4567-89ab-cdef-0123456789a"
        )));
        assert!(!has_uuid(concat!(
            "0123abcg-",
            "4567-89ab-cdef-0123456789ab"
        )));
    }

    #[test]
    fn finds_credentials() {
        let token = concat!("token=gh", "p_", "aaaaaaaaaaaaaaaaaaaaaaaaa");
        assert_eq!(scan_line(token), ["access token"]);
        assert!(scan_line(concat!("gh", "p_short")).is_empty());
        let key = concat!("-----BEGIN RSA PRIVATE ", "KEY-----");
        assert_eq!(scan_line(key), ["private key"]);
    }

    #[test]
    fn short_and_plain_lines_pass() {
        assert!(scan_line("").is_empty());
        assert!(scan_line("Run `cargo run` before opening a PR.").is_empty());
    }
}
