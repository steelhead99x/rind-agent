---
name: rind-bench-results
description: Use when rendering a side benchmark sheet for hosts that are not in admin/bench-report's HOSTS, or committing that sheet and the raw JSON to a results repository.
---

# A side sheet, and where the JSON goes

How a host outside the six was measured, including the note you owe the
reader, is in [rind-bench-hosts](../rind-bench-hosts/SKILL.md). This skill
starts from the `<host>.json` files that run wrote.

## Do not render into the rind repo

`admin/bench-report render` writes the checkout's `BENCHMARKS.md` and
`benchmarks/*.svg`, imports only JSON named in `HOSTS`, and calls
`load_gaps`. A gap over 10% with no `benchmarks/gaps.toml` decision fails
the command. A decision whose `on` is not a `HOSTS` name or a nightly tier
fails it too. Adding a side host means editing `HOSTS` and then either
failing that check or writing a maintainer decision for a machine the
maintainers do not run. Do not do that, and do not open a rind pull request
that replaces `BENCHMARKS.md` with these numbers.

`summary` has the same name list. A file called anything else is invisible
to both commands.

`report()` returns the Markdown and the SVGs as strings. It does not write
them. That is the function a side sheet calls, after pointing `HOSTS`,
`REPORT`, and `REPORT_DIR` at the side hosts and a directory outside the
checkout. Pass `tiers=None` and `gaps=None`. `load_verified()` still reads
the checkout's `benchmarks/verified.toml`, so the ✓ marks stay the
published ones. `VERIFIED` is set when the script is imported; changing
`REPORT_DIR` afterwards does not move it.

The nightly CPU-tier section is the maintainer report's. A side sheet that
left `tiers=None` would say there are no nightly results, which is false.
Replace that section. Replace the generated-by comment, so the file does
not claim to be `admin/bench-report render` output. Replace the sentence
that every host used the pinned toolchain and ran only when idle, unless
that is actually true of every host in the sheet.

## Render

[render-side.py](render-side.py) does that replacement and writes only under
`--out`. `--out` inside the rind checkout is an error. A JSON whose
`status` is not `ok` is an error; leave failed runs out of the sheet and
keep them beside it if you still want the record. Host order on the
command line is the sheet's order. A note is a one-paragraph file, passed
with `--note <host> <file>` or placed next to the JSON as `<stem>.note`.
The renderer prints it as that host's note, the same way `HOSTS["note"]`
is printed for `kixtop`.

```shell
python3 render-side.py \
  --rind /path/to/rind \
  --out /path/to/results/render \
  --note <host> /path/to/results/<host>/<host>.note \
  /path/to/results/<host>/<host>.json
```

The sheet lands at `<out>/BENCHMARKS.md`. SVGs land at `<out>/benchmarks/`,
which is the relative path the Markdown already uses. The script exits if
`report()` no longer contains the comment, the toolchain sentence, or the
`## CPU tiers` and `## Verified code` headings it rewrites. It also exits
if `admin/bench-report` has dropped `report`, `load_verified`, `gaps_of`,
`label`, or `COLUMNS`.

Gaps in the sheet come from `gaps_of` on these hosts only. They are
measurements, rind's time over the fastest other library where that ratio
is above `SLOWER` (1.1). Record them as measurements. The maintainers'
decisions stay in `benchmarks/gaps.toml`, which this sheet leaves
untouched. `rind-dit-held` is excluded from that ratio (`VARIANTS`).

The note for each host has to make the sheet honest. The JSON already
holds the CPU, the OS, the commit, the date, and `toolchain.rustc`. The
paragraph adds the conditions a reader would otherwise miss:

- the rustc was not the `flake.nix` pin, or the run was outside
  `nix develop`, if either is true;
- the machine was shared or in use, if it was, including when `load.noisy`
  is false but other work was on it before the run;
- OpenSSL headers were unpacked into the work directory, if they were;
- the board is in the aarch64-with-crypto-but-without-SHA-3 class
  `quest/m2/bench/README.md` names, if it is;
- on Apple silicon, short rind gaps need the `rind (DIT held)` column;
- ML-DSA-65 was timed here. The published sheet has dashes on those three
  rows, so compare the new numbers with another run that timed them.

One paragraph, no heading. The script rejects a note that starts a heading.

## Results repository

Commit the results to their own repository, separate from the rind
checkout and from this skills repository. Layout:

```text
<host>/<host>.json
<host>/<host>.note
<host>/run.log
render/BENCHMARKS.md
render/benchmarks/*.svg
```

The JSON is the measurement. The log is optional. The sheet is what
`render-side.py` wrote; commit it so a reader does not have to regenerate
it, and keep the JSON so they can. Do not commit a target directory, a
`CARGO_HOME`, a source archive, a checkout, or `.tmp/`.

Do not put a token in a remote URL. Push with an account that can write
that repository. If the `gh` login on the machine you measured from is a
different account, commit from a session whose account can write the
repo. Do not fix a permission failure by embedding credentials.

A failed host JSON (`status` other than `ok`, or `build_cost.status`
`failed`) stays next to the successful ones if you want the record of the
refusal. `render-side.py` accepts a JSON only when `status` is `ok`. After a clean
`admin/build-cost` rerun, merge that stdout into the host JSON as
`rind-bench-hosts` describes, then render again and commit the updated
JSON and sheet together.
