// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::cli::Args;
use crate::jsonfmt::{dumps_compact, dumps_pretty};
use crate::notes::{notes_for, py_repr_str, trim_log};
use crate::scrub::Scrubber;

pub fn run(args: &Args) -> Result<PathBuf, String> {
    let mut renames = HashMap::new();
    for item in &args.rename {
        let (old, new) = split_pair(item, "stage-repo: --rename must be OLD=NEW, not ", true)?;
        renames.insert(old, new);
    }
    let mut replacements = Vec::new();
    for item in &args.replace {
        let (pattern, text) = split_pair(
            item,
            "stage-repo: --replace must be REGEX=TEXT, not ",
            false,
        )?;
        replacements.push((pattern, text));
    }
    let scrub = Scrubber::new(&replacements)?;

    let mut logs: HashMap<String, PathBuf> = HashMap::new();
    let mut notes: HashMap<String, PathBuf> = HashMap::new();
    for (flag, store) in [("--log", &mut logs), ("--note", &mut notes)] {
        let items = if flag == "--log" {
            &args.log
        } else {
            &args.note
        };
        for item in items {
            let (name, path) = split_pair(
                item,
                &format!("stage-repo: {flag} must be NAME=FILE, not "),
                true,
            )?;
            store.insert(name, PathBuf::from(path));
        }
    }

    let out = &args.out;
    for path in &args.json {
        let mut host = load_host(path, &scrub)?;
        let captured = match host.get("host") {
            Some(Value::String(s)) => s.clone(),
            _ => String::new(),
        };
        let name = renames.get(&captured).cloned().unwrap_or(captured);
        if !is_host_token(&name) {
            return Err(format!(
                "stage-repo: {}: host name must be a token",
                path.display()
            ));
        }
        if let Value::Object(map) = &mut host {
            map.insert("host".into(), Value::String(name.clone()));
        }
        let dest = out.join(&name);
        fs::create_dir_all(&dest).map_err(|e| io_err(&dest, e))?;
        let json_path = dest.join(format!("{name}.json"));
        let pretty = dumps_pretty(&host).map_err(|e| format!("stage-repo: {e}"))?;
        fs::write(&json_path, pretty).map_err(|e| io_err(&json_path, e))?;

        let mut extra = String::new();
        if let Some(note_path) = notes.get(&name) {
            extra = scrub.scrub(&read_text_replace(note_path)?);
            if !extra.is_empty() && !extra.ends_with('\n') {
                extra.push('\n');
            }
        }
        let notes_path = dest.join("NOTES.md");
        let body = format!("{}{extra}", scrub.scrub(&notes_for(&host)));
        fs::write(&notes_path, body).map_err(|e| io_err(&notes_path, e))?;

        if let Some(log_path) = logs.get(&name) {
            let raw = read_text_replace(log_path)?;
            let mut body = if log_path.extension().and_then(|e| e.to_str()) == Some("log") {
                trim_log(&raw)
            } else {
                raw
            };
            if !body.is_empty() && !body.ends_with('\n') {
                body.push('\n');
            }
            let dest_log = dest.join(file_name(log_path));
            fs::write(&dest_log, scrub.scrub(&body)).map_err(|e| io_err(&dest_log, e))?;
        }
    }

    if let Some(render) = &args.render {
        let mut sheet = None;
        for candidate in ["BENCHMARKS.md", "STATS.md"] {
            let path = render.join(candidate);
            if path.is_file() {
                sheet = Some(path);
                break;
            }
        }
        let Some(sheet) = sheet else {
            return Err(format!(
                "stage-repo: {} has no BENCHMARKS.md or STATS.md",
                render.display()
            ));
        };
        let readme = out.join("README.md");
        fs::create_dir_all(out).map_err(|e| io_err(out, e))?;
        let text = read_text(sheet.as_path())?;
        fs::write(&readme, scrub.scrub(&text)).map_err(|e| io_err(&readme, e))?;
        let benchmarks = render.join("benchmarks");
        if benchmarks.is_dir() {
            copy_tree_text(&benchmarks, &out.join("benchmarks"), &scrub)?;
        }
    }

    Ok(out.clone())
}

fn load_host(path: &Path, scrub: &Scrubber) -> Result<Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("stage-repo: {}: {}", path.display(), errno_message(path, e)))?;
    let host: Value =
        serde_json::from_str(&text).map_err(|e| format!("stage-repo: {}: {e}", path.display()))?;
    let compact =
        dumps_compact(&host).map_err(|e| format!("stage-repo: {}: {e}", path.display()))?;
    let scrubbed = scrub.scrub(&compact);
    serde_json::from_str(&scrubbed).map_err(|e| {
        format!(
            "stage-repo: scrubbing {} broke the JSON: {e}",
            path.display()
        )
    })
}

fn is_host_token(name: &str) -> bool {
    !name.is_empty() && !name.chars().any(|ch| ch.is_whitespace() || ch == '`')
}

fn split_pair(item: &str, prefix: &str, require_right: bool) -> Result<(String, String), String> {
    match item.split_once('=') {
        Some((left, right)) if !left.is_empty() && (!require_right || !right.is_empty()) => {
            Ok((left.to_string(), right.to_string()))
        }
        _ => Err(format!("{prefix}{}", py_repr_str(item))),
    }
}

fn read_text(path: &Path) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|e| format!("stage-repo: {}: {}", path.display(), errno_message(path, e)))
}

fn read_text_replace(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path)
        .map_err(|e| format!("stage-repo: {}: {}", path.display(), errno_message(path, e)))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn copy_tree_text(source: &Path, dest: &Path, scrub: &Scrubber) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| io_err(dest, e))?;
    copy_tree_inner(source, source, dest, scrub)
}

fn copy_tree_inner(
    root: &Path,
    source: &Path,
    dest: &Path,
    scrub: &Scrubber,
) -> Result<(), String> {
    for entry in fs::read_dir(source).map_err(|e| io_err(source, e))? {
        let entry = entry.map_err(|e| io_err(source, e))?;
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap();
        let target = dest.join(rel);
        if path.is_dir() {
            fs::create_dir_all(&target).map_err(|e| io_err(&target, e))?;
            copy_tree_inner(root, &path, dest, scrub)?;
        } else if path.is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|e| io_err(parent, e))?;
            }
            let text = read_text_replace(&path)?;
            fs::write(&target, scrub.scrub(&text)).map_err(|e| io_err(&target, e))?;
        }
    }
    Ok(())
}

fn file_name(path: &Path) -> PathBuf {
    PathBuf::from(path.file_name().unwrap_or_default())
}

fn io_err(path: &Path, err: std::io::Error) -> String {
    format!(
        "stage-repo: {}: {}",
        path.display(),
        errno_message(path, err)
    )
}

fn errno_message(path: &Path, err: std::io::Error) -> String {
    match err.kind() {
        std::io::ErrorKind::NotFound => {
            format!("[Errno 2] No such file or directory: '{}'", path.display())
        }
        std::io::ErrorKind::PermissionDenied => {
            format!("[Errno 13] Permission denied: '{}'", path.display())
        }
        _ => err.to_string(),
    }
}
