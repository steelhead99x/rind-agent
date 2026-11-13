// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

//! Usage: gen-frames [--font-dir DIR] [OLED_DEMO_DIR]

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn usage() {
    eprintln!("usage: gen-frames [--font-dir DIR] [OLED_DEMO_DIR]");
}

fn default_font_dir() -> PathBuf {
    PathBuf::from("/usr/share/fonts/truetype/dejavu")
}

fn main() -> ExitCode {
    let mut font_dir = None;
    let mut demo_dir = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--font-dir" {
            match args.next() {
                Some(path) => font_dir = Some(PathBuf::from(path)),
                None => {
                    usage();
                    return ExitCode::from(1);
                }
            }
        } else if arg.starts_with('-') {
            usage();
            return ExitCode::from(1);
        } else if demo_dir.is_none() {
            demo_dir = Some(PathBuf::from(arg));
        } else {
            usage();
            return ExitCode::from(1);
        }
    }
    let font_dir = font_dir.unwrap_or_else(default_font_dir);
    let demo_dir = demo_dir.unwrap_or_else(|| PathBuf::from("."));
    match gen_frames::generate(&demo_dir, &font_dir) {
        Ok(names) => {
            let list = names
                .iter()
                .map(|n| format!("'{n}'"))
                .collect::<Vec<_>>()
                .join(", ");
            println!("ok [{list}]");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("gen-frames: {err}");
            ExitCode::from(2)
        }
    }
}
