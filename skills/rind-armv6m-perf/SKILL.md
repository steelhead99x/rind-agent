---
name: rind-armv6m-perf
description: Use when optimizing rind or a dependency for ARMv6-M (thumbv6m): widening multiplies, XIP layout, SRAM placement, or checking that aarch64 and x86 codegen is unchanged.
---

# ARMv6-M performance work in rind

Timing a board is [rind-bench-rp2040](../rind-bench-rp2040/SKILL.md).
Checking constant time on this target is
[rind-armv6m-constant-time](../rind-armv6m-constant-time/SKILL.md).
Compile and backup notes are in
[rind-bench-baremetal](../rind-bench-baremetal/SKILL.md).

## The cfg gate

Gate ARMv6-M specializations with:

```rust
all(target_arch = "arm", not(target_has_atomic = "ptr"))
```

Do not key on `target_feature = "thumb2"`. The atomic-pointer gate is
the stable way to name the no-CAS ARM cores (M0, M0+, M23) that also
lack `umull`. Exercise the new path under `cfg(test)` on every target
so host tests cover it.

The same gate must not change generated code on aarch64 or x86_64.
Compare release rlibs function by function with `codegen-units=1`.
Every function's instruction stream must be identical (allow ThinLTO
`.llvm.<n>` suffix noise). Default codegen-units can reshuffle
partitions when a feature changes the crate hash; that is not a
functional difference, but it is not the check. thumbv6m is expected
to differ.

## Widening multiplies

ARMv6-M has `muls` (32×32→32) and no `umull`. A Rust `u32 as u64 * u32
as u64` product becomes `bl __aeabi_lmul`. In rustc 1.98.1 RP2040 bench
images, that helper was measured at 74 bytes, 37 Thumb instructions,
and 6 `muls`: a general 64×64→64 (low half) routine, not a 32×32
specialization. compiler-builtins' routine is branch-free on rustc
1.99.1 ([rind-bench-baremetal](../rind-bench-baremetal/SKILL.md));
libgcc's routine of the same name branches. Either way, a hot path
should not call it.

Replace each 32×32→64 product with four 16-bit `muls` and adds (the
BearSSL-style split). Confirm on the image you will time that the
touched functions contain **0** `bl __aeabi_lmul` and that LLVM did
not recombine the splits into a widening multiply. Put masks and
biases through `core::hint::black_box` (or rind's `opaque`) so LLVM
cannot form a branch or an `__aeabi_lmul` from them.

Where the 64-bit product lives:

- rind field and GHASH code: a local helper under the cfg above.
- P-256/P-384 `mul_wide` / `sqr_wide`, Montgomery conversions, and
  inversions: same helper, then a constant-time invert (Bernstein–Yang
  safegcd / divsteps). Do not ship the widening-mul patch without the
  branch-free carry follow-up; see
  [rind-armv6m-constant-time](../rind-armv6m-constant-time/SKILL.md).
- curve25519-dalek's u32 backend: patch dalek (`m(u32, u32) -> u64`
  and the `19 * c` reduce), applied with `[patch.crates-io]`. Prefer
  an `inline(never)` outline helper; an inlined field mul was measured
  at 7,408 bytes, and the X25519 ladder at 16,230 bytes, about the
  whole 16 KiB XIP cache (outline ladder 7,876 bytes).
- Poly1305: rind's 32-bit path is generated HACL with 64-bit limbs.
  Do not patch that generated code. A rind-local 32-bit (donna-style)
  block under the same cfg is the upstreamable change; leave HACL on
  other 32-bit targets.
- SHA-3: bit-interleave Keccak-f[1600] onto 32-bit lanes so 64-bit
  rotates become 32-bit `rors`. Confirm the permutation has no
  `bl __aeabi_lmul` / `__aeabi_llsl` / `__aeabi_llsr`.
- ML-KEM compress: four 16-bit `muls` for the 32×32→64 product
  (`Compress_1` can stay a 32-bit mul when the product fits).
- AES: GHASH was the first widening-mul bottleneck (48 `__aeabi_lmul`
  in `gcm_mul64_split`, measured 0 after a muls-only `gcm_mul32_rev`).
  A later fixsliced AES (pure Rust, optionally with generated Thumb-1
  rounds) is a separate change. Generated assembly in the portable
  backend is experimental; say so, and require on-device boot KATs.

## The XIP cache

The RP2040 XIP cache is 16 KiB, 2-way, with 8-byte lines. That is
1024 sets, `set = addr[12:3]`, and an alias every 8 KiB. A refill was
measured at about 68–91 cycles per line in earlier board rounds (the
model often uses 80).

A function larger than one 8 KiB way (unrolled SHA-512 compress is
about 27 KiB in HACL and RustCrypto, measured) fights itself in the
cache. Even a function that fits can lose both ways when a caller,
`memcpy`, or another hot line aliases the same sets. Uniform padding
(`RIND_BENCH_PAD` or similar) cannot fix or reproduce that; only a
change in relative layout does.

On ARMv6-M, place hot loops in SRAM with
`#[unsafe(link_section = ".data.rind_hot.*")]` and `inline(never)`.
cortex-m-rt already copies `.data` and `.data.*` into SRAM at reset,
so no linker-script fork is required. Confirm with `llvm-nm` that the
symbols landed at `0x200xxxxx`, and that thunks into SRAM sit in
flash. The SRAM cost is real (several kilobytes; more when P-256
moves too). Forking `link.x` to steal `.text` sections is a last
resort. A linker `--symbol-ordering-file` depends on mangled names
and can make one bench better and another worse; do not rely on it.

A rolled SHA-512 compress for this cfg (1,724 bytes in a measured
image) is the size fix when the unrolled form will not fit in cache
and cannot be relocated because it lives in a dependency.

## Keep only a measured win

Keep a change that is measured faster on the board, or whose generated
instructions are verified to be the intended ones, without a secret
branch. Drop a change that does not move the median. Gate a change
that helps one primitive and hurts another.

Label model or emulator cycle counts as **estimates**. They are not
board timings.
