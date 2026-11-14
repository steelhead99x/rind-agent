---
name: rind-bench-and-verify
description: Use when a rind change needs benchmarks, constant-time checks, cross-architecture tests, or formal proof checks beyond `just check`.
---

# Benchmarks, timing and proofs in rind

All commands run from the repo root. Most need the Nix dev shells. On a bare
Linux VM a single-user Nix install is enough to get them. Check out the vector
submodules (`thirdparty/cavp`, `thirdparty/cctv`, `thirdparty/wycheproof`)
before `just check` or `just ci`, or the gate stops early.

## Benchmarks

The published comparison inside a rind checkout is `admin/bench-report`.
A comparable set that leaves this checkout follows
[rind-bench-pin](../rind-bench-pin/SKILL.md),
[rind-bench-measure](../rind-bench-measure/SKILL.md),
[rind-bench-schema](../rind-bench-schema/SKILL.md), and
[rind-bench-publish](../rind-bench-publish/SKILL.md). The pin, the three
runs, and `cargo xtask bench` live in those skills. A bare `cargo bench`
is not that set.

```shell
cargo bench -p rind-bench -- <filter>   # one local harness, criterion's own timing
admin/bench-report run [--host NAME] --commit REV --jobs N --target-dir DIR --out FILE
                                        # this machine; NAME defaults to a CPU slug
admin/bench-report summary DIR          # every <host>.json in DIR
admin/bench-report collect [--host NAME]...
                                        # only machines in COLLECT_HOSTS, over ssh
admin/bench-report render --from DIR    # every measured host JSON in DIR
```

`cargo bench` with no package and no filter builds every harness under
`rind-bench/benches` and uses criterion's defaults, 3 s of warm-up and 5 s
of measurement. `admin/bench-report` shortens that to
`--warm-up-time 1 --measurement-time 3` so one host takes about half an
hour, and it times only `OPERATIONS` (62 at `7ab9f85`, the commit the
rest of this file was checked against). Harnesses the report never asks
for, such as `mldsa44`, `mldsa87`, `sha1`, `md5`, `pbkdf2`, and
`rsa_gen`, still run under a bare `cargo bench`. Use `run` for the
report's set.

`run` measures the machine it is on. Anyone can run it. Omit `--host`
and the recorded name is a filesystem-safe slug of the CPU model:
`Intel(R) Core(TM) Ultra 9 275HX` becomes `intel-core-ultra-9-275hx`,
`AMD Ryzen 9 7950X 16-Core Processor` becomes `amd-ryzen-9-7950x`, and
`Apple M3 Max` becomes `apple-m3-max`. `--host` sets another name of
letters, digits, `.`, `_`, and `-`, at most 81 characters, starting
with a letter or digit. Nightly tier names and `tiers` are reserved.
The JSON records the raw CPU string, `cpu.slug`, the logical core
count, and `physical_cpus` when the OS reports it. `host` is the slug
unless `--host` set another name. Write the file as `<host>.json`.
This naming is `admin/bench-report` at `1fca3086`
(steelhead99x/rind #83). On a checkout older than that commit, `run`
still requires `--host`. rind-results `cargo xtask bench` passes that
flag, set to the CPU slug when `--board` is omitted.

`collect` is the maintainer path. It reaches only the machines named in
`COLLECT_HOSTS`, over ssh, in that list's order. `--host` on `collect`
must be one of those names. Any other name fails with
`unknown collect host` and points at `admin/bench-report run`. For each
name it archives the commit with `git archive`. When that name is the
first label of this machine's hostname, the archive is unpacked
locally; otherwise it is copied over ssh. On the host it runs
`nix shell` of the quest revision pinned in `flake.lock`, then `quest gate`,
then `nix develop --command python3 admin/bench-report run`. That host
needs Nix with flakes, and network so Nix can fetch the pinned quest and
the dev shell's inputs.

`summary` prints every `<host>.json` in the directory. `tiers.json` is
the nightly CPU tiers, not a host. Machines in `COLLECT_HOSTS` stay in
that list's order. Every other host follows by file name. `render`
writes this checkout's `BENCHMARKS.md` and `benchmarks/*.svg`, and it
deletes a generated SVG the new report does not contain. `--from DIR`
imports every measured host JSON (`status` `ok`). It then exits if a
gap over 10% (`SLOWER` is 1.1) has no decision in `benchmarks/gaps.toml`.
A decision's `on` may name any host that has a result, or any nightly
tier. A name with no result fails.

Measuring a machine for a published set is
[rind-bench-measure](../rind-bench-measure/SKILL.md). Host notes are in
[rind-bench-hosts](../rind-bench-hosts/SKILL.md). The JSON layout is
[rind-bench-schema](../rind-bench-schema/SKILL.md) and
[rind-bench-results](../rind-bench-results/SKILL.md). Those leave this
repo's `BENCHMARKS.md` untouched.

Shared hosts are noisy; the nightly `bench` workflow gives the clean numbers.
Every gap over 10% in the published report needs a quest or a reason in
`benchmarks/gaps.toml`. Record the host (CPU and OS) and the date with every
number. Re-measure on the head that will merge, or state that hardware and
date of the earlier figures.

## Other architectures

```shell
just armv7                # build, lint, and Linux gnueabihf qemu tests
just qemu <shell>         # portable backend; shells are in TESTING.md, "Running the tools"
just wasm                 # portable backend on wasm32-wasip2 under wasmtime
just miri                 # portable backend under Miri
just miri s390x-unknown-linux-gnu   # big-endian under Miri
```

`just qemu` lints and tests only the portable backend. The shells include
`s390x`, `riscv64`, `riscv64-musl`, `loongarch64`, `armv6`,
`armv6-softfloat`, `powerpc64le`, `powerpc64`, `powerpc`, and `i686`.
`i686` runs natively on an x86_64 host and is covered at the workspace
MSRV. The others run under qemu unless the host is already that target.

To exercise a lower CPU tier on a capable host, set
`RIND_CPU_DISABLE_<id>` (dots dropped, so
`sse4.1` is `RIND_CPU_DISABLE_sse41`) in a debug build, or in a release
build with `--cfg rind_cpu_disable`. The ids are the feature names
TESTING.md's denied tiers and `rind/src/low/<arch>/cpu.rs` use, for example
`avx512f`, `avx2`, `ssse3`, `aes`, and `neon`. Compare each forced
backend's output with the scalar or portable path on the same vectors.

A native SIMD path on another architecture (NEON on an x86_64 host, for
example) needs qemu-user with that target, not `just qemu`.

## Android under QEMU

A pull request skips the `android` job in `.github/workflows/build.yml`
(`if: github.event_name != 'pull_request'`). That job runs rind's Android
tests. `just armv7` (`sh/armv7.sh`) builds and lints
`armv7-linux-androideabi`, then runs qemu on
`armv7-unknown-linux-gnueabihf`. ARM-mode gnueabihf misses the Thumb path.
TESTING.md's armv7 rows are those `just armv7` tiers. Its portable rows
name the `android` job for 32-bit x86 and for ARMv5TE
(`arm-linux-androideabi`, ARM mode).

Use NDK r27c at API 28. API 28 is Android 9, which `Cross.toml` records
as the system the CI images hold. Unpack `android-ndk-r27c-linux.zip`.
Both 32-bit targets link with `armv7a-linux-androideabi28-clang`, and
aarch64 with `aarch64-linux-android28-clang`, under
`toolchains/llvm/prebuilt/linux-x86_64/bin`. Build static bionic
binaries with `-C target-feature=+crt-static`. Doctests need the same
flag in `RUSTDOCFLAGS`: Cargo passes rustdoc that variable, which is
why `flake.nix` sets it for the qemu shells.

```shell
NDK=/path/to/android-ndk-r27c
BIN="$NDK/toolchains/llvm/prebuilt/linux-x86_64/bin"
rustup target add armv7-linux-androideabi thumbv7neon-linux-androideabi aarch64-linux-android
export RUSTFLAGS="-C target-feature=+crt-static"
export RUSTDOCFLAGS="-C target-feature=+crt-static"
export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="$BIN/armv7a-linux-androideabi28-clang"
export CARGO_TARGET_THUMBV7NEON_LINUX_ANDROIDEABI_LINKER="$BIN/armv7a-linux-androideabi28-clang"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$BIN/aarch64-linux-android28-clang"
export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_RUNNER="unshare --user --map-root-user --pid --fork qemu-arm"
export CARGO_TARGET_THUMBV7NEON_LINUX_ANDROIDEABI_RUNNER="unshare --user --map-root-user --pid --fork qemu-arm"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_RUNNER=qemu-aarch64
```

Targets are `armv7-linux-androideabi` (rustc lowers it to Thumb),
`thumbv7neon-linux-androideabi` (Thumb), and `aarch64-linux-android`.

32-bit bionic accepts only a pid below 65536: `pthread_mutex_t` cannot
hold a larger one. The 32-bit runner above is qemu-arm user mode inside
`unshare --user --map-root-user --pid --fork`, which puts the process in
a fresh pid namespace. The aarch64 runner is `qemu-aarch64`.

`qemu-user` provides `qemu-arm` and `qemu-aarch64`. When `apt-get`
cannot install it, download its `.deb` files from the Ubuntu archive
suite `resolute` (26.04) and require `sha256sum -c` against that
suite's published `SHA256SUMS` or the hashes in its `Packages` file.
If the suite, those hashes, or the check is missing, stop and ask the
user. Do not pipe the download to a shell. Extract each checked
`.deb` with `dpkg -x` into one prefix. Put that prefix's `usr/bin` on
`PATH` and its libraries on `LD_LIBRARY_PATH`.

Check out the vector submodules first. Then run the job's test step:
the same `-p` list, `--locked --release`, and release checks
(`CARGO_PROFILE_RELEASE_DEBUG_ASSERTIONS=true`,
`CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS=true`). Pass the matrix `expect`
values: `RIND_TEST_ARM_NEON=1 RIND_TEST_ARM_CRYPTO_EXT=1` for both
32-bit targets, and `RIND_TEST_ARM_CRYPTO_EXT=1` for aarch64. qemu-user's
default CPU is `any`, which it treats as `max`, so NEON and the crypto
extension are present.

```shell
export CARGO_PROFILE_RELEASE_DEBUG_ASSERTIONS=true
export CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS=true
RIND_TEST_ARM_NEON=1 RIND_TEST_ARM_CRYPTO_EXT=1 \
  cargo test -p rind -p rind-rustls -p rind-jsonwebtoken \
  -p rind-object-store -p rind-rcgen -p rind-ring -p rind-aws-lc-rs \
  -p rind-dimpl -p rind-str0m --locked --release \
  --target armv7-linux-androideabi
```

Repeat that command for `thumbv7neon-linux-androideabi` with the same
variables, and for `aarch64-linux-android` with only
`RIND_TEST_ARM_CRYPTO_EXT=1`.

Run it at the pull request head and at the commit before the fix. The
pre-fix commit has to fail for the bug's reason. A pass at both commits
shows nothing about whether the tests catch the bug. On the armv7 Thumb
ABI fix, the pre-fix commit failed `low::abi` (3) and
`cargo test -p rind --lib` (227), every failure `sp_after == 0`. At the
head, the armv7 command above reported 746 passed, 0 failed.

A run that stops in `apt-get install qemu-user-static` because
`ubuntu-26.04` has no installation candidate never reached the tests.
Treat that as an infrastructure failure. The job installs
`qemu-user-binfmt` instead (`build.yml`, rind #600).

## Wasm

`just wasm` runs the portable backend's release vectors on wasm32-wasip2
under wasmtime. There are no differential tests on wasm, because aws-lc
does not build for it; `just portable` runs the same Rust against aws-lc.
Any test or helper that still touches aws-lc needs the gate in
[Miri](#miri).

Tests under wasmtime do not inherit the host environment. `sh/wasm.sh`
forwards only `SLOW_TESTS`, and the release run sets `SLOW_TESTS=1`.

proptest does not read `PROPTEST_RNG_SEED` on wasm32; apply the printed
seed in the wasm proptest config. wasm32-wasip2's standard library aborts
on panic, so a failure can be replayed from that seed but cannot shrink,
and `should_panic` or `catch_unwind` tests cannot run there.

proptest's fork runner (`wait-timeout`) does not build for wasm. Drop the
`fork` feature for the wasm dev-dependency only; leave it on for host
builds.

A Cargo target runner is a single config-array entry with no extra `--`,
and it must fail when a harness ran zero tests.

Wasm tiers do not check constant time: the engine recompiles the code and
may turn a select into a branch. Say so in the docs for that tier.

## Constant time

```shell
nix develop .#ctgrind --command admin/ctgrind
nix develop .#ctgrind --command admin/ctgrind unit --release -- <test path>   # one test
nix develop .#sde --command admin/ct-sde          # AVX-512, VAES, SHA ext code
nix develop .#ct-qemu --command admin/ct-qemu     # armv7 crypto ext code
nix develop .#ct-qemu-aarch64 --command admin/ct-qemu --target aarch64-unknown-linux-gnu
                                                  # aarch64 code that runs only with SHA3
nix develop .#binsec --command admin/ct-binsec    # symbolic proof of ct_binsec_* tests
nix develop .#binsec --command admin/ct-binsec --exact [test]   # one harness
nix develop .#dudect --command admin/timing dudect
```

A new secret path gets a test that marks its secrets with ctgrind and
declassifies only the final public output; the run should end with valgrind's
`ERROR SUMMARY: 0`.

Pass a `Vec<u8>`'s bytes to the `secret` and `public` helpers in an
adapter's ctgrind test (`rind-aws-lc-rs/tests/ctgrind.rs` is one):
`secret(&v[..])` and `public(&v[..])`. They take `&T` and mark
`size_of_val` bytes, so `secret(&v)` marks the header (pointer, capacity,
length) and taints that public length. Lengths derived from it are
public, but ctgrind reports them as secret-dependent branches: PKCS#8
and DER length octets, the ASN.1 parser's bounds checks, and the `Vec`'s
own drop. Declassify a secret-derived buffer with `public` before
comparing it (`assert_eq!`, `==`).

That still leaves the payload unmarked, so a passing run can miss
coverage. When a failure is fixed in the test's marking, confirm the
same bytes are still covered. A declassify before `assert_eq` covers
only a true public output or a test artifact, such as a verify `Result`.

Library tests mark secrets with `low::ct::secret_slice` and declassify
with `low::ct::public_slice` or `low::ct::into_public` before
`assert_eq!` on those bytes (`rind/src/low/ct.rs`).
`rind/src/high/agreement.rs` copies a shared secret, calls
`public_slice`, then compares. A loader can mark its input secret for
the rest of the process (that file's "Loading marks the input's scalar
secret"), so a later assertion or a later test that reuses the same
buffer inherits that taint. Copy the bytes into an owned buffer per
test, then `public_slice` that copy.

CI's `ctgrind` workflow runs `admin/ctgrind` with no test-path filter.
On a pull request the build argument is `--release`
(`.github/workflows/ctgrind.yml` at the pin in
[rind-bench-pin](../rind-bench-pin/SKILL.md)). A passing filtered local
run is not a substitute.

ctgrind under valgrind needs Linux x86. It does not run on Apple Silicon.
A cloud VM can run it when that VM is Linux x86. The run is for that head;
repeat it when the head moves.

Run a new primitive's ctgrind test and its canonical-encoding checks on
every backend the crate builds: native x86_64, the portable backend
(`--cfg rind_portable` with `curve25519_dalek_backend="serial"`, as
`sh/portable.sh` sets), aarch64, and armv7. `TESTING.md` names the ctgrind
job for the portable backend. Decide whether the primitive needs a
`ct_binsec_*` test; `admin/ct-binsec` does not cover armv7. Compare a
non-canonical encoding with an established implementation, such as
`akd_core` for ECVRF.

Do not use `cargo test --all-features` as a local stand-in for
`admin/ctgrind`: it enables `__ctgrind` and needs the valgrind headers
the `.#ctgrind` shell provides. Without them, bindgen fails to find
`valgrind/valgrind.h`. For ordinary local tests, use default features
plus the specific features a test needs.

`admin/ct-vartime` runs inside `just check` and fails on unlisted divisions or
floating point. Counting blocks of a hash whose output isn't a power of two
(SHA-384's 48 bytes) with a loop keeps a division out of it. A public
division that has to stay is allowlisted by its exact function name; note
that a 128-bit division (`2^64 / n`) becomes `__udivti3` only on 64-bit
targets, so it shows up per target.

`overflowing_sub`'s borrow compiles to a conditional jump, which ctgrind
flags on secret data. Take the borrow from a shift of the wrapped
difference, with `black_box` on the mask, the way `low::generic::bignum`
does.

dudect and Marvin need quiet hardware; don't trust numbers from
a shared box.

## Mutation testing

The `mutants` workflow runs nightly and on `workflow_dispatch`. Like
fuzzing, it never runs on pull requests. Nightly, through
`admin/mutants`, it mutates the Rust that changed in the last day, on
each architecture rind has a runner for, on armv7 under qemu, and on
the AVX-512 paths under Intel SDE. A mutant no architecture's tests
catch fails the run. `alert.yml` reports that failure when the run was
scheduled. Combine the `mutants.out` directories with
`admin/mutants --survivors`. macOS arm64 covers only the DIT code.
`.github/workflows/mutants.yml` and `admin/mutants` have the details.

`CONTRIBUTING.md` still names a mutation run over a pull request's
diff. The workflow does not start it. For evidence on a branch,
dispatch the workflow or run `admin/mutants` locally on the touched
crates.

`gh workflow run mutants --ref <branch>` dispatches it. With no `base`
input it mutates the last day on that ref. Set `base` to mutate what
the branch changed since that revision.

A local run uses the dev shell, which provides cargo-mutants.
Arguments pass through, and the run covers this host's architecture:

```shell
nix develop --command admin/mutants -p <crate>
```

`quest/m2/confidence/mutants-adapters.md` uses that `-p <crate>` form.
Pass `--in-diff` a diff of `*.rs` since the merge-base of `<base>` and
`HEAD` to limit the run to the branch, as the workflow's diff step
does.

## Proofs

```shell
nix develop .#prove --command admin/prove
```

Checks the HOL Light proofs of the assembly, then translates the Rust under
proof to Lean with Aeneas and checks `proofs/`. `benchmarks/verified.toml`
records what is proven end to end; only mark a row proven when every line of
code on that path, glue included, is machine-checked.

The `hol-light` and `cosim` jobs in `.github/workflows/prove.yml` set
`timeout-minutes: 30` at the pin. A proof that runs longer is
cancelled, which is a CI failure, not a failed proof. A first-time
contributor's workflow runs still need a maintainer's approval before
they start, as [Known issues](../rind-quest-run/SKILL.md#known-issues)
records.

## Slow tests

`SLOW_TESTS=1 cargo test` adds the large vector sets (for example CCTV's
million ML-KEM vectors).

## Miri

Some dev-dependencies (`aws-lc-rs` and `aws-lc-sys` among them) are left
out of Miri and wasm builds. `rind/Cargo.toml` gates them with
`cfg(not(any(miri, target_family = "wasm")))`: Miri cannot run C, and
aws-lc does not build for wasm.

Every test or helper that touches one needs
`#[cfg(not(any(miri, target_family = "wasm")))]`, like the existing aws-lc
tests. A file that only exists to call aws-lc uses the inner `#!` form.
A missing gate breaks `just wasm` and the wasm nightly (the `wasm` job in
`.github/workflows/build.yml`), not only Miri: the build fails to compile
the tests. After ML-DSA, `rind/tests/heap_scratch.rs` called aws-lc under
`#![cfg(feature = "wipe")]` alone.

After adding a differential test, check that the Miri build still compiles
(`cargo +nightly miri test --no-run -p rind --lib`) and that `just wasm`
still compiles.

