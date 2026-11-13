//! Walking the tree.

use std::fs;
use std::path::{Path, PathBuf};

/// Every file under `root` that `keep` accepts, sorted. Skips `.git` (a
/// directory in a normal checkout, or a file in a worktree) and `target`,
/// and every other dot-directory unless `dot_dirs` is set.
pub fn files(root: &Path, dot_dirs: bool, keep: impl Fn(&Path) -> bool) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for path in entries.flatten().map(|e| e.path()) {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            // A worktree's `.git` is a file whose `gitdir:` line would
            // otherwise trip the privacy lint.
            if name == ".git" {
                continue;
            }
            if path.is_dir() {
                let skip = name == "target" || (!dot_dirs && name.starts_with('.'));
                if !skip {
                    stack.push(path);
                }
            } else if keep(&path) {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// `path` relative to `root`, for messages.
pub fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Scratch(PathBuf);

    impl Scratch {
        fn new() -> Self {
            static N: AtomicU64 = AtomicU64::new(0);
            let dir = std::env::temp_dir().join(format!(
                "rind-agent-walk-{}-{}",
                std::process::id(),
                N.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn skips_a_git_dir_and_a_git_file() {
        let checkout = Scratch::new();
        fs::create_dir(checkout.0.join(".git")).unwrap();
        fs::write(checkout.0.join(".git").join("config"), "placeholder\n").unwrap();
        fs::write(checkout.0.join("ok.txt"), "ok\n").unwrap();

        let worktree = Scratch::new();
        // Built with concat! so this file does not itself contain a home path.
        let gitdir = concat!("gitdir: /", "Users/someone/repo/.git/worktrees/x\n");
        fs::write(worktree.0.join(".git"), gitdir).unwrap();
        fs::write(worktree.0.join("ok.txt"), "ok\n").unwrap();

        assert_eq!(
            files(&checkout.0, true, |_| true),
            [checkout.0.join("ok.txt")]
        );
        assert_eq!(
            files(&worktree.0, true, |_| true),
            [worktree.0.join("ok.txt")]
        );
    }
}
