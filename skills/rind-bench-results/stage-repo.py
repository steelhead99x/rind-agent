#!/usr/bin/env python3
"""Pointer. Staging a results tree moved to steelhead99x/rind-results.

A published set is written straight into results/<host>/<upstream-sha>/
by that repo's cargo xtask:

    cargo xtask report --dir <dir>
    cargo xtask verify --dir <dir>

This file does not stage a tree. See skills/rind-bench-schema/SKILL.md.
"""

import sys

sys.exit(
    "moved to steelhead99x/rind-results: cargo xtask report --dir "
    "(skills/rind-bench-schema/SKILL.md)"
)
