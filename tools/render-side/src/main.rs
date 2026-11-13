// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

//! Usage: render-side --rind RIND --out OUT [--note HOST FILE]... JSON...

use render_side::RenderArgs;
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage: render-side --rind RIND --out OUT [--note HOST FILE]... JSON...";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let parsed = match parse_args(&args) {
        Ok(a) => a,
        Err(code) => return ExitCode::from(code),
    };
    match render_side::render(&parsed) {
        Ok(rendered) => {
            println!("{}", rendered.sheet.display());
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}

fn parse_args(args: &[String]) -> Result<RenderArgs, u8> {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        eprintln!("{USAGE}");
        return Err(0);
    }
    let mut rind = None;
    let mut out = None;
    let mut notes = Vec::new();
    let mut json = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if a == "--rind" || a == "--out" {
            if i + 1 >= args.len() {
                eprintln!("{USAGE}");
                return Err(2);
            }
            let value = PathBuf::from(&args[i + 1]);
            if a == "--rind" {
                rind = Some(value);
            } else {
                out = Some(value);
            }
            i += 2;
        } else if a == "--note" {
            if i + 2 >= args.len() {
                eprintln!("{USAGE}");
                return Err(2);
            }
            notes.push((args[i + 1].clone(), PathBuf::from(&args[i + 2])));
            i += 3;
        } else if a.starts_with("--") {
            eprintln!("{USAGE}");
            return Err(2);
        } else {
            json.push(PathBuf::from(a));
            i += 1;
        }
    }
    match (rind, out) {
        (Some(rind), Some(out)) if !json.is_empty() => Ok(RenderArgs {
            rind,
            out,
            notes,
            json,
        }),
        _ => {
            eprintln!("{USAGE}");
            Err(2)
        }
    }
}
