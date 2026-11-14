---
name: rind-bench-pin
description: Use when choosing the rind commit a published benchmark set measures, including a fork or patches for a board.
---

# The commit a published set measures

Upstream is [kixel-ai/rind](https://github.com/kixel-ai/rind).
`moq-dev/rind` redirects there. `gh repo view moq-dev/rind` and
`GET /repos/moq-dev/rind` report `full_name` `kixel-ai/rind`. Resolve
that name before the next API call. Old `moq-dev/rind` links still
redirect in a browser.

The comparable set is upstream commit
[`d6ca9a756d3d1e71489e61e44c04f33c3c893589`](https://github.com/kixel-ai/rind/commit/d6ca9a756d3d1e71489e61e44c04f33c3c893589)
(`d6ca9a75`, OPRF). Every new result's `commit` is that full upstream
SHA. It is never a fork SHA.

`0b1f1469c8e17e361e3353432bc7fceee9e04214` is the previous comparable
set. Directories named with that SHA stay as history. Do not overwrite
them, delete them, or mix their numbers into `d6ca9a75` tables.

Any machine can measure this commit. Publish it under a CPU slug.
Who `collect` can reach is `COLLECT_HOSTS`, and that list is separate
from this pin.

How to run that commit is
[rind-bench-measure](../rind-bench-measure/SKILL.md). The JSON shape
and directory are
[rind-bench-schema](../rind-bench-schema/SKILL.md).

## Clean tree

Before configure, build, or measure, `git status --porcelain` in the
rind checkout is empty. Any output is a hard failure. Do not discard
uncommitted work to make it empty. Stop and report the paths.

A patch is allowed only where that upstream commit cannot run:
`no_std`, or Xtensa. Two forms are acceptable.

1. A fork branch whose commits are the upstream pin plus the patches,
   each patch its own commit.
2. The same patches applied and then committed, so the measured `HEAD`
   is a commit.

Record every patch as `name` (the filename) and `sha256` (the hash of
the patch bytes). `upstream_commit` is the pin. `fork_commit` is the
measured `HEAD` when that `HEAD` is not the upstream commit. A
worktree with uncommitted edits has no `fork_commit`.

Benchmark patches, firmware, and result sets stay on `steelhead99x`
repositories. Do not open that work as a pull request or an issue on
kixel-ai/rind. Publishing is
[rind-bench-publish](../rind-bench-publish/SKILL.md).

## What this pin builds

At `d6ca9a75`, `admin/bench-report` sets

```text
CRITERION_ARGS = ["--noplot", "--warm-up-time", "1", "--measurement-time", "3"]
```

and `OPERATIONS` is the 68 ids built from `SECTIONS` in that file.
Read both from the measured commit. Do not reuse a count, a filter, or
a criterion argument list from another checkout.

Upstream rind at this pin does not build `#![no_std]`. A board that
needs `no_std` or Xtensa uses the fork above.
[rind-hardware](https://github.com/steelhead99x/rind-hardware) carries
`patches/0001` through `patches/0005` for that port, plus patches that
are not rind (ring, for example). An optional set lives in its own
directory under `patches/` and is a labelled result directory, not part
of the comparable set. Do not number an optional set `0001` through
`0005`.

## Tables from this commit

Evaluate bench tables from the measured rind commit, not from a
snapshot that was generated against a different SHA. In a
[rind-results](https://github.com/steelhead99x/rind-results) checkout,
with `--rind` pointing at the clean measured checkout:

```shell
cargo xtask bench-tables --rind /path/to/rind
```

`cargo xtask bench` reads operations from the `--rind` checkout it was
given. `bench-tables` only refreshes the snapshot for review. A
`tools/bench-tables.json` left over from another SHA is the wrong
table.
