// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

pub mod cli;
pub mod jsonfmt;
pub mod notes;
pub mod run;
pub mod scrub;

use std::ffi::OsString;
use std::io::{self, Write};
use std::process::ExitCode;

use cli::{HELP, ParseOutcome, parse_args};

/// Entry used by the binary. Exit 0 on success or `--help`, 2 on argparse
/// errors, 1 on every other `stage-repo:` error.
pub fn main_with_args<I, S>(args: I) -> ExitCode
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let argv: Vec<String> = args
        .into_iter()
        .map(|s| s.into().to_string_lossy().into_owned())
        .collect();
    match parse_args(&argv) {
        ParseOutcome::Help => {
            let _ = write!(io::stdout(), "{HELP}");
            ExitCode::SUCCESS
        }
        ParseOutcome::Error(msg) => {
            let _ = write!(io::stderr(), "{msg}");
            ExitCode::from(2)
        }
        ParseOutcome::Ok(args) => match run::run(&args) {
            Ok(out) => {
                println!("{}", out.display());
                ExitCode::SUCCESS
            }
            Err(msg) => {
                eprintln!("{msg}");
                ExitCode::from(1)
            }
        },
    }
}
