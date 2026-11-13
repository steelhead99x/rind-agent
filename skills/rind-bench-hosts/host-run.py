#!/usr/bin/env python3
"""Pointer. A published host run is rind-results `cargo xtask bench`.

    cargo xtask bench --board <id> --rind <path> --commit <pin> --runs 3

`<id>` is xeon-vm, m1pro, pi5, msi-001, or jetson-orin-nano.
jetson-orin-nano also passes --strict. mac-run, pi-run, and orin-runs
are collectors; see skills/rind-bench-measure/SKILL.md.
This file does not measure.
"""

import sys

sys.exit(
    "moved to steelhead99x/rind-results: cargo xtask bench "
    "(skills/rind-bench-measure/SKILL.md)"
)
