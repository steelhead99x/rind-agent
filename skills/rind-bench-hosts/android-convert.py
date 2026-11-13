#!/usr/bin/env python3
"""Pointer. Pixel captures are collected, then published, in rind-results.

    cargo xtask pixel-run --serial <id>
    cargo xtask bench --board pixel9pro --rind <path> --commit <pin> --capture <dir> --runs 3

--serial is required. This file does not convert a capture.
See skills/rind-bench-measure/SKILL.md.
"""

import sys

sys.exit(
    "moved to steelhead99x/rind-results: cargo xtask pixel-run --serial "
    "then cargo xtask bench --board pixel9pro "
    "(skills/rind-bench-measure/SKILL.md)"
)
