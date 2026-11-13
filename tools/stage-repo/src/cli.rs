// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

use std::path::PathBuf;

pub const USAGE: &str = "\
usage: stage-repo [-h] --out OUT --json JSON [--rename OLD=NEW]
                  [--replace REGEX=TEXT] [--render RENDER] [--log NAME=FILE]
                  [--note NAME=FILE]
";

pub const HELP: &str = "\
usage: stage-repo [-h] --out OUT --json JSON [--rename OLD=NEW]
                  [--replace REGEX=TEXT] [--render RENDER] [--log NAME=FILE]
                  [--note NAME=FILE]

Stage a results repository from host JSON and an optional rendered sheet.

options:
  -h, --help            show this help message and exit
  --out OUT
  --json JSON
  --rename OLD=NEW
  --replace REGEX=TEXT
  --render RENDER       directory with BENCHMARKS.md or STATS.md and
                        benchmarks/
  --log NAME=FILE
  --note NAME=FILE
";

pub struct Args {
    pub out: PathBuf,
    pub json: Vec<PathBuf>,
    pub rename: Vec<String>,
    pub replace: Vec<String>,
    pub render: Option<PathBuf>,
    pub log: Vec<String>,
    pub note: Vec<String>,
}

pub enum ParseOutcome {
    Ok(Args),
    Help,
    Error(String),
}

enum Flag {
    Out,
    Json,
    Rename,
    Replace,
    Render,
    Log,
    Note,
}

fn flag_of(name: &str) -> Option<Flag> {
    match name {
        "--out" => Some(Flag::Out),
        "--json" => Some(Flag::Json),
        "--rename" => Some(Flag::Rename),
        "--replace" => Some(Flag::Replace),
        "--render" => Some(Flag::Render),
        "--log" => Some(Flag::Log),
        "--note" => Some(Flag::Note),
        _ => None,
    }
}

pub fn parse_args(argv: &[String]) -> ParseOutcome {
    // Prefer --help/-h when the rest of argv is only known flags and values,
    // matching argparse for the usual help invocations.
    if argv.iter().any(|a| a == "-h" || a == "--help") && !has_unrecognized(argv) {
        return ParseOutcome::Help;
    }

    let mut out: Option<PathBuf> = None;
    let mut json = Vec::new();
    let mut rename = Vec::new();
    let mut replace = Vec::new();
    let mut render: Option<PathBuf> = None;
    let mut log = Vec::new();
    let mut note = Vec::new();

    let mut i = 0;
    while i < argv.len() {
        let arg = argv[i].as_str();
        if arg == "-h" || arg == "--help" {
            return ParseOutcome::Help;
        }
        let (name, inline) = split_long(arg);
        let Some(flag) = flag_of(name) else {
            return ParseOutcome::Error(unrecognized(arg));
        };
        let value = if let Some(v) = inline {
            v.to_string()
        } else if i + 1 < argv.len() && !is_option(&argv[i + 1]) {
            i += 1;
            argv[i].clone()
        } else {
            return ParseOutcome::Error(expected_one(name));
        };
        match flag {
            Flag::Out => out = Some(PathBuf::from(value)),
            Flag::Json => json.push(PathBuf::from(value)),
            Flag::Rename => rename.push(value),
            Flag::Replace => replace.push(value),
            Flag::Render => render = Some(PathBuf::from(value)),
            Flag::Log => log.push(value),
            Flag::Note => note.push(value),
        }
        i += 1;
    }

    let mut missing = Vec::new();
    if out.is_none() {
        missing.push("--out");
    }
    if json.is_empty() {
        missing.push("--json");
    }
    if !missing.is_empty() {
        return ParseOutcome::Error(format!(
            "{USAGE}stage-repo: error: the following arguments are required: {}\n",
            missing.join(", ")
        ));
    }

    ParseOutcome::Ok(Args {
        out: out.unwrap(),
        json,
        rename,
        replace,
        render,
        log,
        note,
    })
}

fn split_long(arg: &str) -> (&str, Option<&str>) {
    if let Some((name, value)) = arg.split_once('=') {
        (name, Some(value))
    } else {
        (arg, None)
    }
}

fn is_option(s: &str) -> bool {
    s == "-h" || s == "--help" || s.starts_with("--")
}

fn expected_one(name: &str) -> String {
    format!("{USAGE}stage-repo: error: argument {name}: expected one argument\n")
}

fn unrecognized(arg: &str) -> String {
    format!("{USAGE}stage-repo: error: unrecognized arguments: {arg}\n")
}

fn has_unrecognized(argv: &[String]) -> bool {
    let mut i = 0;
    while i < argv.len() {
        let arg = argv[i].as_str();
        if arg == "-h" || arg == "--help" {
            i += 1;
            continue;
        }
        let (name, inline) = split_long(arg);
        if flag_of(name).is_none() {
            return true;
        }
        if inline.is_some() {
            i += 1;
            continue;
        }
        if i + 1 < argv.len() && !is_option(&argv[i + 1]) {
            i += 2;
        } else {
            i += 1;
        }
    }
    false
}
