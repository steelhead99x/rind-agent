#!/usr/bin/env python3
"""Stage a results repository from host JSON and an optional rendered sheet.

Rewrites home-directory paths and private IPv4 addresses. Pass
--replace REGEX=TEXT for a machine name or a board product name.

  stage-repo.py --out DIR --json FILE [--rename OLD=NEW] [--render DIR]
                [--log NAME=FILE] [--replace REGEX=TEXT]
"""

import argparse
import json
import re
import sys
from pathlib import Path

HOME = re.compile("(?:/" + "Users|/" + "home)/" + r"\S+")
PRIVATE_IP = re.compile(
    r"\b(?:192\.168\.\d{1,3}\.\d{1,3}|10\.\d{1,3}\.\d{1,3}\.\d{1,3}|"
    r"172\.(?:1[6-9]|2\d|3[0-1])\.\d{1,3}\.\d{1,3})\b"
)
LOG_KEEP = re.compile(
    r"^(Benchmarking \S+$|\s+time:|\s+thrpt:|host-run:|Warning:|error|"
    r"bench-report|start |end |WATCHDOG|ABORT|libssl|.*Finished|.*failed|.*panicked)"
)


def scrubber(replacements):
    compiled = [(re.compile(pattern), text) for pattern, text in replacements]

    def scrub(value):
        value = HOME.sub("~", value)
        value = PRIVATE_IP.sub("<lan-ip>", value)
        for pattern, text in compiled:
            value = pattern.sub(text, value)
        return value

    return scrub


def load_host(path, scrub):
    try:
        host = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        sys.exit(f"stage-repo: {path}: {error}")
    try:
        host = json.loads(scrub(json.dumps(host)))
    except json.JSONDecodeError as error:
        sys.exit(f"stage-repo: scrubbing {path} broke the JSON: {error}")
    return host


def notes_for(host):
    load = host.get("load") or {}
    cost = host.get("build_cost") or {}
    openssl = (host.get("libraries") or {}).get("openssl") or {}
    cost_line = f"- Build cost: {cost.get('status')}"
    if cost.get("status") == "ok":
        tool = cost.get("host") or {}
        cost_line += f", {tool.get('cc')}, {tool.get('jobs')} jobs"
    elif cost.get("reason"):
        cost_line += f" ({cost.get('reason')})"
    timed = len({row["operation"] for row in host.get("results") or []})
    lines = [
        f"# `{host.get('host')}`",
        "",
        f"- CPU: {host.get('cpu', {}).get('model')} ({host.get('cpu', {}).get('arch')}, "
        f"{host.get('cpu', {}).get('logical_cpus')} logical CPUs)",
        f"- OS: {host.get('os')}",
        f"- Toolchain: {(host.get('toolchain') or {}).get('rustc')}; "
        f"{(host.get('toolchain') or {}).get('cargo')}",
        f"- Commit: {host.get('commit')}",
        f"- Started / finished (UTC): {host.get('started')} / {host.get('finished')}",
        f"- Busy CPUs before build {load.get('busy_cpus_before_build')}, "
        f"before run {load.get('busy_cpus_before_run')}, "
        f"during run {load.get('other_busy_cpus_during_run')} "
        f"(harness limit {load.get('limit_cpus')})",
        f"- Noisy flag (harness): {load.get('noisy')}",
        f"- Libraries refused for missing CPU features: {host.get('refused') or 'none'}",
        f"- OpenSSL linked: {openssl.get('linked')}",
        cost_line,
        f"- Operations timed: {timed}; results: {len(host.get('results') or [])}",
        "",
    ]
    return "\n".join(lines)


def trim_log(text):
    kept = []
    for line in text.splitlines():
        if not LOG_KEEP.match(line):
            continue
        if line.startswith("Benchmarking ") and kept and kept[-1].startswith("Benchmarking "):
            kept[-1] = line
        else:
            kept.append(line)
    return "\n".join(kept) + ("\n" if kept else "")


def copy_tree_text(source, dest, scrub):
    dest.mkdir(parents=True, exist_ok=True)
    for path in source.rglob("*"):
        if not path.is_file():
            continue
        relative = path.relative_to(source)
        target = dest / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(scrub(path.read_text(encoding="utf-8", errors="replace")), encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--json", required=True, action="append", type=Path)
    parser.add_argument("--rename", action="append", default=[], metavar="OLD=NEW")
    parser.add_argument("--replace", action="append", default=[], metavar="REGEX=TEXT")
    parser.add_argument("--render", type=Path, help="directory with BENCHMARKS.md or STATS.md and benchmarks/")
    parser.add_argument("--log", action="append", default=[], metavar="NAME=FILE")
    parser.add_argument("--note", action="append", default=[], metavar="NAME=FILE")
    args = parser.parse_args()

    renames = {}
    for item in args.rename:
        old, sep, new = item.partition("=")
        if not sep or not old or not new:
            sys.exit(f"stage-repo: --rename must be OLD=NEW, not {item!r}")
        renames[old] = new
    replacements = []
    for item in args.replace:
        pattern, sep, text = item.partition("=")
        if not sep or not pattern:
            sys.exit(f"stage-repo: --replace must be REGEX=TEXT, not {item!r}")
        replacements.append((pattern, text))
    scrub = scrubber(replacements)
    logs = {}
    notes = {}
    for flag, store in (("--log", logs), ("--note", notes)):
        for item in getattr(args, flag[2:]):
            name, sep, path = item.partition("=")
            if not sep or not name or not path:
                sys.exit(f"stage-repo: {flag} must be NAME=FILE, not {item!r}")
            store[name] = Path(path)

    out = args.out
    for path in args.json:
        host = load_host(path, scrub)
        name = renames.get(host.get("host"), host.get("host"))
        if not isinstance(name, str) or not name or any(ch.isspace() or ch == "`" for ch in name):
            sys.exit(f"stage-repo: {path}: host name must be a token")
        host["host"] = name
        dest = out / name
        dest.mkdir(parents=True, exist_ok=True)
        (dest / f"{name}.json").write_text(json.dumps(host, indent=1) + "\n", encoding="utf-8")
        extra = ""
        if name in notes:
            extra = scrub(notes[name].read_text(encoding="utf-8", errors="replace"))
            if extra and not extra.endswith("\n"):
                extra += "\n"
        (dest / "NOTES.md").write_text(scrub(notes_for(host)) + extra, encoding="utf-8")
        if name in logs:
            raw = logs[name].read_text(encoding="utf-8", errors="replace")
            body = trim_log(raw) if logs[name].suffix == ".log" else raw
            if body and not body.endswith("\n"):
                body += "\n"
            (dest / logs[name].name).write_text(scrub(body), encoding="utf-8")

    if args.render:
        sheet = None
        for candidate in ("BENCHMARKS.md", "STATS.md"):
            path = args.render / candidate
            if path.is_file():
                sheet = path
                break
        if sheet is None:
            sys.exit(f"stage-repo: {args.render} has no BENCHMARKS.md or STATS.md")
        (out / "README.md").write_text(scrub(sheet.read_text(encoding="utf-8")), encoding="utf-8")
        benchmarks = args.render / "benchmarks"
        if benchmarks.is_dir():
            copy_tree_text(benchmarks, out / "benchmarks", scrub)
    print(out)


if __name__ == "__main__":
    main()
