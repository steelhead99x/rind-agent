---
name: rind-bench-measure
description: Use when running a published rind benchmark set on a host, an Android device, or a board.
---

# Running a published set

Pick the commit in [rind-bench-pin](../rind-bench-pin/SKILL.md) first.
The output directory is
[rind-bench-schema](../rind-bench-schema/SKILL.md). Publish only after
[rind-bench-publish](../rind-bench-publish/SKILL.md).

All of the commands below are run from a checkout of
[rind-results](https://github.com/steelhead99x/rind-results). The
helpers that used to sit next to these skills are pointers and exit
without measuring. See
[rind-bench-hosts](../rind-bench-hosts/SKILL.md) and
[rind-bench-results](../rind-bench-results/SKILL.md).

## The entry point

```shell
cargo xtask bench --board <id> --rind /path/to/rind --commit <pin> --out <dir> --runs 3
cargo xtask report --dir <dir>
cargo xtask verify --dir <dir>
```

`<pin>` is the full upstream SHA. `--commit` stays that SHA when the
checkout at `--rind` is a fork of it. `--runs` defaults to 3. `<id>`
is the published host or board id (`esp32s3`, `nrf52840`, or a host
name). `<dir>` is `results/<id>/<pin>/`, or
`results/<id>/<pin>-<label>/` for an A/B or a diagnostic.

Before the first run, `cargo xtask bench --help` shows `--board`,
`--rind`, `--commit`, `--out`, and `--runs`, and `cargo xtask report
--help` and `cargo xtask verify --help` show `--dir`. If a subcommand
is missing, stop. Do not start a comparable set with a substitute.

These commands do not edit `admin/bench-report`. They pass that file's
`CRITERION_ARGS` (`--noplot --warm-up-time 1 --measurement-time 3`) and
its `OPERATIONS` from the measured commit. A host or an Android phone
is that `run`, three times. A published set does not use a bare
`cargo bench`.

Regenerating a tree already in rind-results, under the path it was
published at, uses the command that tree's `RESULTS.md` names. That
regenerates history. It does not start a comparable set.

## Hosts and Android

`git status --porcelain` on the rind checkout is empty. Ship that
commit with `git archive` the way `admin/bench-report collect` does,
or build in the clean checkout itself. Do not copy a target directory
or a cargo registry.

One speed run at a time on a machine. `run` calls `admin/build-cost`
after the benchmarks. Disk, OpenSSL headers, the idle refusal, the
toolchain pin, Apple DIT, and ML-DSA are in
[rind-bench-hosts](../rind-bench-hosts/SKILL.md). A host that `run`
refuses as busy is not part of the comparable set. Record it only as
a labelled diagnostic, or do not record it.

An Android capture uses the same operations, the same criterion
arguments, and schema 1. Save the capture described in
rind-bench-hosts, then let `cargo xtask bench` write the JSON. Do not
publish a JSON assembled by the old converter.

`ns` is the median of the three runs' estimates. Three runs is the
published count.

## Boards

Ask the person at the machine before any flash, erase, or unlock.
Name the board and the image. Wait for an explicit yes. Building does
not flash.

[rind-hardware](https://github.com/steelhead99x/rind-hardware) already
builds and runs ESP32-S3 and nRF52840 firmware, and it builds RP2040.
Upstream rind at the pin still has no `#![no_std]` build; the firmware
is the fork plus the committed patches in rind-bench-pin. Backup and
the compile-only matrix are in
[rind-bench-baremetal](../rind-bench-baremetal/SKILL.md). Read the
flash at least twice and match the sha256 before you overwrite a
firmware image you do not own. Flash only the application region, and
check that the bootloader and SoftDevice are unchanged.

Until the yes, do not run rind-results `cargo xtask run-all`,
`run-all-nrf`, `capture`, or `capture-nrf`, and do not run
`espflash`, `probe-rs download`, or `probe-rs run`. `probe-rs run`
programs from `0x08000000` and overwrites an EFR32MG24 bootloader.

Correctness first: known-answer checks pass, and the libraries that
are raced agree, before any timed sample. A library that fails to
build is recorded with the compiler error, not dropped quietly. Time
the same keys, nonces, sizes, core, and clock. Keep radios and
unrelated interrupts off. On the board, time with the cycle counter,
many samples, and report the median, the spread, ns/op, cycles/op,
and cycles/byte. The empty-call cost is reported beside the fastest
row and is not subtracted. No print, display update, or log line sits
inside the timed loop. Start the run from an explicit host go line.

ESP32-S3: pass `--flash-freq 80mhz` and confirm the bootloader's
`Boot SPI Speed` line in every run log. espflash writes that clock
into the image header and otherwise leaves 40 MHz. A 40 MHz log is a
labelled historical set, not the comparable set. The published
instruction cache is the chip default (16 KB). A larger cache is a
diagnostic label.

nRF52 parts that just left a vendor firmware can still have the
hardware watchdog running. Read WDT `RUNSTATUS`, `CRV`, and `RREN` at
boot, log them, and reload every enabled RR register between timed
regions and while idle. Do not copy metadata onto a UF2 drive.

Xtensa: a local whose alignment is greater than 16 can make LLVM
realign the stack without `movsp` and break the register windows.
ring's `poly1305_state_st` is the known case; the firmware patch keeps
that state 16-byte aligned. Confirm the image on hardware.

Run the suite three times. Relink at least once and publish the
per-row shift. Padding has to sit ahead of the code under test; check
with `nm -n` which library moved. A relink of a few bytes is labelled
as that. Do not claim the times are immune to layout. A doubled
instruction cache, a padding sweep, or a second RNG stays in a
labelled directory.

On macOS, if espflash or esptool reports no serial data on the
USB-Serial/JTAG port, `probe-rs reset --chip esp32s3` on that probe
comes before another flash attempt, and only after the flash was
approved. On a fresh rind-hardware checkout, build `esp-idf-sys`
before ring's C build so the Xtensa GCC is installed. Use the pinned
rustfmt, not an auto-installed toolchain.

## Substitutes that are not a published set

Stop, rather than falling back, when `cargo xtask bench` is absent.

- `cargo xtask host-run` fakes the dev-shell check and times `HEAD`
  once. `--busy-share` rewrites `BUSY_SHARE` inside `admin/bench-report`
  and restores the file afterwards.
- `cargo xtask report --board esp32s3` (and `nrf52840`) writes the
  pre-unification tree. It has no `--dir`. Use it only to regenerate
  that historical tree.
- `cargo verify` without `--dir` checks stored boards, not an
  arbitrary directory.
- `python3` `scripts/host-run.py`, `scripts/android-convert.py`, or
  `scripts/render-ours.py` in rind-results, and the pointer files in
  this repo.
- A bare `cargo bench`, or any in-place edit of `admin/bench-report`.
