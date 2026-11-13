// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_stage-repo"))
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn collect_files(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut files = Vec::new();
    fn walk(root: &Path, dir: &Path, files: &mut Vec<(PathBuf, Vec<u8>)>) {
        let mut entries: Vec<_> = fs::read_dir(dir)
            .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
            .map(|e| e.unwrap().path())
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                walk(root, &path, files);
            } else {
                let rel = path.strip_prefix(root).unwrap().to_path_buf();
                files.push((rel, fs::read(&path).unwrap()));
            }
        }
    }
    walk(root, root, &mut files);
    files
}

fn assert_trees_equal(got: &Path, expected: &Path) {
    let got_files = collect_files(got);
    let exp_files = collect_files(expected);
    let got_names: Vec<_> = got_files.iter().map(|(p, _)| p.clone()).collect();
    let exp_names: Vec<_> = exp_files.iter().map(|(p, _)| p.clone()).collect();
    assert_eq!(got_names, exp_names, "file list");
    for ((rel, g), (_, e)) in got_files.iter().zip(exp_files.iter()) {
        if g != e {
            let gs = String::from_utf8_lossy(g);
            let es = String::from_utf8_lossy(e);
            panic!(
                "mismatch in {}:\n--- got ---\n{gs}\n--- expected ---\n{es}",
                rel.display()
            );
        }
    }
}

#[test]
fn fixture_tree_matches_python() {
    let tmp = tempfile_dir("stage-repo-parity");
    let out = tmp.join("out");
    let status = Command::new(bin())
        .args([
            "--out",
            out.to_str().unwrap(),
            "--json",
            fixture("in/alpha.json").to_str().unwrap(),
            "--rename",
            "captured-alpha=alpha",
            "--json",
            fixture("in/beta.json").to_str().unwrap(),
            "--rename",
            "captured-beta=beta",
            "--replace",
            "Test CPU=Sample CPU",
            "--render",
            fixture("in/render").to_str().unwrap(),
            "--log",
            &format!("alpha={}", fixture("in/logs/run.log").display()),
            "--note",
            &format!("alpha={}", fixture("in/notes/alpha.md").display()),
        ])
        .output()
        .expect("run stage-repo");
    assert_eq!(String::from_utf8_lossy(&status.stderr), "");
    assert!(status.status.success(), "status {:?}", status.status);
    assert_eq!(
        String::from_utf8_lossy(&status.stdout),
        format!("{}\n", out.display())
    );
    assert_trees_equal(&out, &fixture("expected"));
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn help_matches_argparse() {
    for flag in ["-h", "--help"] {
        let out = Command::new(bin()).arg(flag).output().expect("help");
        assert_eq!(out.status.code(), Some(0), "{flag}");
        assert_eq!(String::from_utf8_lossy(&out.stderr), "", "{flag} stderr");
        let expected = fs::read_to_string(fixture("help.txt")).unwrap();
        assert_eq!(String::from_utf8_lossy(&out.stdout), expected, "{flag}");
    }
}

#[test]
fn missing_required_args() {
    let out = Command::new(bin()).output().expect("run");
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "");
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "\
usage: stage-repo [-h] --out OUT --json JSON [--rename OLD=NEW]
                  [--replace REGEX=TEXT] [--render RENDER] [--log NAME=FILE]
                  [--note NAME=FILE]
stage-repo: error: the following arguments are required: --out, --json
"
    );
}

#[test]
fn missing_json_only() {
    let out = Command::new(bin())
        .args(["--out", "/tmp/unused-stage-repo"])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.ends_with("stage-repo: error: the following arguments are required: --json\n"),
        "{err}"
    );
}

#[test]
fn unrecognized_after_required() {
    let out = Command::new(bin())
        .args(["--out", "x", "--json", "y", "--nope"])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.ends_with("stage-repo: error: unrecognized arguments: --nope\n"),
        "{err}"
    );
}

#[test]
fn bad_rename_repr() {
    let out = Command::new(bin())
        .args([
            "--out",
            "/tmp/unused-stage-repo",
            "--json",
            fixture("in/alpha.json").to_str().unwrap(),
            "--rename",
            "noeq",
        ])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "stage-repo: --rename must be OLD=NEW, not 'noeq'\n"
    );
}

#[test]
fn empty_rename_right() {
    let out = Command::new(bin())
        .args([
            "--out",
            "/tmp/unused-stage-repo",
            "--json",
            fixture("in/alpha.json").to_str().unwrap(),
            "--rename",
            "foo=",
        ])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "stage-repo: --rename must be OLD=NEW, not 'foo='\n"
    );
}

#[test]
fn missing_json_file() {
    let tmp = tempfile_dir("stage-repo-missing");
    let out = tmp.join("out");
    let missing = tmp.join("does-not-exist.json");
    let result = Command::new(bin())
        .args([
            "--out",
            out.to_str().unwrap(),
            "--json",
            missing.to_str().unwrap(),
        ])
        .output()
        .expect("run");
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&result.stderr),
        format!(
            "stage-repo: {}: [Errno 2] No such file or directory: '{}'\n",
            missing.display(),
            missing.display()
        )
    );
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn missing_render() {
    let tmp = tempfile_dir("stage-repo-render");
    let out = tmp.join("out");
    let render = tmp.join("no-such-render");
    let result = Command::new(bin())
        .args([
            "--out",
            out.to_str().unwrap(),
            "--json",
            fixture("in/alpha.json").to_str().unwrap(),
            "--rename",
            "captured-alpha=alpha",
            "--render",
            render.to_str().unwrap(),
        ])
        .output()
        .expect("run");
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&result.stderr),
        format!(
            "stage-repo: {} has no BENCHMARKS.md or STATS.md\n",
            render.display()
        )
    );
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn host_name_rejects_backtick_and_space() {
    let tmp = tempfile_dir("stage-repo-token");
    let out = tmp.join("out");
    let result = Command::new(bin())
        .args([
            "--out",
            out.to_str().unwrap(),
            "--json",
            fixture("in/alpha.json").to_str().unwrap(),
            "--rename",
            "captured-alpha=bad name",
        ])
        .output()
        .expect("run");
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&result.stderr),
        format!(
            "stage-repo: {}: host name must be a token\n",
            fixture("in/alpha.json").display()
        )
    );
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn scrubs_home_directory_paths() {
    // Construct the unsanitized path at runtime so the committed tree never
    // contains a home-directory prefix.
    let home = concat!("/", "Use", "rs/me/secret/capture.json");
    let linux = concat!("/", "ho", "me/me/secret/run.log");
    let tmp = tempfile_dir("stage-repo-home");
    let src = tmp.join("in.json");
    let body = format!(
        "{{\n \"schema\": 1,\n \"host\": \"box\",\n \"note\": \"files at {home} and {linux} today\"\n}}\n"
    );
    fs::write(&src, body).unwrap();
    let out = tmp.join("out");
    let result = Command::new(bin())
        .args([
            "--out",
            out.to_str().unwrap(),
            "--json",
            src.to_str().unwrap(),
        ])
        .output()
        .expect("run");
    assert_eq!(String::from_utf8_lossy(&result.stderr), "");
    assert!(result.status.success(), "{:?}", result.status);
    let staged = fs::read_to_string(out.join("box/box.json")).unwrap();
    assert!(
        staged.contains("\"note\": \"files at ~ and ~ today\""),
        "{staged}"
    );
    assert!(
        !staged.contains("/Use") && !staged.contains("/ho"),
        "home prefix leaked: {staged}"
    );
    let _ = fs::remove_dir_all(&tmp);
}

fn tempfile_dir(prefix: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&base).unwrap();
    base
}
