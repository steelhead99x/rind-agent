---
name: rind-bench-publish
description: Use when publishing a measured rind benchmark set, including the verify step and the pull request that carries it.
---

# Publishing a measured set

The tree is the one
[rind-bench-schema](../rind-bench-schema/SKILL.md) describes, filled by
[rind-bench-measure](../rind-bench-measure/SKILL.md) at the commit in
[rind-bench-pin](../rind-bench-pin/SKILL.md).

## Verify, then publish

From a [rind-results](https://github.com/steelhead99x/rind-results)
checkout, after `cargo xtask report --dir <out>` has written the
directory:

```shell
cargo xtask verify --dir <out>
```

`<out>` is any directory. The command must succeed on a tree that is
not already under `results/`. A non-zero exit, or help text with no
`--dir`, means the set is not published. Do not commit it while verify
is red.

`cargo verify` with no `--dir` only checks boards already stored in
that repository (`esp32s3`, `nrf52840`, `msi-001`, and a host that
already has `results/<host>/raw/`). It is not this check.

Read the report back from `<out>` before pushing. Every gap where
another library is more than 10% faster than rind (rind's `SLOWER` of
1.1, OpenSSL included) has a reason in that directory's `gaps.toml`.
A decision that covers no gap is an error.

For a board, reflash the images you are publishing, check each image's
sha256, and run again. The new medians land within the larger of 1%
and the recorded run spread. A larger move is a different set. Ask
before that flash, as rind-bench-measure requires.

## Where the pull request goes

Open the pull request on a `steelhead99x` repository: rind-results for
the JSON and the sheet, rind-hardware for firmware. Do not open a pull
request or an issue on kixel-ai/rind for a benchmark patch, a firmware
port, or a result set. Quest work that is not a benchmark follows
[fork-to-upstream](../fork-to-upstream/SKILL.md).

Do not commit a target directory, `CARGO_HOME`, a source archive, a
checkout, `.tmp`, or a firmware image that is not the published
artifact the results tree already tracks. Do not put a token in a
remote URL.

Search the tree you are about to commit for a home-directory path, a
`~/` path, a private IPv4 address, a token, a MAC address (six hex
bytes separated by `:` or `-`), and a `by-id` serial
(`/dev/serial/by-id/` or a `usb-` product string with a serial or MAC
suffix). Keep a stable board label, and replace the serial or MAC
suffix with `<usb-serial>` before commit. Rustc backtraces and cargo
compile lines are not board identifiers.

`NOTES.md` names the upstream SHA, the fork SHA and patch hashes when
there is a fork, the hardware SHA, the host or board, the clock, and
the date. A display or demo build that embeds the table is regenerated
from this tree before anyone flashes it.

The board-port checklist that used to be proposed as its own skill is
this file plus rind-bench-measure and rind-bench-pin. Do not add a
second skill for the same port.
