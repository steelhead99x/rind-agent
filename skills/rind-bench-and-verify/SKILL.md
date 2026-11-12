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
just qemu <shell>         # portable backend: s390x, riscv64, loongarch64, powerpc*, armv6, i686
just miri                 # portable backend under Miri
just miri s390x-unknown-linux-gnu   # big-endian under Miri
```

`just qemu` lints and tests only the portable backend. To exercise a lower
CPU tier on a capable host, set `RIND_CPU_DISABLE_<id>` (dots dropped, so
`sse4.1` is `RIND_CPU_DISABLE_sse41`) in a debug build, or in a release
build with `--cfg rind_cpu_disable`. The ids are the feature names
TESTING.md's denied tiers and `rind/src/low/<arch>/cpu.rs` use, for example
`avx512f`, `avx2`, `ssse3`, `aes`, and `neon`. Compare each forced
backend's output with the scalar or portable path on the same vectors.

A native SIMD path on another architecture (NEON on an x86_64 host, for
example) needs qemu-user with that target, not `just qemu`.

## Wasm

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
nix develop .#binsec --command admin/ct-binsec    # symbolic proof of ct_binsec_* tests
nix develop .#dudect --command admin/timing dudect
```

A new secret path gets a test that marks its secrets with ctgrind and
declassifies only the final public output; the run should end with valgrind's
`ERROR SUMMARY: 0`.

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

Some dev-dependencies (`aws-lc-sys` among them) only exist outside Miri.
Every test or helper that touches one needs `#[cfg(not(miri))]`, like the
existing aws-lc tests; one missing gate stops Miri compiling every unit
test in the crate. After adding a differential test, check that the Miri
build still compiles (`cargo +nightly miri test --no-run -p rind --lib`).

