---
name: rind-bench-results
description: Use when placing a published benchmark tree in rind-results, or reading an older side sheet that still uses the per-host layout.
---

# Where a published set goes

How the set was measured is
[rind-bench-measure](../rind-bench-measure/SKILL.md). The directory and
the JSON keys are
[rind-bench-schema](../rind-bench-schema/SKILL.md). The verify step is
[rind-bench-publish](../rind-bench-publish/SKILL.md). Host notes are in
[rind-bench-hosts](../rind-bench-hosts/SKILL.md).

`admin/bench-report render` rewrites the rind checkout's `BENCHMARKS.md`
and `benchmarks/*.svg`, and it only accepts the six `HOSTS` names. Why
that refuses a new host is in
[rind-bench-and-verify](../rind-bench-and-verify/SKILL.md). A published
set does not call `render`. rind-results `cargo xtask report --dir`
writes `RESULTS.md` into the output directory and leaves the rind
checkout's `BENCHMARKS.md` alone.

```shell
cargo xtask report --dir results/<host>/<upstream-sha>
cargo xtask verify --dir results/<host>/<upstream-sha>
```

Commit that directory to
[rind-results](https://github.com/steelhead99x/rind-results), not to
kixel-ai/rind and not to this skills repository. Do not commit a target
directory, `CARGO_HOME`, a source archive, a checkout, or `.tmp/`. Do
not put a token in a remote URL.

The old `render-side.py`, `stage-repo.py`, and `page-data.py` pointers
are gone.
`cargo xtask render` is the old side sheet. `cargo xtask stage`
regenerates an older per-host tree. A new set uses `report --dir`.
There is no `page-data` subcommand.

## Older trees

Trees already in rind-results keep their paths. That includes:

```text
<host>/<host>.json
<host>/<host>.note
<host>/run.log
render/BENCHMARKS.md
render/benchmarks/*.svg
```

and the board directories `results/esp32s3/`, `results/nrf52840/`, and
`results/msi-001/`. When a new `<upstream-sha>` directory is the
comparable set for that host, add a pointer in the old notes naming
the new path. Do not rewrite the old numbers in place.

`render()` in `admin/bench-report` returns Markdown and SVGs and does
not write them. A historical side sheet pointed `HOSTS`, `REPORT`, and
`REPORT_DIR` at the side hosts and a directory outside the rind
checkout, with `tiers=None` and `gaps=None`. `load_verified()` still
reads the checkout's `benchmarks/verified.toml`. With `tiers=None` the
CPU-tier section would say there are no nightly results, so that sheet
replaced the section, the generated-by comment, and the sentence that
every host used the pinned toolchain and ran only when idle, unless
that sentence was true of every host. Gaps in that sheet came from
`gaps_of` where rind is slower than `SLOWER` (1.1). They are
measurements. `benchmarks/gaps.toml` in the rind checkout stays the
maintainers' file. `rind-dit-held` is excluded (`VARIANTS`).

The host note adds what the JSON does not already show: a rustc other
than the pin, or a run outside `nix develop`; a shared or in-use
machine; OpenSSL headers unpacked locally; an aarch64 part with the
crypto extension and without SHA-3; on Apple silicon, that short gaps
need the DIT-held column; that ML-DSA-65 was timed here while the
upstream sheet has dashes on those three rows.
