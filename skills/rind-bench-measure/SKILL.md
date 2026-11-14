---
name: rind-bench-measure
description: Use when running a published rind benchmark set on a host, an Android device, or a board.
---

# Running a published set

Pick the commit in [rind-bench-pin](../rind-bench-pin/SKILL.md) first.
The output directory is
[rind-bench-schema](../rind-bench-schema/SKILL.md). Publish only after
[rind-bench-publish](../rind-bench-publish/SKILL.md).

Every command below is `cargo xtask` in a checkout of
[rind-results](https://github.com/steelhead99x/rind-results) that
contains the `bench` subcommand. The Python files next to the older
skills are pointers and exit without measuring. Host notes that are
not a command (disk, OpenSSL headers, DIT, ML-DSA) stay in
[rind-bench-hosts](../rind-bench-hosts/SKILL.md).

## The comparable set

```shell
cargo xtask bench --board <id> --rind /path/to/rind --commit <pin> --runs 3
cargo xtask report --dir <dir>
cargo xtask verify --dir <dir>
```

`<pin>` is the full upstream SHA from rind-bench-pin. `--commit`
defaults to that pin and stays the upstream SHA when `--rind` is a
fork of it. `--runs` defaults to 3. `<dir>` defaults to
`results/<id>/<pin>/`. Add `--label <label>` for an A/B or a
diagnostic (`results/<id>/<pin>-<label>/`).

`<id>` is one of `xeon-vm`, `m1pro`, `pi5`, `msi-001`,
`jetson-orin-nano`, `pixel9pro`, `esp32s3`, `nrf52840`, `efr32mg24`,
`rp2040`. There is no Pi 3 id. `pi-run` measures `pi5` only. Do not
point a Pi 3 at it.

`git status --porcelain` on the rind checkout is empty before this
runs. `bench` refuses a dirty tree and refuses the previous pin's
directory. It calls `admin/bench-report` with that file's
`CRITERION_ARGS` (`--noplot --warm-up-time 1 --measurement-time 3`)
and does not edit the file. A published set does not use a bare
`cargo bench`.

If `cargo xtask bench --help` does not list `--board`, this checkout
is older than that entry point. Stop. Do not time the set with
`host-run`, a shell script, or `python3`.

`ns` is the median of the three runs' estimates.

Do not pass `--accept-toolchain`, `--record-busy`, `--skip-build-cost`,
or `--busy-share` on a comparable set. Those flags are written to
`relaxations.json`. On `jetson-orin-nano`, pass `--strict` as well:
without it, `bench` accepts the local toolchain, records a busy host,
and skips build-cost.

## Jetson Orin Nano

Comparable set:

```shell
cargo xtask bench --board jetson-orin-nano --rind /path/to/rind --commit <pin> --runs 3 --strict
```

`orin-setup`, `orin-run <n> [core]`, and `orin-runs` are the on-board
collectors. `orin-runs` is three runs of `admin/bench-report` and
always accepts the local toolchain, records a busy host, and skips
build-cost. A tree from `orin-runs` is not the comparable set.

## Mac, the M1 Pro

`mac-run` defaults `--commit` to the pin. It accepts the host rustc
and records a busy host, and it stops when free space is under 5 GiB.
The comparable set is `bench --board m1pro` with neither
`--accept-toolchain` nor `--record-busy`.

## Pi 5

`pi-run` defaults `--commit` to the pin and records host id `pi5`.
When pkg-config cannot see OpenSSL it unpacks `libssl-dev` into the
work directory and does not install it. It accepts the host rustc and
records a busy host. The comparable set is `bench --board pi5` without
those flags. A Pi 3 has no runner here.

## MSI

`bench --board msi-001`. There is no separate MSI command. The older
`results/msi-001/` tree with no upstream SHA in the path is history.

## Pixel 9 Pro

Collect on the phone first. `--serial` is required:

```shell
cargo xtask pixel-run --serial <id>
```

The rustls second pass is `pixel-run2 --serial <id>`. That pass is a
labelled directory, not the comparable set. Publish the capture with:

```shell
cargo xtask bench --board pixel9pro --rind /path/to/rind --commit <pin> --capture <dir> --runs 3
```

`bench` refuses `pixel9pro` when `--capture` is missing. The capture
holds `getprop.txt` (only the properties
[rind-bench-hosts](../rind-bench-hosts/SKILL.md) records, never a full
`getprop` dump), `stat-before`, `stat-after`, `criterion/`, and the
battery dumps from that skill. Parsing matches `admin/bench-report run`.

## ESP32-S3 and nRF52840

Ask the person at the machine before any flash, erase, or unlock.
Name the board and the image. Wait for an explicit yes. Building does
not flash.

After the yes, ESP32-S3:

```shell
cargo xtask bench --board esp32s3 --rind /path/to/rind --commit <pin> --images <dir> --port <port> --runs 3
```

`--port` is required. Pass `--flash-freq 80mhz` on the espflash
invocation this wraps, and confirm `Boot SPI Speed` in every run log.
A 40 MHz log is a labelled historical set. `run-all` and `capture`
also flash. Do not run them before the yes.

nRF52840, after the yes:

```shell
cargo xtask bench --board nrf52840 --rind /path/to/rind --commit <pin> --images <dir> --runs 3
```

That path uses the UF2 runner. Read WDT `RUNSTATUS`, `CRV`, and `RREN`
at boot on a part that just left a vendor firmware, and reload every
enabled RR register between timed regions. Do not copy metadata onto
a UF2 drive.

## EFR32MG24 and RP2040

`bench` does not flash these boards and does not reuse the nRF
runner. Flash with that board's own runner after a yes, then:

```shell
cargo xtask report --dir <dir>
cargo xtask verify --dir <dir>
```

Backup, the bootloader span, and the compile-only matrix are in
[rind-bench-baremetal](../rind-bench-baremetal/SKILL.md). `probe-rs
run` programs an EFR32MG24 from `0x08000000` and overwrites the
bootloader.

## What every board run keeps

Correctness first: known-answer checks pass before any timed sample.
A library that fails to build is recorded with the compiler error.
Time the same keys, nonces, sizes, core, and clock. On the board, time
with the cycle counter and report the median, the spread, ns/op,
cycles/op, and cycles/byte. The empty-call cost is reported beside the
fastest row and is not subtracted. No print or display update sits
inside the timed loop.

Run three times. Relink at least once and publish the per-row shift.
Padding sits ahead of the code under test. A doubled instruction
cache is a labelled directory. The published ESP32-S3 cache is the
16 KB default.

Xtensa: a local aligned past 16 bytes can make LLVM realign the stack
without `movsp`. ring's `poly1305_state_st` is the known case. The
firmware patch keeps that state 16-byte aligned.

On macOS, if espflash reports no serial data on the USB-Serial/JTAG
port, `probe-rs reset --chip esp32s3` comes before another attempt,
and only after the flash was approved. Build `esp-idf-sys` before
ring's C files on a fresh rind-hardware checkout.

## Not a comparable set

- `cargo xtask host-run`, including `--busy-share`. One run of `HEAD`.
  The idle limit changes in memory. The file is not edited.
- `orin-run`, `orin-runs`, `mac-run`, and `pi-run` when they accept
  the host toolchain or record a busy host.
- `python3` `scripts/host-run.py`, `scripts/android-convert.py`,
  `scripts/render-ours.py`, `scripts/orin-run.sh`, `scripts/pixel-run.sh`,
  `scripts/mac-run.sh`, or `scripts/pi-run.sh`. The rind-results stack
  replaces those with the subcommands above.
- A bare `cargo bench`, or any edit of `admin/bench-report`.

Regenerating a tree already published under its old path uses the
command that tree's `RESULTS.md` names. That does not start a
comparable set.
