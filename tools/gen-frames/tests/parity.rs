// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use gen_frames::font::Fonts;
use gen_frames::generate::{build_frames, frames_header};
use gen_frames::png_rgb;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_gen-frames"))
}

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn setup_demo(root: &std::path::Path) {
    let assets = root.join("assets");
    fs::create_dir_all(&assets).unwrap();
    for name in ["moqdev-layout_favicon.svg", "wordmark-light.svg"] {
        fs::copy(fixtures().join("assets").join(name), assets.join(name)).unwrap();
    }
}

#[test]
fn frames_header_matches_python_byte_for_byte() {
    let fonts = Fonts::new(&fixtures().join("fonts")).expect("load fonts");
    let frames = build_frames(&fixtures().join("assets"), &fonts).expect("build frames");
    let got = frames_header(&frames);
    let expected = fs::read_to_string(fixtures().join("expected/frames.h")).unwrap();
    assert_eq!(got, expected);
}

#[test]
fn preview_pixels_match_python() {
    let fonts = Fonts::new(&fixtures().join("fonts")).expect("load fonts");
    let frames = build_frames(&fixtures().join("assets"), &fonts).expect("build frames");
    let dir = tempfile_demo();
    setup_demo(&dir);
    let names = gen_frames::generate(&dir, &fixtures().join("fonts")).expect("generate");
    assert_eq!(names, ["moq", "rind", "bench"]);
    for name in [
        "preview-1-moq.png",
        "preview-2-rind.png",
        "preview-3-bench.png",
        "preview.png",
    ] {
        let (w, h, got) = png_rgb::read_rgb(&dir.join(name)).expect(name);
        let (ew, eh, expected) =
            png_rgb::read_rgb(&fixtures().join("expected").join(name)).expect(name);
        assert_eq!((w, h), (ew, eh), "{name} size");
        let diffs = got
            .iter()
            .zip(expected.iter())
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(diffs, 0, "{name} pixel diffs");
    }
    let _ = frames;
}

#[test]
fn cli_writes_header_and_matches_stdout() {
    let dir = tempfile_demo();
    setup_demo(&dir);
    let out = Command::new(bin())
        .arg("--font-dir")
        .arg(fixtures().join("fonts"))
        .arg(&dir)
        .output()
        .expect("run gen-frames");
    assert_eq!(String::from_utf8_lossy(&out.stderr), "");
    assert!(out.status.success(), "status {:?}", out.status);
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "ok ['moq', 'rind', 'bench']\n"
    );
    let got = fs::read_to_string(dir.join("oled_demo/frames.h")).unwrap();
    let expected = fs::read_to_string(fixtures().join("expected/frames.h")).unwrap();
    assert_eq!(got, expected);
}

#[test]
fn usage_matches_exit_and_stderr() {
    let out = Command::new(bin())
        .arg("--unknown")
        .output()
        .expect("run gen-frames");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "usage: gen-frames [--font-dir DIR] [OLED_DEMO_DIR]\n"
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "");
}

fn tempfile_demo() -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "gen-frames-parity-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&base).unwrap();
    base
}

/// Pillow 11.3 RAQM `getlength` with the fixture fonts, times 64: totals
/// that include GPOS kerning (`Xeon`, `M1 Pro`, `TLS 1.3 handshake`,
/// `pure-Rust crypto`) and ones that do not.
#[test]
fn raqm_lengths_match_pillow() {
    use gen_frames::font::FontKind::{Sans, SansBold};
    let fonts = Fonts::new(&fixtures().join("fonts")).expect("load fonts");
    for (text, kind, size, want) in [
        ("moq.dev", SansBold, 13, 4053),
        ("Media over", Sans, 10, 3553),
        ("QUIC", SansBold, 10, 1772),
        ("pure-Rust crypto", Sans, 9, 4804),
        ("TLS 1.3 handshake", SansBold, 9, 6120),
        ("X25519, full", Sans, 9, 3479),
        ("aws-lc", SansBold, 9, 2041),
        ("Xeon", Sans, 9, 1440),
        ("M1 Pro", Sans, 9, 1959),
        ("1.57ms", Sans, 9, 2142),
    ] {
        for mono in [false, true] {
            let got = fonts.text_length(text, kind, size, mono).expect("length");
            assert_eq!(
                got * 64.0,
                f64::from(want),
                "{text} at {size} (mono {mono})"
            );
        }
    }
}
