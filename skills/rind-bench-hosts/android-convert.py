#!/usr/bin/env python3
"""Turn an Android capture into an admin/bench-report host JSON.

The phone does not run `admin/bench-report run`. This imports that script
and parses criterion the same way `run()` does: `benchmark.json` plus
`estimates.json`, slope if present, otherwise mean.

The capture directory holds:

  getprop.txt, stat-before, stat-after
  battery-before.txt, battery-after.txt
  started, finished          ISO-8601 timestamps
  cpuid.json                 rind-cpuid's JSON, or {"cpuid": {}}
  criterion/**/new/benchmark.json and estimates.json
  openssl-linked.txt         optional; else pass --openssl-linked
"""

import argparse
import glob
import importlib.machinery
import importlib.util
import json
import os
import re
import sys
from pathlib import Path


def load_renderer(rind):
    path = rind / "admin" / "bench-report"
    if not path.is_file():
        sys.exit(f"android-convert: no {path}")
    loader = importlib.machinery.SourceFileLoader("bench_report", str(path))
    spec = importlib.util.spec_from_loader("bench_report", loader)
    module = importlib.util.module_from_spec(spec)
    loader.exec_module(module)
    for name in ("operation_of", "OPERATIONS", "LIBRARIES", "CRATES", "locked_versions", "CRITERION_ARGS", "operation_regex"):
        if not hasattr(module, name):
            sys.exit(f"android-convert: admin/bench-report has no {name}")
    return module


def read_text(path):
    try:
        return path.read_text(encoding="utf-8", errors="replace").strip()
    except OSError as error:
        sys.exit(f"android-convert: {path}: {error}")


def getprop(path):
    props = {}
    for line in read_text(path).splitlines():
        match = re.match(r"\[(.+?)\]: \[(.*)\]", line)
        if match:
            props[match.group(1)] = match.group(2)
    return props


def cpu_times(text):
    fields = [int(part) for part in text.split()[1:9]]
    idle = fields[3] + fields[4]
    return sum(fields) - idle, sum(fields)


def dumpsys_map(text):
    out = {}
    for line in text.splitlines():
        if ":" not in line:
            continue
        key, value = line.split(":", 1)
        out[key.strip()] = value.strip()
    return out


def temperature_c(battery):
    try:
        return int(battery.get("temperature", "0")) / 10
    except ValueError:
        return None


def results_of(module, capture):
    found_results = []
    pattern = str(capture / "criterion" / "**" / "new" / "benchmark.json")
    for path in sorted(glob.glob(pattern, recursive=True)):
        meta = json.loads(Path(path).read_text(encoding="utf-8"))
        found = module.operation_of(meta["full_id"])
        if not found:
            continue
        operation, library = found
        estimates_path = Path(path).parent / "estimates.json"
        estimates = json.loads(estimates_path.read_text(encoding="utf-8"))
        estimate = estimates.get("slope") or estimates["mean"]
        found_results.append(
            {
                "operation": operation,
                "library": module.LIBRARIES.get(library, library),
                "id": meta["full_id"],
                "ns": estimate["point_estimate"],
                "ns_low": estimate["confidence_interval"]["lower_bound"],
                "ns_high": estimate["confidence_interval"]["upper_bound"],
                "throughput": meta.get("throughput"),
            }
        )
    order = {operation: index for index, operation in enumerate(module.OPERATIONS)}
    found_results.sort(key=lambda row: (order[row["operation"]], row["library"]))
    return found_results


def refused_of(module, host):
    if not hasattr(module, "REQUIRED") or not hasattr(module, "host_arch"):
        return {}
    cpuid = host["rind_cpu"].get("cpuid", {})
    arch = module.host_arch(host)
    refused = {}
    for library, features in module.REQUIRED.items():
        missing = [feature for feature in features.get(arch, []) if not cpuid.get(feature)]
        if missing:
            refused[library] = missing
    return refused


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--rind", required=True, type=Path)
    parser.add_argument("--capture", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--host", required=True)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--rustc", required=True, help="first line of rustc -vV on the build machine")
    parser.add_argument("--cargo", required=True)
    parser.add_argument("--target", required=True, help="triple, API level, and NDK, as text")
    parser.add_argument("--cpu-model", required=True)
    parser.add_argument("--logical-cpus", required=True, type=int)
    parser.add_argument("--openssl-linked", default="", help="what the binary linked; else openssl-linked.txt")
    args = parser.parse_args()
    if any(ch.isspace() or ch == "`" for ch in args.host):
        sys.exit("android-convert: host name must be a token without spaces or backticks")

    capture = args.capture
    linked_path = capture / "openssl-linked.txt"
    linked = args.openssl_linked.strip() or (read_text(linked_path) if linked_path.is_file() else "")
    if not linked:
        sys.exit("android-convert: pass --openssl-linked or provide openssl-linked.txt")

    module = load_renderer(args.rind.resolve())
    props = getprop(capture / "getprop.txt")
    busy0, total0 = cpu_times(read_text(capture / "stat-before"))
    busy1, total1 = cpu_times(read_text(capture / "stat-after"))
    busy_share = (busy1 - busy0) / max(1, total1 - total0)
    before = dumpsys_map(read_text(capture / "battery-before.txt"))
    after = dumpsys_map(read_text(capture / "battery-after.txt"))
    cpuid_path = capture / "cpuid.json"
    rind_cpu = json.loads(read_text(cpuid_path)) if cpuid_path.is_file() else {"cpuid": {}}

    locked = module.locked_versions()
    libraries = {
        library: {crate: ", ".join(locked.get(crate, ["?"])) for crate in crates}
        for library, crates in module.CRATES.items()
    }
    libraries["openssl"]["linked"] = linked

    host = {
        "schema": 1,
        "host": args.host,
        "commit": args.commit,
        "started": read_text(capture / "started"),
        "cpu": {"model": args.cpu_model, "arch": "aarch64", "logical_cpus": args.logical_cpus},
        "os": (
            f"Android {props.get('ro.build.version.release', '?')} "
            f"(API {props.get('ro.build.version.sdk', '?')}, "
            f"build {props.get('ro.build.id', '?')})"
        ),
        "toolchain": {"rustc": args.rustc, "cargo": args.cargo, "target": args.target},
        "libraries": libraries,
        "rind_cpu": rind_cpu,
        "status": "ok",
        "finished": read_text(capture / "finished"),
        "criterion_args": module.CRITERION_ARGS + [module.operation_regex(())],
        "load": {
            "limit_cpus": round(max(1.0, 0.1 * args.logical_cpus), 2),
            "system_busy_share_during_run": round(busy_share, 3),
            "note": (
                "whole-system busy share across the run, benchmarks included; "
                "not the harness's other-work count"
            ),
            "noisy": None,
        },
        "battery": {
            "before": before,
            "after": after,
            "temperature_c": {"before": temperature_c(before), "after": temperature_c(after)},
        },
        "results": results_of(module, capture),
        "build_cost": {
            "status": "failed",
            "started": "",
            "reason": "not measured: the binaries were cross-compiled on another host",
        },
    }
    host["refused"] = refused_of(module, host)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(host, indent=1) + "\n", encoding="utf-8")
    print(f"{len(host['results'])} results; busy share {busy_share:.3f}")


if __name__ == "__main__":
    main()
