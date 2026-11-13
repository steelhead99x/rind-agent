---
name: rind-bench-hosts
description: Use when measuring rind on a host outside admin/bench-report's six, after rind-bench-measure, including a shared machine, a Mac, or an Android phone.
---

# Measuring rind on your own hosts

A published set follows
[rind-bench-pin](../rind-bench-pin/SKILL.md),
[rind-bench-measure](../rind-bench-measure/SKILL.md),
[rind-bench-schema](../rind-bench-schema/SKILL.md), and
[rind-bench-publish](../rind-bench-publish/SKILL.md). The command is
rind-results `cargo xtask bench`. The notes below are what
`admin/bench-report run` does when that command calls it, and what the
host note should add.

`collect` accepts only the six names in `HOSTS`. Upstream's sheet for
those six is
[rind-bench-and-verify](../rind-bench-and-verify/SKILL.md). A side tree
does not rewrite rind's `BENCHMARKS.md`. Where that JSON goes is
[rind-bench-results](../rind-bench-results/SKILL.md).

One speed run per machine. `run` calls `admin/build-cost` after the
benchmarks. Other heavy builds on that machine wait. Separate machines can
run together.

## What `run` does

`nix develop` sets `IN_RIND_DEV_SHELL=1`. A published set does not
launch the next command by hand and does not edit the script first.
`cargo xtask bench` runs it. Inside the measured checkout the
invocation is:

```shell
nix develop --command python3 admin/bench-report run \
  --host <name> --commit <full-sha> --jobs <n> \
  --target-dir <work>/target --out <work>/<name>.json
```

`<name>` is recorded. It is not looked up in `HOSTS`. The report uses 3
jobs on its shared 15 GB host and 4 to 16 on the others. Cap jobs to what
this machine can compile without swapping.

`run` builds `rind-bench` with `--features __bench_openssl` and times
`OPERATIONS` from the measured commit with `CRITERION_ARGS`. The count
at the comparable-set pin is in
[rind-bench-pin](../rind-bench-pin/SKILL.md). An older count of 62
belongs to `7ab9f85`, the commit the rest of
[rind-bench-and-verify](../rind-bench-and-verify/SKILL.md) was checked
against.
Python must be 3.11 or newer (`tomllib`). The dev shell provides
`pkg-config` and OpenSSL.

Ship the commit with `git archive --format=tar`, as `collect` does.
Submodule contents are omitted. Unpack into a directory you can delete.

## Refusals

1. `check_idle` samples 5 seconds and refuses when busy logical CPUs exceed
   `max(1, 0.1 * logical CPUs)`. On 8 CPUs the limit is 1.0. One shared
   8-vCPU VM at 2.3 was refused. The JSON is `status` `busy`.
2. `toolchain()` requires `IN_RIND_DEV_SHELL=1` and a `rustc -vV` release
   equal to `flake.nix`'s `rust-bin.stable` pin (`1.98.1` at the commit the
   README cites). `fail()` writes no file. The text is `bench-report: `
   plus `run inside the dev shell: ...` or `rustc <version> is not the pinned <pin>`.
3. `check_idle` runs again before the benchmarks. A refusal writes
   `status` `busy` with the toolchain fields filled in.

A failed `cargo` invocation raises and writes no file. `admin/build-cost`
then applies its own 25% check.

## Idle and the dev shell

`run` refuses a busy host and a rustc that is not the flake pin. See
Refusals below. A published set does not patch those checks.
[host-run.py](host-run.py) is a pointer and exits before measuring.
rind-results `cargo xtask host-run` fakes the dev-shell check, times
`HEAD`, and runs once. `--busy-share` changes the idle limit in memory
and does not edit `admin/bench-report`. Neither form is a published
set. The machine commands are in
[rind-bench-measure](../rind-bench-measure/SKILL.md). A host that `run` refuses is left out of the comparable
set. A diagnostic run goes in a labelled directory, as
[rind-bench-schema](../rind-bench-schema/SKILL.md) describes.

Do not copy a target directory or a cargo registry to another machine.

## Disk

Check free space on the work directory's filesystem before `nix develop`
and before `run`. Start at 5 GiB or more. Stop and delete the target
directory under 1.5 GiB. Keep the checkout, `CARGO_HOME`, and
`--target-dir` in that directory.

Those floors are from one run, not from the rind scripts: the release
target was about 2 GB, the crate cache about 220 MB, and a disk with about
6 GB free filled up. Vendored OpenSSL is the large build-cost fixture.

Linux: `df -B1 --output=avail <work>`. macOS: `df -k <work>` (1024-byte
blocks). After a successful run, delete the target directory,
`CARGO_HOME`, and `<checkout>/.tmp`. Keep the JSON.

## Build cost

`admin/build-cost` cold-builds (`cargo build --release --frozen`, three
runs) each fixture in `admin/build-cost-fixtures`: `rind`, `aws-lc-rs`,
`ring`, `rustcrypto`, `graviola`, `libcrux`, `openssl`,
`openssl-vendored`, `rustls-rind`, `rustls-aws-lc-rs`, `rustls-ring`. The
JSON has wall time, CPU time, `stripped_bytes`, tools beyond `rustc` and a
linker, and the crate list. The sheet prints decimal MB
(`stripped_bytes / 1e6`).

It exits `error: host is busy (N of M CPUs busy)` when `busy_cpus()`
exceeds `0.25 * cpus()`. On Linux the sample is the next 5 seconds of
`/proc/stat` scaled by `os.cpu_count()`, and `cpus()` is
`len(os.sched_getaffinity(0))`. On macOS `busy_cpus()` is the 1-minute
load average, so N in the message is that average, and a run started
immediately can fail on an idle Mac. `run` stores a non-zero exit as
`build_cost.status` `failed` and keeps the speed numbers. The published
`nixos` row is this failure. Its stored reason still says
`load average ...`. The script at the cited commit prints
`N of M CPUs busy`.

The script does not check the dev shell or the pin. It records `rustc` and
`cc` on `PATH`. A published set keeps the `build_cost` object that `run`
wrote. For an older JSON whose `build_cost.status` is `failed`, rerun
when under the 25% line:

```shell
nix develop --command python3 admin/build-cost
```

Stdout is a new object. To show it, set the host's `build_cost` to that
object plus `"status": "ok"` and `"started"`.

## OpenSSL headers without root

`run` calls `openssl version` and stores it at `libraries.openssl.linked`.
A missing CLI fails the run. On a Debian-family board with the library and
the CLI but no headers and no root:

1. Stop if `pkg-config --exists openssl` succeeds.
2. `apt-get download libssl-dev` and `dpkg -x` into the work directory.
   Do not install it.
3. Copy `usr/include/<triplet>/openssl/*.h` (`gcc -print-multiarch`) into
   `usr/include/openssl/`.
4. Point `OPENSSL_LIB_DIR` at `libssl.so` and `libcrypto.so` symlinks to
   the system's `libssl.so.3` and `libcrypto.so.3`. Point
   `OPENSSL_INCLUDE_DIR` at the unpacked `usr/include`. `openssl-sys` 0.9
   skips pkg-config when both are set.

On Apple silicon, `openssl-sys` looks under `/opt/homebrew` before
pkg-config. A published set uses the flake's pinned rustc. A historical
JSON that recorded another rustc says so in the host note.
`rind-bench/benches/mldsa.rs` skips OpenSSL ML-DSA when the linked
OpenSSL is older than 3.5.

## Another machine

From the machine that already has a key: `ssh -o BatchMode=yes`. On
failure, stop. Do not try a password. Some images have no default account.

To identify a board you own, read `ip neigh` or `arp -a` and match the
vendor MAC prefix.

Copy a `git archive` of the measured commit. Run `cargo xtask bench`
from a rind-results checkout on the machine that will measure. One job,
log in the work directory, bring back the output directory, and delete
the target directory there. The hop machine can run its own suite.

## Android

The phone does not run `admin/bench-report`. Collect with
`cargo xtask pixel-run --serial <id>` (`--serial` is required), then
publish with `cargo xtask bench --board pixel9pro --capture <dir>`, as
[rind-bench-measure](../rind-bench-measure/SKILL.md) specifies. The
capture is a cross-compiled `rind-bench` for `aarch64-linux-android`,
run under `/data/local/tmp`, then deleted.

Save `dumpsys battery` as `battery-before.txt` and `battery-after.txt`,
`/proc/stat` as `stat-before` and `stat-after`, `getprop` as `getprop.txt`,
the criterion tree, `cpuid.json` from a same-target `rind-cpuid`, and
ISO-8601 `started` and `finished`.

[android-convert.py](android-convert.py) is a pointer and exits before
converting. `cargo xtask bench` writes the published JSON from that
capture, with the same operations, criterion arguments, and schema 1 as
`run`. JSON already produced by the old converter has
`load.system_busy_share_during_run` covering the benchmarks, so
`load.noisy` is null, and `build_cost.status` `failed` because the
binaries were cross-compiled. `dumpsys` temperature is tenths of a
degree; that converter stored Celsius. A new set does not copy those
sentinels. One historical capture linked vendored static OpenSSL.

## Noisy

`load.noisy` is set when other CPUs during the run, after subtracting the
benchmark process, exceed `busy_limit()`. `summary` prints `(noisy)`.
`measured()` adds `, noisy`. The published `nixos` row is that mark.

Say in the host note when the machine was shared, including when `noisy`
is false and the pre-check counts were high. A small gap on a noisy host
is not a finding. The nightly `bench` workflow is the clean comparison.

## Apple silicon

`rind_rows()` in `rind-bench/benches/criterion/mod.rs` adds `rind-dit-held`
only for Apple `aarch64` with DIT detected, holding `rind::cpu::Dit::hold()`
across the timed loop. `VARIANTS` keeps that column out of the
fastest-library and gap calculations. The published `kixtop` note
attributes short-input gaps to per-call DIT. Read the DIT-held column
before calling a short-input gap a regression.

## ML-DSA

Published `BENCHMARKS.md` marks ML-DSA-65 keygen, sign, and verify not
timed on every `HOSTS` entry. `OPERATIONS` includes those three, and `run`
times them. `mldsa44` and `mldsa87` are outside `OPERATIONS`. Compare the
three rows with another run that timed them.

## aarch64 without SHA-3

`quest/m2/bench/README.md` says no report host and no forced tier times
aarch64 with the crypto extension and without SHA-3. Name the chip. The
published aarch64 column is a different configuration.

## Time

`CRITERION_ARGS` is shortened so a host takes about half an hour. One
Cortex-A76 board took about 70 minutes of criterion. Build-cost on that
run added about 10 to 30 minutes per host. Those longer figures are from
that run.
