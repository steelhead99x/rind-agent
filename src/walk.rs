//! Walking the tree.

use std::fs;
use std::path::{Path, PathBuf};

/// Every file under `root` that `keep` accepts, sorted. Skips `.git` and
/// `target`, and every other dot-directory unless `dot_dirs` is set.
pub fn files(root: &Path, dot_dirs: bool, keep: impl Fn(&Path) -> bool) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for path in entries.flatten().map(|e| e.path()) {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if path.is_dir() {
                let skip =
                    name == ".git" || name == "target" || (!dot_dirs && name.starts_with('.'));
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
