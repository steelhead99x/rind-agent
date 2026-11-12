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

```shell
cargo bench                              # local criterion benches
admin/bench-report collect               # report benches on each bench host over ssh
admin/bench-report render --from DIR     # regenerate BENCHMARKS.md and SVGs
```

Shared hosts are noisy; the nightly `bench` workflow gives the clean numbers.
Every gap over 10% needs a quest or a reason in `benchmarks/gaps.toml`.
Record the host (CPU and OS) and the date with every number. Re-measure on
the head that will merge, or state that hardware and date of the earlier
figures.

## Other architectures

```shell
just armv7                # 32-bit ARM build, lint, and qemu tests
just qemu <shell>         # portable backend; shells are listed in CONTRIBUTING.md
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

## Wasm

`just wasm` runs the portable backend's release vectors on wasm32-wasip2
under wasmtime. There are no differential tests on wasm, because aws-lc
does not build for it; `just portable` runs the same Rust against aws-lc.
Any test or helper that still touches aws-lc needs the gate in
[Miri](#miri).

Tests under wasmtime do not inherit the host environment. Forward
`SLOW_TESTS`, `PROPTEST_CASES` and `PROPTEST_RNG_SEED` into the guest
explicitly, and set `SLOW_TESTS=1` the way the portable tier does.

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

The nightly `mutants` workflow, through `admin/mutants`, mutates recent
Rust on each architecture rind has a runner for, on armv7 under qemu, and
on the AVX-512 paths under Intel SDE. A mutant no architecture's tests
catch fails the run; combine the `mutants.out` directories with
`admin/mutants --survivors`. macOS arm64 covers only the DIT code.
`.github/workflows/mutants.yml` and `admin/mutants` have the details.

A pull request that touches the mutation inputs also gets a run over its
diff. That run is advisory, but fix a red one before merging, as
`CONTRIBUTING.md` says. A large diff can exceed the job limit: shard it
with cargo-mutants' `--shard k/n`, choosing `n` from `--list`'s count so
each shard fits, then combine the shards with `admin/mutants --survivors`.
The planned work is in `quest/m2/confidence/mutants-large-diffs.md`.

## Proofs

```shell
nix develop .#prove --command admin/prove
```

Checks the HOL Light proofs of the assembly, then translates the Rust under
proof to Lean with Aeneas and checks `proofs/`. `benchmarks/verified.toml`
records what is proven end to end; only mark a row proven when every line of
code on that path, glue included, is machine-checked.

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

