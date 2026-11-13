// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args_os();
    let _argv0 = args.next();
    stage_repo::main_with_args(args)
}
