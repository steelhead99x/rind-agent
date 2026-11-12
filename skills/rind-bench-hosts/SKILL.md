---
name: rind-bench-hosts
description: Use when measuring rind's benchmark report on a host outside admin/bench-report's six, including a shared machine, a Mac, a board reached through another machine, or an Android phone.
---

# Measuring rind on your own hosts

`admin/bench-report run` is the host side. `collect` accepts only the six
names in `HOSTS`. For a side sheet, use
[rind-bench-results](../rind-bench-results/SKILL.md). Command details are in
[rind-bench-and-verify](../rind-bench-and-verify/SKILL.md).

One speed run per machine. `run` calls `admin/build-cost` after the
benchmarks. Other heavy builds on that machine wait. Separate machines can
run together.

## Command

`nix develop` sets `IN_RIND_DEV_SHELL=1`.

```shell
nix develop --command python3 admin/bench-report run \
  --host <name> --commit <full-sha> --jobs <n> \
  --target-dir <work>/target --out <work>/<name>.json
```

`<name>` is recorded. It is not looked up in `HOSTS`. The report uses 3
jobs on its shared 15 GB host and 4 to 16 on the others. Cap jobs to what
this machine can compile without swapping.

`run` builds `rind-bench` with `--features __bench_openssl` and times
`OPERATIONS` (62 at the commit the README cites) with `CRITERION_ARGS`.
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

## Off the dev shell

[host-run.py](host-run.py) replaces `toolchain()` and `check_idle()`, so it
skips the dev-shell check and the pin check. Busy counts still land in
`load`, and `run` still sets `load.noisy`. Stderr prints
`host-run: N of M CPUs busy <when> (harness limit L); continuing`. A missing
name exits before measuring.

Use it when the host will not go idle, or when the dev shell will not fit
on the disk. Record `toolchain.rustc` in the host note when it is not the
pin.

```shell
RIND_ROOT=/path/to/rind python3 host-run.py \
  --host <name> --commit <full-sha> --jobs <n> \
  --target-dir <work>/target --out <work>/<name>.json
```

Omit `RIND_ROOT` when the script is in the checkout root.
`HOSTRUN_FREE_TARGET=1` deletes `CARGO_TARGET_DIR` before build-cost.
`build_env` drops `CARGO_TARGET_*`, so build-cost uses
`<checkout>/.tmp/build-cost`. `HOSTRUN_SKIP_BUILD_COST=1` stores
`build_cost.status` `failed` with reason
`skipped by host-run (HOSTRUN_SKIP_BUILD_COST)`.

Copy the wrapper with the archive. Do not copy a target directory or a
cargo registry back.

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
`cc` on `PATH`. Rerun when under the 25% line:

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
pkg-config. Record `rustc` when it is not the pin.
`rind-bench/benches/mldsa.rs` skips OpenSSL ML-DSA when the linked
OpenSSL is older than 3.5.

## Another machine

From the machine that already has a key: `ssh -o BatchMode=yes`. On
failure, stop. Do not try a password. Some images have no default account.

To identify a board you own, read `ip neigh` or `arp -a` and match the
vendor MAC prefix.

Copy the archive and `host-run.py`. One `nohup` job, log in the work
directory, copy the JSON back, delete the target directory there. The hop
machine can run its own suite.

## Android

Cross-compile `rind-bench` for `aarch64-linux-android` with the NDK. Push
with `adb`, run under `/data/local/tmp`, then delete that directory.

Save `dumpsys battery` as `battery-before.txt` and `battery-after.txt`,
`/proc/stat` as `stat-before` and `stat-after`, `getprop` as `getprop.txt`,
the criterion tree, `cpuid.json` from a same-target `rind-cpuid`, and
ISO-8601 `started` and `finished`.

[android-convert.py](android-convert.py) imports `admin/bench-report` and
parses criterion as `run()` does. Pass `--host`, `--commit`, the cross
`rustc` and `cargo` lines, `--cpu-model`, `--logical-cpus`, and
`--openssl-linked` (or `openssl-linked.txt`). One capture linked vendored
static OpenSSL.

`load.system_busy_share_during_run` includes the benchmarks, so
`load.noisy` is null. `build_cost.status` is `failed` because the binaries
were cross-compiled. `dumpsys` temperature is tenths of a degree; the
converter stores Celsius.

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
