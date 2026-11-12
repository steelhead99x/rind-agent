#!/usr/bin/env python3
"""Run rind's `admin/bench-report run` when that script would refuse the host.

Same operations, criterion arguments, parsing, and JSON. What changes:

* `toolchain()` does not require `IN_RIND_DEV_SHELL`, and it does not require
  the rustc release `flake.nix` pins. The host's `rustc -vV` first line is
  what the JSON stores.
* `check_idle()` does not refuse a busy host. It still returns the busy-CPU
  count, so `run()` stores it under `load` and still sets `load.noisy` from
  other work during the benchmarks.
* If `HOSTRUN_FREE_TARGET` is `1`, the benchmark target directory is deleted
  before `admin/build-cost`. If `HOSTRUN_SKIP_BUILD_COST` is `1`, build cost
  is recorded as failed with a reason that says it was skipped.

Prefer `nix develop --command python3 admin/bench-report run` when the host
can meet the real checks. Point `RIND_ROOT` at the checkout if this file is
not in that checkout's root.
"""

import importlib.machinery
import importlib.util
import os
import shutil
import sys

root = os.environ.get("RIND_ROOT") or os.path.dirname(os.path.abspath(__file__))
path = os.path.join(root, "admin", "bench-report")
loader = importlib.machinery.SourceFileLoader("bench_report", path)
spec = importlib.util.spec_from_loader("bench_report", loader)
br = importlib.util.module_from_spec(spec)
loader.exec_module(br)

for _name in (
    "toolchain",
    "check_idle",
    "build_cost",
    "busy_cpus",
    "cpu_count",
    "busy_limit",
    "output",
    "main",
):
    if not hasattr(br, _name):
        sys.exit(f"host-run: admin/bench-report has no {_name}; this wrapper is out of date")


def toolchain(env):
    version = br.output(["rustc", "-vV"], env)
    cargo = br.output(["cargo", "-V"], env)
    return ["cargo"], {"rustc": version.splitlines()[0], "cargo": cargo}, "openssl"


def check_idle(when):
    busy = br.busy_cpus()
    limit = br.busy_limit()
    print(
        f"host-run: {busy:.1f} of {br.cpu_count()} CPUs busy {when} "
        f"(harness limit {limit:.1f}); continuing",
        file=sys.stderr,
        flush=True,
    )
    return round(busy, 2)


orig_build_cost = br.build_cost


def build_cost(env, jobs, refused):
    if os.environ.get("HOSTRUN_FREE_TARGET") == "1":
        shutil.rmtree(env["CARGO_TARGET_DIR"], ignore_errors=True)
    if os.environ.get("HOSTRUN_SKIP_BUILD_COST") == "1":
        return {
            "status": "failed",
            "started": "",
            "reason": "skipped by host-run (HOSTRUN_SKIP_BUILD_COST)",
        }
    return orig_build_cost(env, jobs, refused)


br.toolchain = toolchain
br.check_idle = check_idle
br.build_cost = build_cost
sys.argv = ["bench-report", "run", *sys.argv[1:]]
br.main()
