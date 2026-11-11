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

## Other architectures

```shell
just armv7                # 32-bit ARM build, lint, and qemu tests
just qemu <shell>         # portable backend: s390x, riscv64, loongarch64, powerpc*, armv6, i686
just miri                 # portable backend under Miri
just miri s390x-unknown-linux-gnu   # big-endian under Miri
```

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
