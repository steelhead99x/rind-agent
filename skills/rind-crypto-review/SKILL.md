---
name: rind-crypto-review
description: Use when reviewing or self-checking a rind diff before it merges — the correctness, constant-time, input-handling and wiping rules a crypto change must pass.
---

# Reviewing a rind change

Check the diff against each rule. Any "no" blocks the merge until fixed or
explained in the PR.

## Correctness

- [ ] Every new or changed primitive is tested against Wycheproof where vectors
      exist, and differentially against aws-lc-rs.
- [ ] New arithmetic says where its correctness argument comes from (a proof,
      a cited paper, or the upstream it was taken from).
- [ ] Code copied from another project keeps its copyright lines and an
      `SPDX-License-Identifier`, and `LICENSE` and `ALGORITHMS.md`'s Sources
      table both list the project.
- [ ] Behavior questions (accept, refuse, wipe) follow aws-lc-rs unless the PR
      says why not.

## Hostile input

- [ ] Malformed keys, signatures, ciphertexts and encodings return an error,
      never panic.
- [ ] The cost of rejecting bad input is bounded (no attacker-controlled loop
      counts or allocations).

## Constant time

- [ ] No branch, memory index, or variable-time instruction (division,
      floating point) depends on a secret.
- [ ] New secret paths have a ctgrind test. If valgrind can't see the code
      (AVX-512, VAES, SHA extensions), the SDE or qemu runner covers it.
- [ ] `admin/ct-vartime` still passes; any new division is listed in
      `admin/ct-vartime.toml` only if its operands are public.

## Platform rules

- [ ] Exactly one backend per target; no runtime fallback.
- [ ] A CPU missing a required feature fails with an error naming it.

## Wiping

- [ ] Key types wipe their whole value on drop, public parts included.
- [ ] Keyed state and heap scratch derived from a private key are wiped through
      `low::zeroise` / `low::zeroise_value`, under the `wipe` feature.
- [ ] Nothing else gets wiped (digest contexts, plaintext, stack temporaries);
      that's speed spent for nothing.

## API

- [ ] Misuse fails to compile where a type can enforce it.
- [ ] One way to do each job. Borrowed ring/aws-lc-rs names are only used when
      they're as clear, and `MIGRATING.md` maps them.
- [ ] Adapter crates expose exactly their upstream provider's API, nothing
      added.

## Performance

- [ ] Performance-sensitive changes include before/after numbers against
      aws-lc-rs on the CPUs the author has.
- [ ] Any gap over 10% has a quest or a reason in `benchmarks/gaps.toml`.
