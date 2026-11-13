---
name: rind-crypto-review
description: Use when reviewing or self-checking a rind diff for correctness, constant time, hostile input, wiping, and API shape.
---

# Reviewing a rind change

Check the diff against each rule. Any "no" blocks the merge until fixed or
explained in the PR.

## Correctness

- [ ] Every new or changed primitive is tested against Wycheproof where vectors
      exist, against NIST ACVP samples where `TESTING.md` lists them, and
      differentially against aws-lc-rs.
- [ ] Where aws-lc cannot build (wasm, for example), check verdicts against a
      file recorded from aws-lc, plus a host test that keeps the file equal
      to aws-lc. Never accept "either verdict" as a check.
- [ ] Every test or helper that touches aws-lc is gated with
      `#[cfg(not(any(miri, target_family = "wasm")))]`. A missing gate
      breaks `just wasm` and the wasm nightly, not only Miri
      (`rind/tests/heap_scratch.rs` after ML-DSA).
- [ ] New arithmetic says where its correctness argument comes from (a proof,
      a cited paper, or the upstream it was taken from).
- [ ] Code copied from another project keeps its copyright lines and an
      `SPDX-License-Identifier`, and `LICENSE` and `ALGORITHMS.md`'s Sources
      table both list the project.
- [ ] A committed verdict or vector file has a drift check and a documented
      way to regenerate it, such as an ignored test that re-records the file
      from the reference implementation.
- [ ] Behavior questions (accept, refuse, wipe) follow aws-lc-rs unless the PR
      says why not.
- [ ] A known difference from upstream is documented in the README, pinned by
      a test, and named in the PR as a maintainer decision. secp256k1 PKCS#8
      returning `InvalidEncoding` where aws-lc-rs returns `WrongAlgorithm`
      is one.
- [ ] A comment or PR sentence that states what an upstream parser does cites
      the upstream file and the lines it was checked against. "aws-lc checks
      ML-KEM by length only" and "PSS default parameters" are that kind of
      claim.
- [ ] Every key encoding the RFC requires is accepted. RFC 9474, for example,
      needs id-RSASSA-PSS SPKIs as well as rsaEncryption.
- [ ] When a modulus' bit length is 1 mod 8, the byte dropped from an encoding
      is checked to be zero rather than ignored. The same check applies to any
      unused high bits, or the octet above `emLen`: if they should be zero and
      are not, reject the encoding.
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
- [ ] Differential tests cover every constructor and entry point (seed,
      expanded key, `generate`, PKCS#8 import and export, signing) and
      malformed inputs: wrong lengths, mismatched parameter sets, and a
      PKCS#8 whose public key does not match.
- [ ] A shared primitive added for one feature is used only by that
      feature, or the other callers are checked and their behavior is
      unchanged. Portable `edwards25519::Point::from_bytes`, added for
      ECVRF, is checked against Ed25519.

## Hostile input

- [ ] Malformed keys, signatures, ciphertexts and encodings return an error,
      never panic. Hostile input gets a protocol error, not `Fault`, which
      is reserved for internal bugs.
- [ ] A new `Fault` path, such as a SPAKE2+ encode failure, is unreachable
      from peer-controlled input. If a peer can reach it, the PR says
      which protocol error it returns instead.
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
- [ ] Map errors with typed error values or variants, never by matching
      error message text.
- [ ] When rejecting a key for the wrong algorithm or parameter set,
      check the OID or parameter set before any key expansion. Don't
      reparse the key under each alternative to decide.
- [ ] An adapter's error variant matches the upstream parser. `WrongAlgorithm`
      versus `InvalidEncoding` follows every type whose `priv_decode` is set.
      aws-lc's `EVP_parse_private_key` (`crypto/evp_extra/evp_asn1.c`, methods
      in `p_methods.c`) decodes RSA, RSA-PSS, EC, DSA, Ed25519, X25519,
      ML-DSA, and ML-KEM. DH is in that table and has no `priv_decode`. Each
      type rind does not recognize gets a differential test or a documented
      difference.
- [ ] That parity covers each encoding of an algorithm, not only its OID.
      The upstream parser may treat the forms differently: an ML-KEM seed
      versus an expanded private key; RSA-PSS with absent, empty, or
      explicit-hash parameters; OAEP; id-ea-rsa. Give each form a
      differential test. Where upstream accepts a form and rind cannot match
      it (DSA, secp224r1, secp256k1, Brainpool, PSS with an explicit hash),
      list it in the README and name it as a maintainer decision.
- [ ] A new failure mode uses the error variant of the closest existing rind
      or upstream case. Hash-to-curve exhaustion on verify, or a proof from
      a different suite, is that kind of choice. When the closest case is
      unclear, list it as a maintainer question and leave the PR a draft.
- [ ] A hash-to-curve or `expand_message` entry point refuses an empty
      domain-separation tag, a requested length above the RFC's 65535-byte or
      255-block limit, and a reduction wider than the bignum helper can read.
      An oversize DST is replaced as the RFC requires, not truncated.
- [ ] An error reserved for a wrong length is not used for a value that is
      the right length but out of range. A signature of length `k` that is
      `>= n` is `BadSignature`, not `WrongLength`.
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
- [ ] A ctgrind test's `secret` and `public` on a `Vec<u8>` take the bytes
      (`&v[..]`), not the header. Declassify a secret-derived buffer with
      `public` before comparing it (`assert_eq!`, `==`).
- [ ] Marking that header also leaves the payload unmarked, so a passing
      run can miss coverage. When a ctgrind failure is fixed, say whether
      the change was the test's marking or the code, and confirm the same
      bytes are still covered. A declassify before `assert_eq` covers only
      a true public output or a test artifact, such as a verify `Result`.
- [ ] A ctgrind test that leaves octets unmarked says why, against the parser
      that reads them, and cites those lines in a comment. Leading octets of
      an RSA private integer (`rind/src/high/asn1.rs`) and the low octet of a
      prime (`rind/src/mid/rsa_priv.rs`) stay unmarked only when those lines
      show the parser treats them as public.
- [ ] Every new entry point is wrapped in `Entry::new_secret()` or
      `Entry::new_public()`.
      Inner helpers don't open their own nested `Entry`.
- [ ] The ctgrind test calls every public function on the secret path, not
      just the main one, including every constructor and entry point
      (seed, expanded key, `generate`, PKCS#8 import and export, and
      signing). SIMD paths valgrind can't run (AVX-512) get a
      randomized comparison against the scalar version and an SDE run.
- [ ] A secrets test covers every constructor that takes a secret.
      `secrets_stay_secret` includes `VerifierRecord::from_record`, which
      takes `w0`.
- [ ] ctgrind and canonical-encoding tests run on every backend the crate
      builds: native x86_64, `--cfg rind_portable` with
      `curve25519_dalek_backend="serial"`, aarch64, and armv7. A new
      primitive gets a `ct_binsec_*` test or says why not. A non-canonical
      encoding is compared with an established implementation (`akd_core`
      for ECVRF).
- [ ] A comment that names a test names one that exists.
- [ ] Comments and `AGENTS.md` don't claim ctgrind coverage that the test
      doesn't actually give.
- [ ] Wasm test tiers do not check constant time: the engine recompiles and
      may branch. Docs that describe the tier say so.
- [ ] Types holding secret-derived values don't derive `Debug` or
      `PartialEq`. A tag or confirmation value gets a redacting `Debug` and
      a hand-written `ct_equal` that declassifies only the boolean. Nothing
      else compares them outside `cfg(test)`: `as_ref()` slices, `Hash`,
      `Ord`, or serialization.
- [ ] When an expected confirmation tag or MAC stays secret through
      `ct_equal`, the full ctgrind suite is rerun, not only that test.
- [ ] A hash-state change, such as cloning a SHA-512 prefix state, is
      checked for whether every byte absorbed into that state is public.
- [ ] A value computed from a secret and meant to be published (a VRF
      `Output`, for example) has a written decision on whether it is secret.
      While it is secret, or that decision is still open, it does not derive
      a variable-time `PartialEq` or a `Debug` that prints its bytes. An
      open decision is a maintainer question.
- [ ] Constants are stored in the form the code uses, not decoded lazily.
      A `OnceLock` init pulls std's panic path into the variable-time scan.
- [ ] `admin/ct-vartime` still passes; any new division is listed in
      `admin/ct-vartime.toml` only if its operands are public.
- [ ] BINSEC (`admin/ct-binsec`) covers the `ct_binsec_*` tests on x86_64 and
      the portable backend, and a short aarch64 set. It does not run on
      armv7, and it denies CPU features it cannot decode. Those paths stay
      with ctgrind, `admin/ct-sde`, or `admin/ct-qemu`, as `TESTING.md` says.
- [ ] On ARMv6-M, a carry or mask compare must not compile to `cmp`/`blo`
      (or another secret-dependent branch). Check the thumbv6m disassembly
      and an instruction-trace comparison, not only ctgrind. The steps are
      in [rind-armv6m-constant-time](../rind-armv6m-constant-time/SKILL.md).
      A widening-mul patch is not complete without branch-free carries.

## Platform rules

- [ ] Exactly one backend per target; no runtime fallback.
- [ ] A CPU missing a required feature fails with an error naming it.
- [ ] A claim about which platforms or targets are tested matches
      `.github/workflows/build.yml`, `Cross.toml`, and `sh/dispatch.sh`.
      A target that only compiles is not tested: `aarch64-apple-ios` is
      `cargo test --no-run`, and `wasm32-unknown-unknown` is compile-only.
- [ ] ABI or harness code that depends on instruction-set mode is checked
      for every target triple in each mode. `pc_ahead` is 4 under Thumb
      (`target_feature = "thumb-mode"`) and 8 under ARM, so both the Thumb
      triples and `arm-linux-androideabi` are covered. When that cross test
      cannot be run locally, the PR says so and waits for the CI job to be
      green.
- [ ] An instruction-set or target claim (Thumb versus ARM, Vita, Android)
      is checked with `rustc --print target-spec-json` on the pinned
      toolchain. The PR names that `rustc --version` and that it is the
      `rust-toolchain` `flake.nix` pins. rind has no `rust-toolchain.toml`.
- [ ] A local emulator run is evidence, not the CI job. qemu-arm with NDK
      r27c, API 28, and static bionic is not CI's dynamic bionic, and the
      PR names that difference. The CI job is still green. The notes show
      the test failing before the fix and passing after.

## Wiping

- [ ] Key types wipe their whole value on drop, public parts included.
      A zeroing test covers every constructor (seed, expanded key,
      `generate`, and PKCS#8 import), not only one path.
- [ ] Keyed state and heap scratch derived from a private key are wiped through
      `low::zeroise` / `low::zeroise_value`, under the `wipe` feature.
- [ ] Nothing else gets wiped (digest contexts, plaintext, stack temporaries);
      that's speed spent for nothing.
- [ ] `heap_scratch` covers every heap buffer that holds a secret, in
      `verify` as well as `derive`.
- [ ] Wipe and leak canaries report through the same comparison path as the
      real checks, without relying on panics, so they run on every target.
      A refactor that drops `should_panic` still proves, on native, that a
      planted leak is caught.
- [ ] Canaries that call the real `independent` and `zeroed` checks are
      run on wasm32-wasip2 as well as native.
- [ ] A wipe test's needle is long enough that a chance match is not a pass,
      and short enough to catch a partial wipe (16 bytes, for example).
      The test comment says why that length.
- [ ] Error paths wipe the same secret state the success path does. A failed
      `blind`, `finalize`, `derive`, or handshake still drops the types that
      hold the secret, and no early return skips a wipe of heap scratch.
- [ ] `low::guard_pages` still covers each assembly kernel that takes a
      caller's slice. Valgrind cannot decode the fastest tiers and
      AddressSanitizer does not instrument assembly, so this is the bounds
      check on them.

## API

- [ ] Misuse fails to compile where a type can enforce it, and the docs
      don't claim a compile error that the types don't actually give.
- [ ] Matches on rind's own enums are exhaustive, with no `_` arm, so a new
      variant breaks the build instead of mapping silently.
- [ ] One way to do each job. Borrowed ring/aws-lc-rs names are only used when
      they're as clear, and `MIGRATING.md` maps them.
- [ ] Adapter crates expose exactly their upstream provider's API and
      behavior, nothing added.
- [ ] A feature that is accepted but does nothing is a bug. If an adapter
      crate takes a feature name from its upstream (rind-ring's
      `wasm32_unknown_unknown_js`, for example), wire it to what that crate's
      same-named feature does, and clippy-check it on every PR. Gate the
      dependency on the same `cfg` upstream uses.
      `wasm32_unknown_unknown_js` is
      `all(target_arch = "wasm32", target_os = "unknown")`, so
      wasm32-wasip2 (`target_os = "wasi"`) keeps its WASI source.
- [ ] A new public item in the `rind` crate is the change itself. When a
      binding, an adapter, or a performance change only needs the item,
      keep it `pub(crate)`, or ask the maintainer and leave the PR a draft
      while that decision is open. `AGENTS.md` requires one way to do each
      job, and that misuse fails to compile. `CONTRIBUTING.md` lists every
      public API change under Impact. A public
      `seal_in_place_append_tag_with_nonce` on `RandomizedNonceKey`, a
      public `cpu::wide_multiplier`, or a public `drbg` module is a
      maintainer question.
- [ ] An adapter item put behind a cargo feature to match upstream still
      builds. Build that crate with `--no-default-features`, and with
      `--no-default-features --features` set to each feature on its own.
      Its own ungated callers may still use the item. An aws-lc-rs adapter
      method gated on `ring-io`, with its caller left ungated, broke
      `cargo check -p rind-aws-lc-rs --no-default-features` with E0599.

## Performance

- [ ] Performance-sensitive changes include before/after numbers against
      aws-lc-rs on the CPUs the author has, with the host and the date.
- [ ] Any gap over 10% has a quest or a reason in `benchmarks/gaps.toml`.
