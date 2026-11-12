#!/usr/bin/env python3
"""Print a compact page JSON from a benchmark summary object.

The input is one object with `order`, `columns`, and `hosts`. Each host has
`rows` (operation -> {label, times}), `cpu`, `os`, `rustc`, `commit`,
`started`, `load`, `lead`, `timed`, `source`, and `build_cost`.
`render-side.py` does not write this file.

  page-data.py summary.json
"""

import json
import sys
from pathlib import Path


def main():
    if len(sys.argv) != 2:
        sys.exit("usage: page-data.py summary.json")
    summary = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
    order = summary["order"]
    columns = list(summary["columns"])
    labels = {}
    for host in summary["hosts"].values():
        for row, value in host["rows"].items():
            labels[row] = value["label"]
    rows = [[row, labels[row]] for row in order if row in labels]
    hosts = []
    for name, host in summary["hosts"].items():
        times = []
        for row, _label in rows:
            value = host["rows"].get(row)
            times.append(
                [
                    float(f"{value['times'][column]:.4g}") if value and column in value["times"] else None
                    for column in columns
                ]
            )
        cost = host.get("build_cost") or {}
        if cost.get("status") == "ok":
            builds = [
                [
                    build["name"],
                    build["median_seconds"],
                    round(build["median_cpu_seconds"]),
                    round(build["stripped_bytes"] / 1e6, 2),
                    len(build["packages"]),
                ]
                for build in cost.get("builds", [])
            ]
            failed = None
        else:
            builds = []
            failed = cost.get("reason")
        load = host.get("load") or {}
        hosts.append(
            {
                "id": name,
                "ours": host.get("source") == "this run",
                "cpu": host["cpu"]["model"],
                "arch": host["cpu"]["arch"],
                "os": host["os"],
                "rustc": host["rustc"].split(" (")[0],
                "commit": host["commit"][:8],
                "date": host["started"][:10],
                "noisy": bool(load.get("noisy")),
                "busy": load.get("other_busy_cpus_during_run"),
                "lead": host["lead"],
                "timed": host["timed"],
                "t": times,
                "builds": builds,
                "bcfail": failed,
            }
        )
    page = {
        "rows": rows,
        "cols": [[column, summary["columns"][column]] for column in columns],
        "hosts": hosts,
    }
    json.dump(page, sys.stdout, separators=(",", ":"))
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
