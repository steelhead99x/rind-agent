---
name: rind-armv6m-constant-time
description: Use when checking that rind or a dependency stays constant-time on ARMv6-M, including carry compares, disassembly scans, and emulator instruction traces.
---

# Constant time on ARMv6-M

ctgrind, SDE, and `admin/ct-qemu` in
[rind-bench-and-verify](../rind-bench-and-verify/SKILL.md) do not cover
thumbv6m. The review checklist in
[rind-crypto-review](../rind-crypto-review/SKILL.md) still applies;
this skill is how to check the Thumb-1 object code.

`muls` is fixed-time on the Cortex-M0+ in the RP2040. compiler-builtins
`__aeabi_lmul` is branch-free on rustc 1.99.1; libgcc's routine of the
same name is not. See [rind-bench-baremetal](../rind-bench-baremetal/SKILL.md).
A hot path should still avoid `__aeabi_lmul` for performance, as in
[rind-armv6m-perf](../rind-armv6m-perf/SKILL.md).

## The carry-compare lesson

On thumbv6m, LLVM routinely compiles a u64-free carry compare
(`overflowing_sub`, `a < b` on a carry, a final `h >= p` mask select)
into `cmp` followed by `blo` (or `bhs`, `bmi`, `bne`). The instruction
trace then depends on secret data.

A widening-mul rewrite that only removes `__aeabi_lmul` can still
introduce those branches. One P-256 32×32→64 patch did: `mul_wide`,
`sqr_wide`, `jdouble`, `jadd`, and `jmixadd` were no longer constant
time until a follow-up used branch-free 16-bit column carries and
opaque selects. Do not ship or time the first patch as a complete
change without that follow-up.

The same pattern appeared in a Poly1305 `h >= p` mask and can appear
in any `cmovznz` or Booth-sign select (those have also compiled to
`bne`). Put masks and biases through `core::hint::black_box` (or
rind's `opaque`) so LLVM cannot turn them back into branches.

`overflowing_sub`'s borrow compiles to a conditional jump on other
targets too; rind already takes the borrow from a shift of the wrapped
difference, as [rind-bench-and-verify](../rind-bench-and-verify/SKILL.md)
describes.

## Disassembly scan

On the exact image you will flash or publish, disassemble every hot
secret path (llvm-objdump from the same toolchain). For each function:

- No conditional branch on a secret. Branches on public loop counters,
  public lengths, and public key sizes are allowed; list them.
- No register-indexed load or store with a secret index (`ldr r, [r, r]`,
  `ldrb [r, r]`, and the same for stores). A public table index (a
  round counter, a key-length enum) is allowed; say so.
- No secret-dependent table (an S-box is a failure). Fixsliced or
  bitsliced AES must not load tables.
- Residual `bl __aeabi_lmul` in rind secret code is a performance
  issue and, if the helper ever comes from libgcc, a constant-time
  issue. Treat an unexpected site as a failed check.

A one-instruction `bmi` that decodes from padding after `pop {pc}` is
not a real branch; confirm it is padding before ignoring it.

## Emulator instruction traces

Static disassembly can miss a path. Compare instruction traces (PC
traces) of the same function across random secret inputs, for example
with a Unicorn (or other ARMv6-M) emulator:

1. For each hot function, run at least eight random inputs (more for
   a new invert or ladder).
2. Hash or otherwise compare the PC sequence, or the full instruction
   trace. They must be identical for a CONSTANT verdict.
3. If they differ, the first diverging branch is the bug. Fix it with
   a branch-free carry or an opaque mask, then repeat the trace and
   the disassembly scan.

Public loop structure can still branch (a limb count, a fixed number
of divsteps). Those counts must not depend on the secret. A
constant-time invert such as Bernstein–Yang safegcd uses a fixed
divstep bound (for example 25×30 for P-256, 38×30 for P-384, by the
usual theorem bound) and must show one trace for all inputs.

Emulator cycle counts are **estimates**, not board timings, especially
when the model ignores XIP misses. They are valid evidence for
instruction-trace equality.

## Code size and the cache

A branch-free rewrite can grow the hot path past the 16 KiB XIP cache
and regress board timings even when the traces stay constant-time. One
P-256 follow-up grew the hot code from about 9.3 KiB to 15.3 KiB, and
ECDH went 1.52× slower until the hot functions were packed or placed
in SRAM. After a constant-time fix, time the image on the board and
follow [rind-armv6m-perf](../rind-armv6m-perf/SKILL.md). Emulator cycle
counts cannot predict that regression.

Board KATs confirm the new path matches the known answers. They do not
prove constant time or a performance win. The sign-off list in
[rind-bench-rp2040](../rind-bench-rp2040/SKILL.md) requires a
disassembly scan as well as the KATs.

## What is not enough

A green ctgrind run on x86 or aarch64 does not prove thumbv6m is
constant time: the carry compare only became a branch on this target.
Host tests, Wycheproof, board KATs, and "zero `__aeabi_lmul`" are
necessary and not sufficient. Re-run the thumbv6m scan when the
compiler, LTO settings, or the surrounding crate change.
