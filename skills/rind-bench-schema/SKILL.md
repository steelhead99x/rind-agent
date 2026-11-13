---
name: rind-bench-schema
description: Use when writing or reading the JSON, metadata, and directory of a published rind benchmark set.
---

# Schema and layout of a published set

The commit rules are in
[rind-bench-pin](../rind-bench-pin/SKILL.md). The run that fills this
tree is [rind-bench-measure](../rind-bench-measure/SKILL.md). Nothing
here is published until
[rind-bench-publish](../rind-bench-publish/SKILL.md) passes.

## Schema 1

`<host>.json` is bench-report schema 1, the object `admin/bench-report
run` writes with `"schema": 1`. At the pin in rind-bench-pin that object
holds `host`, `commit`, `started`, `finished`, `cpu`, `os`, `status`,
`toolchain`, `libraries`, `rind_cpu`, `refused`, `criterion_args`,
`load`, `results`, and `build_cost`. Each `results` row holds
`operation`, `library`, `id`, `ns`, `ns_low`, `ns_high`, and
`throughput`. Keep `"schema": 1`. Optional keys do not bump it.

`commit` is the upstream SHA from rind-bench-pin, even when the binary
was built from a fork.

Optional keys, omitted when they do not apply:

- `upstream_commit`, the same full upstream SHA as `commit`, present
  whenever `fork_commit` is set.
- `fork_commit`, the measured fork `HEAD`.
- `patches`, a list of `{ "name", "sha256" }`.
- `hardware_commit`, the rind-hardware commit that was built.
- `harness_commit`, the commit of the harness that was timed, when that
  harness is not the rind commit (the firmware repo on a board).
- `firmware_sha256`, the sha256 of the one image that was timed. A run
  with one image per bench file keeps each image's sha256 beside its
  name, as rind-results `images.json` already does, and omits this key.
- `runs`, how many runs were combined.
- `runs_ns`, on a result row, that row's estimate from each run.
- `warmup`, `iterations`, `metric_units`.
- `cycles`, `cycles_per_byte`, `run_spread_pct`, on a board row.
- `spread_is`.

`ns` is the median of the runs' estimates. On a host the estimate is
the one `admin/bench-report` stores: criterion's slope, or the mean
when there is no slope. On a board the estimate is that run's
cycle-counter median.

When `ns_low` and `ns_high` are the minimum and maximum of those run
estimates, set `spread_is` to `runs`. Host JSON that still carries
criterion's confidence interval in those two fields does not set
`spread_is`. rind-results `cargo xtask report` on `main` at
`d4a1d803` stores the run minimum and maximum in `ns_low` and
`ns_high` for a board, so a
published file of that shape includes `spread_is`.

`meta.json` repeats the provenance so a reader does not have to mine
the report: `upstream_commit`, and, when set, `fork_commit`, `patches`,
`hardware_commit`, `harness_commit`, and `firmware_sha256`.

## Directory

Under the rind-results checkout:

```text
results/<host>/<upstream-sha>/<host>.json
results/<host>/<upstream-sha>/meta.json
results/<host>/<upstream-sha>/NOTES.md
results/<host>/<upstream-sha>/run.log
results/<host>/<upstream-sha>/raw/
results/<host>/<upstream-sha>/RESULTS.md
```

`<host>` is the published id, the same string as JSON `host`.
`<upstream-sha>` is the full 40-character upstream SHA. An A/B or a
diagnostic set uses `results/<host>/<upstream-sha>-<label>/`. `<label>`
is lowercase kebab-case (`icache32`, `poly1305-32`). The comparable set
has no label.

`NOTES.md` records what the JSON does not: the board clock, a shared
machine, a rustc other than the flake pin, OpenSSL headers unpacked
locally, an aarch64 part with the crypto extension and without SHA-3,
and, on Apple silicon, that a short-input gap needs the DIT-held
column. Those host facts are in
[rind-bench-hosts](../rind-bench-hosts/SKILL.md).

## Older trees

Leave a tree that is already published where it is. That includes
`results/<host>/<host>.json` with no upstream SHA in the path,
`results/esp32s3/` and `results/nrf52840/` at the repository root of
`results/`, `results/msi-001/`, the ESP32-S3 set whose flash clock was
40 MHz, and any directory whose SHA is the previous comparable set
`0b1f1469c8e17e361e3353432bc7fceee9e04214`. `cargo xtask bench` refuses
to write that previous SHA. When a new tree replaces one of them as the comparable set,
add a short pointer in the old `NOTES.md` naming the new directory. Do
not rewrite the old numbers into the new shape in place.

A host directory of `raw/run-*.json` plus `<host>.json` was built by
`cargo xtask host-report`. Regenerating that historical directory uses
the command its `RESULTS.md` names. A new comparable set uses the
layout above.
