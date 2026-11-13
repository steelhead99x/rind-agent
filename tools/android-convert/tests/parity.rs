// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_android-convert"))
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn run_ok(out: &Path) -> std::process::Output {
    Command::new(bin())
        .arg("--rind")
        .arg(fixture("rind-stub"))
        .arg("--capture")
        .arg(fixture("capture"))
        .arg("--out")
        .arg(out)
        .arg("--host")
        .arg("pixel9pro")
        .arg("--commit")
        .arg("abcdef1234567890abcdef1234567890abcdef12")
        .arg("--rustc")
        .arg("rustc 1.98.1 (48a229cea 2026-09-01)")
        .arg("--cargo")
        .arg("cargo 1.98.1 (797e8a9bc 2026-08-05)")
        .arg("--target")
        .arg("aarch64-linux-android (API 31), Android NDK r28c, cross-compiled on arm64")
        .arg("--cpu-model")
        .arg("Google Tensor G4 (Pixel 9 Pro)")
        .arg("--logical-cpus")
        .arg("8")
        .output()
        .expect("run android-convert")
}

#[test]
fn fixture_matches_python_byte_for_byte() {
    let tmp = std::env::temp_dir().join(format!(
        "android-convert-parity-{}.json",
        std::process::id()
    ));
    let out = run_ok(&tmp);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(stderr, "", "stderr: {stderr}");
    assert!(out.status.success(), "status {:?}", out.status);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let expected_stdout = std::fs::read_to_string(fixture("expected-ok.stdout")).unwrap();
    assert_eq!(stdout, expected_stdout);
    let got = std::fs::read_to_string(&tmp).unwrap();
    let expected = std::fs::read_to_string(fixture("expected-ok.json")).unwrap();
    assert_eq!(got, expected);
    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn usage_exits_2_without_args() {
    let out = Command::new(bin()).output().expect("run android-convert");
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "");
    assert!(
        String::from_utf8_lossy(&out.stderr).starts_with("usage: android-convert "),
        "stderr {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn bad_host_name_matches_python() {
    let tmp = std::env::temp_dir().join("android-convert-bad-host.json");
    let out = Command::new(bin())
        .arg("--rind")
        .arg(fixture("rind-stub"))
        .arg("--capture")
        .arg(fixture("capture"))
        .arg("--out")
        .arg(&tmp)
        .arg("--host")
        .arg("bad name")
        .arg("--commit")
        .arg("c")
        .arg("--rustc")
        .arg("r")
        .arg("--cargo")
        .arg("c")
        .arg("--target")
        .arg("t")
        .arg("--cpu-model")
        .arg("m")
        .arg("--logical-cpus")
        .arg("1")
        .output()
        .expect("run android-convert");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "android-convert: host name must be a token without spaces or backticks\n"
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "");
}
