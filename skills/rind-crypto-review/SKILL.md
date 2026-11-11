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
- [ ] Every key encoding the RFC requires is accepted. RFC 9474, for example,
      needs id-RSASSA-PSS SPKIs as well as rsaEncryption.
- [ ] When a modulus' bit length is 1 mod 8, the byte dropped from an encoding
      is checked to be zero rather than ignored.
- [ ] Point addition handles equal and opposite inputs, either through a
      complete formula or a constant-time select. Many Jacobian `add`
      routines are undefined for P = Q, and a peer can force that case.
- [ ] Complete addition is its own function (`complete_add`), called only
      where a peer can force the exceptional cases. Don't slow a shared
      `add` that ECDSA's precomputation already proves safe.
- [ ] Protocol code handles the identity explicitly. When a peer can make
      a difference such as X - w0*M land on the identity, reject it (as the
      RFC allows) before scalar multiplication, and never hand affine (0,0)
      to an assembly routine that isn't specified for it.
- [ ] Every function that returns a value has a test that checks the value,
      through vectors or a differential, not only its length or that it
      differs from another output. Wrong per-curve constants pass shape-only
      tests.
- [ ] Tests reach the edges: inputs at and just past the modulus, values
      that reduce to zero, every hash the RFC gives vectors for, and
      parameters that aren't powers of two (such as lane counts). Fixed
      lists of four to six inputs belong in a proptest.

## Hostile input

- [ ] Malformed keys, signatures, ciphertexts and encodings return an error,
      never panic. Hostile input gets a protocol error, not `Fault`, which
      is reserved for internal bugs.
- [ ] A function documented to take one encoding accepts only that one.
      An SPKI parser that falls back to a parser that also takes bare PKCS#1
      returns keys missing the parameters the SPKI would have carried.
- [ ] Parameters parsed from a key (PSS hash and salt length, say) are bound
      to the key object and checked at use, not only validated at parse.
- [ ] Nested encodings use the narrowest parser: the BIT STRING inside an
      id-RSASSA-PSS SPKI is a PKCS#1 RSAPublicKey, not another SPKI.
- [ ] Don't keep a copy of the caller's message in state when a digest will
      do. A copy forces an arbitrary length cap the RFC doesn't have.
- [ ] The cost of rejecting bad input is bounded (no attacker-controlled loop
      counts or allocations).
- [ ] Rejection sampling masks the candidate to the bound's bit length before
      comparing. Otherwise a fixed retry cap fails most of the time for sizes
      just past a byte boundary, such as 1025-bit moduli.
- [ ] Large allocations are fallible all the way through: no `vec![..]`
      on a size the caller controls. On Linux with overcommit,
      `try_reserve` succeeding doesn't mean the pages exist, so the docs
      shouldn't promise an error where the OOM killer may strike instead.
- [ ] A fuzz target's oracle matches the spec. For a KDF that hashes the
      output length (Argon2), a truncated tag isn't a prefix of the full
      one. A wrong oracle reports crashes on correct code.

## Constant time

- [ ] No branch, memory index, or variable-time instruction (division,
      floating point) depends on a secret.
- [ ] New secret paths have a ctgrind test. If valgrind can't see the code
      (AVX-512, VAES, SHA extensions), the SDE or qemu runner covers it.
- [ ] An identity or validity check on a secret-derived point doesn't
      declassify and branch. If Z is the identity exactly when X = w0*M, an
      early return leaks w0 through timing.
- [ ] Values marked secret and later handed out as public output (a random
      prefix or salt, say) are declassified at that point. Otherwise ctgrind
      reports false positives in the caller.
- [ ] Every new entry point is wrapped in `Entry::new_secret()` or
      `Entry::new_public()`.
      Inner helpers don't open their own nested `Entry`.
- [ ] The ctgrind test calls every public function on the secret path, not
      just the main one. SIMD paths valgrind can't run (AVX-512) get a
      randomized comparison against the scalar version and an SDE run.
- [ ] A comment that names a test names one that exists.
- [ ] Comments and `AGENTS.md` don't claim ctgrind coverage that the test
      doesn't actually give.
- [ ] Types holding secret-derived values don't derive `Debug` or
      `PartialEq`: write a redacting `Debug` and a constant-time equality.
- [ ] Constants are stored in the form the code uses, not decoded lazily.
      A `OnceLock` init pulls std's panic path into the variable-time scan.
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
- [ ] `heap_scratch` covers every heap buffer that holds a secret, in
      `verify` as well as `derive`.

## API

- [ ] Misuse fails to compile where a type can enforce it, and the docs
      don't claim a compile error that the types don't actually give.
- [ ] Matches on rind's own enums are exhaustive, with no `_` arm, so a new
      variant breaks the build instead of mapping silently.
- [ ] One way to do each job. Borrowed ring/aws-lc-rs names are only used when
      they're as clear, and `MIGRATING.md` maps them.
- [ ] Adapter crates expose exactly their upstream provider's API, nothing
      added.

## Performance

- [ ] Performance-sensitive changes include before/after numbers against
      aws-lc-rs on the CPUs the author has.
- [ ] Any gap over 10% has a quest or a reason in `benchmarks/gaps.toml`.
