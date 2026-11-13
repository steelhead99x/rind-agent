#!/usr/bin/env python3
"""Pointer. A published tree is written by rind-results `cargo xtask bench`.

    cargo xtask bench --board <id> --rind <path> --commit <pin> --runs 3
    cargo xtask verify --dir <dir>

`cargo xtask stage` only regenerates an older per-host tree.
This file does not stage one. See skills/rind-bench-schema/SKILL.md.
"""

import sys

sys.exit(
    "moved to steelhead99x/rind-results: cargo xtask bench "
    "then cargo xtask verify --dir "
    "(skills/rind-bench-schema/SKILL.md)"
)
