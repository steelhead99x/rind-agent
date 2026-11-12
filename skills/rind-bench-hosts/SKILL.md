---
name: rind-bench-hosts
description: Use when measuring rind's benchmark report on a host that is not one of admin/bench-report's six, including a shared machine, a Mac, or a board reached through another machine.
---

# Measuring rind on your own hosts

The report's host side is `admin/bench-report run`. `collect` will not start
it on any name but the six in `HOSTS`. What those commands do, and why
`render` is the wrong tool for a new host, is in
[rind-bench-and-verify](../rind-bench-and-verify/SKILL.md). Putting the JSON
into a sheet without editing rind's `BENCHMARKS.md` is in
[rind-bench-results](../rind-bench-results/SKILL.md).

Run one speed measurement at a time on a machine. `run` calls
`admin/build-cost` only after the benchmarks, so the two do not share the
CPUs. Another heavy build on that same machine belongs either before the
whole run or after it. Two machines that do not share CPUs can run at the
same time. `collect` itself is one host after another so those hosts do not
share a machine or a network.

## The command

From a checkout of the commit you are measuring. `IN_RIND_DEV_SHELL=1` is
set on the default dev shell, `nix develop`:

```shell
nix develop --command python3 admin/bench-report run \
  --host <name> --commit <full-sha> --jobs <n> \
  --target-dir <work>/target --out <work>/<name>.json
```

`<name>` is a short token. It is recorded; `run` does not look it up in
`HOSTS`. `<jobs>` is the build's job cap. The report uses 3 on its shared
15 GB host because more jobs run that machine out of memory, and 4 to 16 on
the others. Cap it to what this machine can compile without swapping.

`run` builds `rind-bench` with `--features __bench_openssl`, times
`OPERATIONS` through the criterion filter in `operation_regex`, and stores
one JSON object with `schema` 1. The script imports `tomllib`, so
`python3` has to be 3.11 or newer; in the dev shell that is the shell's
Python. That shell also provides `pkg-config` and OpenSSL, which the
feature's build links.

Ship the commit the way `collect` does, with `git archive --format=tar`.
The archive does not contain submodule contents (`thirdparty/s2n-bignum`,
`thirdparty/wycheproof`, `thirdparty/cavp`, `thirdparty/cctv`). The report
benches and `admin/build-cost-fixtures` are files in the main tree. Unpack
the archive into a directory you can delete. See [Disk space](#disk-space).

## When run refuses

`run` stops in three places, in this order. A failed `cargo` build raises
and leaves no result file. The idle check is the path that writes one,
with `status` `busy`.

1. Before the build, `check_idle` samples for 5 seconds. It refuses when
   the average busy logical CPUs are above `max(1, 0.1 * logical CPUs)`.
   On 8 CPUs the floor makes that limit 1.0. In one run a shared 8-vCPU
   VM sat at 2.3 and was refused. The JSON has `status` `busy` and the
   reason from `Busy`.
2. `toolchain()` then requires `IN_RIND_DEV_SHELL=1`, and a `rustc -vV`
   `release:` line equal to the `rust-bin.stable` version `flake.nix`
   pins. At the commit the README cites, that version is `1.98.1`. If
   either check fails, `fail()` runs and writes no result file. The process
   exits `bench-report: ` plus
   `run inside the dev shell: nix develop --command admin/bench-report run ...`
   or `rustc <version> is not the pinned <pin>`.
3. Before the benchmarks, `check_idle` runs again. A refusal here also
   writes `status` `busy`, after the toolchain fields are already filled in.

`admin/build-cost`, which `run` starts after a successful benchmark, has its
own 25% threshold. A benchmark that passed can still record
`build_cost.status` `failed`. See [Build cost](#build-cost-and-binary-size).

## A host that cannot take the dev shell

[host-run.py](host-run.py) loads `admin/bench-report` and replaces
`toolchain()` and `check_idle()`. Replacing `toolchain()` skips both the
dev-shell check and the pin check. The busy-CPU count is still returned, so
`load.busy_cpus_before_build` and `load.busy_cpus_before_run` are filled in,
and `load.noisy` is still computed by `run`. The script prints
`host-run: N of M CPUs busy <when> (harness limit L); continuing` on stderr.
If `admin/bench-report` no longer has those names, the wrapper exits before
measuring.

Use it only when the real checks cannot be met: a shared machine that will
not go idle, or a disk too full for `nix develop` to fetch the pinned
toolchain and the rest of the default shell. Say so in the host note, and
quote `toolchain.rustc` from the JSON when it is not the pin. The wrapper
does not prove the pin or the idle check, so a result from it is not
automatically the published sheet's pinned-toolchain, idle-host setup.

```shell
RIND_ROOT=/path/to/rind python3 host-run.py \
  --host <name> --commit <full-sha> --jobs <n> \
  --target-dir <work>/target --out <work>/<name>.json
```

`RIND_ROOT` may be omitted when `host-run.py` sits in the checkout root.
`HOSTRUN_FREE_TARGET=1` deletes the benchmark target directory before
build-cost, which is what a tight disk needs: build-cost does not reuse
that directory. `HOSTRUN_SKIP_BUILD_COST=1` records build cost as failed
with reason `skipped by host-run (HOSTRUN_SKIP_BUILD_COST)`, for a later
run of `admin/build-cost` on its own.

Copy the wrapper to the host with the archive. Do not copy a target
directory or a cargo registry back.

## Disk space

Check free space on the filesystem that will hold the work directory before
`nix develop`, and again before `run`. The dev shell downloads the pinned
toolchain and its other packages. Start only with at least 5 GiB free. Keep
the unpacked checkout, `CARGO_HOME`, and `--target-dir` inside one directory
you can remove. If free space falls under 1.5 GiB, stop that run and delete
the target directory rather than filling the disk.

Those two floors are from one multi-host run. The rind scripts do not state
them. On that run the `rind-bench` release target directory was about 2 GB and the
crate cache about 220 MB, and a filesystem that started with about 6 GB free
filled up before the benchmarks finished. `admin/build-cost` then does its
own cold release builds under `<checkout>/.tmp/build-cost`. Vendored
OpenSSL is the large one. `build_env` in `admin/build-cost` drops
`CARGO_TARGET_*`, so pointing `--target-dir` elsewhere does not move that
scratch.

On Linux, `df -B1 --output=avail <work>` is the free-byte count. On macOS,
`df -k <work>` reports 1024-byte blocks. After a successful run, delete the
target directory, `CARGO_HOME`, and `<checkout>/.tmp`. Keep the JSON.

## Build cost and binary size

`admin/build-cost` measures a cold `cargo build --release --frozen` of each
fixture in `admin/build-cost-fixtures`: `rind`, `aws-lc-rs`, `ring`,
`rustcrypto`, `graviola`, `libcrux`, `openssl` (the system library, linked
dynamically), `openssl-vendored` (OpenSSL built from source), and the three
`rustls-*` provider fixtures. The default is three runs. For each build the
JSON has the wall times, the CPU times, `stripped_bytes` of the stripped
binary, the tools the build needed beyond `rustc` and a linker, and the
crates it compiled. The sheet prints size as decimal MB
(`stripped_bytes / 1e6`).

It exits `error: host is busy (N of M CPUs busy)` when `busy_cpus()` is
above `0.25 * cpus()`. On Linux, `busy_cpus()` is the next 5 seconds of
`/proc/stat`, scaled by `os.cpu_count()`, and `cpus()` is
`len(os.sched_getaffinity(0))`. The script uses that sample there because
the 1-minute load average still counts work that just ended, such as the
benchmarks. On macOS, `busy_cpus()` is that load average (so N in the
message is a load average) and `cpus()` is `os.cpu_count()`. A build-cost
started the moment the benchmarks end can therefore fail on a Mac that is
already idle. `run` stores a non-zero exit as `build_cost.status`
`failed`, with `started` and `reason` set to the last stderr line, and
keeps the speed numbers. The published `nixos` sheet says that build was
not measured because the host was busy. The reason stored there still uses
an older wording (`load average ...`); the script at the cited commit
prints the `N of M CPUs busy` form.

The script's own docstring says to run it in the dev shell so the toolchain
matches CI. It does not check `IN_RIND_DEV_SHELL` or the pin. It records
whatever `rustc` and `cc` are on `PATH`. Rerun it alone, once the machine
is under the 25% line, from the same checkout:

```shell
nix develop --command python3 admin/build-cost
```

That prints a new JSON object to stdout and does not edit the host file.
To show it in the sheet, set the host's `build_cost` to that object plus
`"status": "ok"` and a `"started"` timestamp, which is what `run` stores
on success. Leave a failed object in place if you do not have a clean
rerun; the sheet will say it was not measured.

## OpenSSL headers without root

`run` passes `--features __bench_openssl`. On Unix that compiles the
`openssl` dev-dependency, and `openssl-sys` needs headers. `run` also
executes `openssl version` and stores it under `libraries.openssl.linked`.
If the CLI is missing, that call fails the run.

The default dev shell provides both. Outside it, a Debian-family board that
has the runtime library and the `openssl` CLI but not the headers, and no
permission to install packages, can unpack them into the work directory:

1. If `pkg-config --exists openssl` succeeds, the headers are already
   visible. Stop.
2. Otherwise `apt-get download libssl-dev` and `dpkg -x` the deb into a
   directory under the work dir. That downloads a package. It does not
   install it. If the download fails, stop. Do not use sudo, and do not
   copy anything into the system directories.
3. Debian puts architecture-specific headers, including `configuration.h`,
   under `usr/include/<triplet>/openssl/`. The triplet is
   `gcc -print-multiarch`. Copy those headers into the unpacked
   `usr/include/openssl/` so one include directory contains them.
4. Point `OPENSSL_LIB_DIR` at a directory of symlinks named `libssl.so` and
   `libcrypto.so`, aimed at the `libssl.so.3` and `libcrypto.so.3` already
   on the system (often `/usr/lib/<triplet>/`). Point `OPENSSL_INCLUDE_DIR`
   at the unpacked `usr/include`. `openssl-sys` 0.9 uses those two
   variables and does not call pkg-config when both are set.

On Apple silicon, `openssl-sys` looks for Homebrew's OpenSSL under
`/opt/homebrew` before pkg-config. That finds headers. It does not satisfy
the rustc pin. A result built that way still has to say which `rustc` it
used.

The OpenSSL ML-DSA benches in `rind-bench/benches/mldsa.rs` skip themselves
when the linked OpenSSL is older than 3.5. A missing OpenSSL cell on those
rows can be that skip.

## A board reached through another machine

Run the job on the board, from the machine that already has a key for it.
`ssh -o BatchMode=yes` so a missing key fails instead of asking for a
password. If that fails, stop. Ask the owner to install a key from the
machine that will connect, and do not try a password. JetPack images often
have no default account.

To see which address on your own LAN is a board you already own, read the
neighbor table that machine already has (`ip neigh` or `arp -a`) and match
the vendor's MAC prefix. Then try the key. Do not scan for accounts.

Copy the `git archive` and `host-run.py` to a work directory on the board,
and run one background job whose log stays in that directory (`nohup`, with
stdout and stderr in the work dir). Copy back the JSON and the log when it
finishes. Leave the target directory and the cargo home there, or delete
them on the board. Do not run the board's speed suite at the same time as
another heavy build on the board. The machine you hopped through can run
its own suite; the two do not share CPUs.

## Noisy hosts

`run` sets `load.noisy` when other CPUs busy during the benchmarks, after
subtracting the benchmark process, are above the same `busy_limit()` as the
pre-check. `summary` prints `(noisy)`. The sheet's Measured cell gains
`, noisy` from `measured()`. The published `nixos` row is that mark.

Do not start the speed suite while another heavy build is on that host. If
you already know the machine is shared, or someone is using it, say that in
the host note even when `noisy` is false. The pre-check counts live in
`load.busy_cpus_before_build` and `load.busy_cpus_before_run`; a host can
be busy then and quiet in the sample that sets `noisy`. A small gap on a
noisy host is not a finding. The nightly `bench` workflow is the clean
comparison.

## Apple silicon and DIT

`rind_rows()` in `rind-bench/benches/criterion/mod.rs` adds a
`rind-dit-held` row only for an Apple `aarch64` target where DIT is
detected. That row holds `rind::cpu::Dit::hold()` across the timed loop.
Every other target skips the row, because the guard would not change the
code. `admin/bench-report` shows it as `rind (DIT held)` and does not count
it as the fastest library or as a rival in a gap (`VARIANTS`).

The published `kixtop` note says rind sets DIT around each call that handles
a secret, and the other libraries never set it. A new Apple machine's short
operations have that shape. Read the DIT-held column before treating a
small-input gap on the rind column as a regression. The published Known
gaps entry for `kixtop` is the maintainers' decision for that machine, not
a measurement of the new one.

## ML-DSA rows

`BENCHMARKS.md` at the commit the README cites says ML-DSA-65 key
generation, sign, and verify were not timed on every host in `HOSTS`. The
stored JSON has no `mldsa` results. `OPERATIONS` at that same commit
includes those three operations, and `run` times them. `mldsa44` and
`mldsa87` are in
`rind-bench/benches/mldsa.rs` but not in `OPERATIONS`, so `run` does not
time them. Compare a new host's three ML-DSA-65 numbers with another run
that timed them. The published sheet has dashes on those rows.

## aarch64 without SHA-3

`quest/m2/bench/README.md` says no report host and no forced tier times
aarch64 with the crypto extension but without SHA-3, and names that class
as Graviton 2, the Raspberry Pi 5, and most phones. Name the board. The
published aarch64 column measures a different configuration.

## How long to leave it

The comment on `CRITERION_ARGS` says the shortened timing is so a host's
run takes about half an hour. One Raspberry Pi 5 measured this way spent
about 70 minutes in criterion, and build-cost on that same multi-host run
added on the order of 10 to 30 minutes per host. Those longer figures are
from that run. The script's own estimate is the half hour. A quiet log is
the benchmark running.
Do not kill it for silence inside that window, and do not expect a slower
board to match the half hour.
