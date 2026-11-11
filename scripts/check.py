#!/usr/bin/env python3
"""Validate skill frontmatter and relative Markdown links."""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LINK = re.compile(r"\]\(([^)\s]+)\)")
errors = []

for skill in sorted((ROOT / "skills").glob("*/SKILL.md")):
    text = skill.read_text()
    match = re.match(r"---\n(.*?)\n---\n", text, re.S)
    if not match:
        errors.append(f"{skill}: missing YAML frontmatter")
        continue
    fields = dict(
        line.split(": ", 1) for line in match.group(1).splitlines() if ": " in line
    )
    if fields.get("name") != skill.parent.name:
        errors.append(f"{skill}: name must be {skill.parent.name!r}")
    if not fields.get("description", "").strip():
        errors.append(f"{skill}: description is empty")

for doc in sorted(ROOT.rglob("*.md")):
    for target in LINK.findall(doc.read_text()):
        if re.match(r"[a-z]+:", target) or target.startswith("#"):
            continue
        path = (doc.parent / target.split("#", 1)[0]).resolve()
        if not path.exists():
            errors.append(f"{doc.relative_to(ROOT)}: broken link {target}")

for error in errors:
    print(error, file=sys.stderr)
sys.exit(1 if errors else 0)
