// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_page-data"))
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn fixture_matches_python_byte_for_byte() {
    let out = Command::new(bin())
        .arg(fixture("summary-ok.json"))
        .output()
        .expect("run page-data");
    assert_eq!(String::from_utf8_lossy(&out.stderr), "");
    assert!(out.status.success(), "status {:?}", out.status);
    let got = String::from_utf8_lossy(&out.stdout);
    let expected = std::fs::read_to_string(fixture("expected-ok.json")).unwrap();
    assert_eq!(got, expected);
}

#[test]
fn usage_matches_exit_and_stderr() {
    let out = Command::new(bin()).output().expect("run page-data");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "usage: page-data summary.json\n"
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "");
}
