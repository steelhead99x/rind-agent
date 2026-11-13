// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

//! Usage: page-data summary.json

use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let path = match (args.next(), args.next()) {
        (Some(path), None) => path,
        _ => {
            eprintln!("usage: page-data summary.json");
            return ExitCode::from(1);
        }
    };
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(err) => {
            eprintln!("page-data: {path}: {err}");
            return ExitCode::from(2);
        }
    };
    let summary = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("page-data: {path}: {err}");
            return ExitCode::from(2);
        }
    };
    match page_data::page_from_summary(&summary) {
        Ok(page) => {
            print!("{page}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("page-data: {err}");
            ExitCode::from(1)
        }
    }
}
