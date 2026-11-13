// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

//! Usage: android-convert --rind RIND --capture CAPTURE --out OUT --host HOST
//!   --commit COMMIT --rustc RUSTC --cargo CARGO --target TARGET
//!   --cpu-model CPU_MODEL --logical-cpus N [--openssl-linked TEXT]

use android_convert::{ConvertArgs, write_out};
use std::env;
use std::process::ExitCode;

const USAGE: &str = "usage: android-convert --rind RIND --capture CAPTURE --out OUT \
--host HOST --commit COMMIT --rustc RUSTC --cargo CARGO --target TARGET \
--cpu-model CPU_MODEL --logical-cpus LOGICAL_CPUS [--openssl-linked OPENSSL_LINKED]";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let parsed = match parse_args(&args) {
        Ok(a) => a,
        Err(code) => return ExitCode::from(code),
    };
    match android_convert::convert(&parsed) {
        Ok(converted) => {
            if let Err(err) = write_out(&parsed, &converted) {
                eprintln!("{err}");
                return ExitCode::from(1);
            }
            print!("{}", converted.stdout);
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}

fn parse_args(args: &[String]) -> Result<ConvertArgs, u8> {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        eprintln!("{USAGE}");
        return Err(0);
    }
    let mut rind = None;
    let mut capture = None;
    let mut out = None;
    let mut host = None;
    let mut commit = None;
    let mut rustc = None;
    let mut cargo = None;
    let mut target = None;
    let mut cpu_model = None;
    let mut logical_cpus = None;
    let mut openssl_linked = String::new();
    let mut i = 0;
    while i < args.len() {
        let flag = args[i].as_str();
        if !flag.starts_with("--") {
            eprintln!("{USAGE}");
            return Err(2);
        }
        i += 1;
        if i >= args.len() {
            eprintln!("{USAGE}");
            return Err(2);
        }
        let value = args[i].clone();
        i += 1;
        match flag {
            "--rind" => rind = Some(value),
            "--capture" => capture = Some(value),
            "--out" => out = Some(value),
            "--host" => host = Some(value),
            "--commit" => commit = Some(value),
            "--rustc" => rustc = Some(value),
            "--cargo" => cargo = Some(value),
            "--target" => target = Some(value),
            "--cpu-model" => cpu_model = Some(value),
            "--logical-cpus" => {
                logical_cpus = match value.parse::<i64>() {
                    Ok(n) => Some(n),
                    Err(_) => {
                        eprintln!("android-convert: --logical-cpus must be an integer");
                        return Err(2);
                    }
                };
            }
            "--openssl-linked" => openssl_linked = value,
            _ => {
                eprintln!("{USAGE}");
                return Err(2);
            }
        }
    }
    match (
        rind,
        capture,
        out,
        host,
        commit,
        rustc,
        cargo,
        target,
        cpu_model,
        logical_cpus,
    ) {
        (
            Some(rind),
            Some(capture),
            Some(out),
            Some(host),
            Some(commit),
            Some(rustc),
            Some(cargo),
            Some(target),
            Some(cpu_model),
            Some(logical_cpus),
        ) => Ok(ConvertArgs {
            rind: rind.into(),
            capture: capture.into(),
            out: out.into(),
            host,
            commit,
            rustc,
            cargo,
            target,
            cpu_model,
            logical_cpus,
            openssl_linked,
        }),
        _ => {
            eprintln!("{USAGE}");
            Err(2)
        }
    }
}
