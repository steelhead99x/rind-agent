#!/usr/bin/env python3
"""Pointer. Side-sheet rendering moved to steelhead99x/rind-results.

rind-results names the old renderer scripts/render-ours.py. A published
set does not run that script. From a rind-results checkout:

    cargo xtask report --dir <dir>
    cargo xtask verify --dir <dir>

This file does not render. See skills/rind-bench-publish/SKILL.md.
"""

import sys

sys.exit(
    "moved to steelhead99x/rind-results: cargo xtask report --dir "
    "(skills/rind-bench-publish/SKILL.md)"
)
