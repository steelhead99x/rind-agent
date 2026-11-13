---
name: rind-bench-results
description: Use when rendering a side benchmark sheet for hosts outside admin/bench-report's HOSTS, or committing that sheet and the raw JSON to a results repository.
---

# A side sheet, and where the JSON goes

How the host was measured is in
[rind-bench-hosts](../rind-bench-hosts/SKILL.md). Why `render` cannot take
a new host is in
[rind-bench-and-verify](../rind-bench-and-verify/SKILL.md).

`report()` returns Markdown and SVGs. It does not write them. Point
`HOSTS`, `REPORT`, and `REPORT_DIR` at the side hosts and a directory
outside the checkout. Pass `tiers=None` and `gaps=None`. `load_verified()`
still reads the checkout's `benchmarks/verified.toml` (`VERIFIED` is set at
import). With `tiers=None` the CPU-tier section would say there are no
nightly results. Replace that section, the generated-by comment, and the
sentence that every host used the pinned toolchain and ran only when idle,
unless that sentence is true of every host in the sheet.

[render-side.py](render-side.py) does that and writes only under `--out`.
`--out` inside the rind checkout is an error. A JSON whose `status` is not
`ok` is an error. Host order on the command line is the sheet's order. A
note is one paragraph, `--note <host> <file>` or `<stem>.note` beside the
JSON. The script rejects a heading. It exits if `report()` no longer has
the comment, the toolchain sentence, or the `## CPU tiers` and
`## Verified code` headings.

```shell
python3 render-side.py \
  --rind /path/to/rind \
  --out /path/to/results/render \
  --note <host> /path/to/<host>.note \
  /path/to/<host>.json
```

The sheet is `<out>/BENCHMARKS.md`. SVGs are `<out>/benchmarks/`. Gaps come
from `gaps_of` where rind is slower than `SLOWER` (1.1). They are
measurements. `benchmarks/gaps.toml` stays the maintainers' file.
`rind-dit-held` is excluded (`VARIANTS`).

The host note adds what the JSON does not already show: a rustc other than
the pin, or a run outside `nix develop`; a shared or in-use machine;
OpenSSL headers unpacked locally; an aarch64 part with the crypto extension
and without SHA-3; on Apple silicon, that short gaps need the DIT-held
column; that ML-DSA-65 was timed here while the published sheet has dashes
on those three rows.

## Results repository

Commit the JSON and the sheet to a repository other than rind and other
than this skills repository. Layout:

```text
<host>/<host>.json
<host>/<host>.note
<host>/run.log
render/BENCHMARKS.md
render/benchmarks/*.svg
```

Do not commit a target directory, `CARGO_HOME`, a source archive, a
checkout, or `.tmp/`. Do not put a token in a remote URL. Push from an
account that can write the repository.

[stage-repo.py](stage-repo.py) writes that per-host directory from the
JSON, trims a `.log` to timing lines, and copies a rendered sheet from
`--render` (`BENCHMARKS.md` or `STATS.md`, plus `benchmarks/`). It rewrites
home-directory paths and private IPv4 addresses. Pass
`--replace REGEX=TEXT` for a machine name or a board product name.

```shell
python3 stage-repo.py --out /path/to/results-repo \
  --json /path/to/<host>.json --rename <captured>=<published> \
  --render /path/to/render --log <published>=/path/to/run.log
```

A JSON with `status` other than `ok` can sit beside the sheet. `render-side.py`
accepts only `status` `ok`. After a clean `admin/build-cost`, merge that
stdout into `build_cost` as
[rind-bench-hosts](../rind-bench-hosts/SKILL.md) describes, then render
again.

`cargo run -p page-data --locked -- summary.json` prints a compact page
from a summary object with `order`, `columns`, and per-host `rows`,
`cpu`, `os`, `rustc`, `commit`, `started`, `load`, `lead`, `timed`, and
`build_cost`. `render-side.py` does not write that object. The tool
lives in `tools/page-data/` and replaces `page-data.py`.
