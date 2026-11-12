---
name: rind-contribute
description: Use when making a code change to rind — setup, the change itself, and the local gate.
---

# Contributing a change to rind

rind is one Rust workspace: the `rind` crate, `rind-<consumer>` adapter crates
(`rind-rustls`, `rind-ring`, `rind-aws-lc-rs`, ...), and unpublished bench,
cpuid, timing and fuzz crates. Read `AGENTS.md`, `ARCHITECTURE.md`,
`PLATFORMS.md` and `CONTRIBUTING.md` before touching code, plus any
`AGENTS.md` nested under the files you change.

## Setup

1. `nix develop` (or direnv) for CI's toolchain. Without Nix, a stable Rust
   toolchain and `just` cover the basics.
2. `git submodule update --init thirdparty/cavp thirdparty/cctv thirdparty/wycheproof`
3. Install `quest` per <https://github.com/moq-dev/quest/blob/main/SETUP.md>,
   pinned through the repo's flake. Then run `quest context` and follow it
   before any work.

## Make the change

1. One quest or one problem per branch, off `main`.
2. Keep to the `low`/`mid`/`high` layering `ARCHITECTURE.md` describes, and
   shape new public API so misuse fails to compile.
3. Exactly one backend compiles per target. Never add a runtime fallback; a
   missing CPU feature is an error that names the feature.
4. New primitive or new path: Wycheproof vectors where they exist, plus a
   differential test against aws-lc-rs (a dev-dependency only).
5. New secret-handling path: a ctgrind test (`admin/ctgrind`).
6. Before starting a follow-up the branch's PR still lists (a SIMD path, a
   missing backend), read the later commits. The work may already be there.
7. Gate every test or helper that touches aws-lc with
   `#[cfg(not(any(miri, target_family = "wasm")))]`. A missing gate breaks
   `just wasm` and the wasm nightly, not only Miri
   (`rind/tests/heap_scratch.rs` after ML-DSA).

## Gate locally

```shell
just fix     # format and auto-fix lint, scoped to what the branch changed
just check   # lint, compile and test what the branch changed (CI's gate)
cargo test   # full suite when the change is broad
```

Things that trip `just check` on new code:

- A `pub(crate)` function with no caller yet fails clippy's dead-code lint on
  the non-test build. `#[cfg_attr(not(test), allow(dead_code))]` on the entry
  points is enough until the first consumer lands.
- `admin/testing` treats a unit test named `aws_lc*` as a vector test that
  must be listed in `TESTING.md`. Name an internal differential test
  something else.
- A trait with methods named `add`, `mul` or `sqr` shadows the curve types'
  methods once it's in scope. Prefix them (`field_mul`).
- `nix develop --command bash -lc '...'` starts a login shell that drops the
  dev shell's `PATH`, so you get the host's older Cargo. On macOS a login
  zsh (`path_helper` and `brew shellenv`) puts Homebrew's `cargo` ahead of
  the flake toolchain. An isolated `CARGO_HOME` has no `bin/`, so cargo
  subcommands come from `PATH`. `nix develop --command` also does not
  register a gcroot. `.envrc` pins only the shell direnv loaded, then runs
  `nix-collect-garbage --delete-older-than 7d` in the background at most
  once a week. The age applies to old profile generations; unrooted paths
  are deleted anyway, and a manual `nix-collect-garbage` does the same.
  Either can remove `cargo-modules` while `just check` is still running.
  `sh/dispatch.sh` looks tools up once at startup, so it still runs
  `structure` (when `rind/src`, `rind/Cargo.toml`, or `admin/structure`
  changed) and `cargo modules` fails with `no such command: modules`.
  Root the shell from outside the checkout and skip startup files:
  `nix develop --profile <dir>/rind-dev-profile --command bash --noprofile --norc -c 'just check <base>'`.
  Use the same form for `just ci`. Do not collect Nix garbage while a check
  is running.
- In an integration test or fuzz target, `use rind::foo::{self}` hides an
  external crate also called `foo`. Write `::foo` for the external crate.
- `just ci` clippy-checks every crate that depends on `rind`, including a new
  fuzz target, so lint those too.
- A rustup shim on `PATH` (for example from a host package manager) with no
  default toolchain can shadow the Nix dev shell's `cargo-fmt`. Keep the Nix
  shell's `PATH` first, or set a rustup default. Inside `nix develop`,
  export `RUSTUP_TOOLCHAIN` to the dev shell's toolchain directory
  (`$(dirname $(dirname $(which cargo)))`) so the shim cannot shadow
  `cargo fmt` or clippy.
- `cargo test --all-features` turns on rind's `__ctgrind` feature, which
  needs valgrind headers. Without them, bindgen fails to find
  `valgrind/valgrind.h`. Locally, run default features plus the specific
  features a test needs, and leave ctgrind to `admin/ctgrind` in the
  `.#ctgrind` Nix shell, or to CI.
- In a shell script, a failure inside `$(...)` in an assignment can be
  silently ignored. Check the status explicitly, and add a self-test for
  that failure path.

Touching performance: run the relevant `cargo bench` before and after, and
compare against aws-lc-rs. Put the numbers in the PR, with the host and the
date. For the published report's operations and timing, follow
[rind-bench-and-verify](../rind-bench-and-verify/SKILL.md): that run is
`admin/bench-report`, and an unfiltered `cargo bench` times a different set.

## Git, commits and pull requests

Branching, trailers, updating a pushed branch, and squash-merges are in
[rind-git](../rind-git/SKILL.md). Follow that before the first commit.
Run `just ci` before every push.

Before a branch goes upstream, `nix develop --command just check` has
passed on that head. A pass on an older head does not count. The rest of
that gate is in
[Before going upstream](../rind-quest-run/SKILL.md#before-going-upstream).

A fix for a CI job that used to pass names the change that broke it and
why. The Android armv7 job was green at `fcfb32f8` and failing from
`1175aab5`.

Before opening the pull request, check `UPSTREAM.md`. It lists every
workaround for a bug or gap in a tool or dependency rind doesn't own.
A pull request that adds one adds its entry and names that entry, by
its heading, in the workaround's comment (`See UPSTREAM.md, "<heading>".`).
A pull request that drops one removes it from its entry, and deletes
the entry once it lists no workaround. The rule is rind's
`CONTRIBUTING.md`, under Upstream.

A pull request that touches the platform workflow inputs also gets a
cheap subset of those workflows: one ctgrind row, the armv7, portable,
and Dart jobs of `build`, and a timing build. These are advisory, but
fix a red one before merging. Mutation evidence is in
[Mutation testing](../rind-bench-and-verify/SKILL.md#mutation-testing).
The list is in `CONTRIBUTING.md`. An instruction-set or cross job the
author cannot run locally stays open until that job is green. The check
is in [Platform rules](../rind-crypto-review/SKILL.md#platform-rules).
A pull request skips the `android` job; run it with the QEMU recipe in
[Android under QEMU](../rind-bench-and-verify/SKILL.md#android-under-qemu).

## Working from a fork

Open pull requests against the fork's `main` first. Do not contact the
upstream maintainer, and do not open, comment on, or update an upstream
pull request or issue, unless the fork owner has approved the exact text.
Collect questions for the maintainer and wait for that approval before
posting. Send a fork change upstream only after the fork owner has
reviewed it and approved the exact text. Before new work, sync the fork
with upstream as in
[Sync the fork first](../rind-quest-run/SKILL.md#sync-the-fork-first).
For remotes, branch names, and how to rebuild an upstream-bound branch,
follow [rind-git](../rind-git/SKILL.md), including fetching both remotes
before every step and before every push. The rest of the upstream
sequence is in
[Before going upstream](../rind-quest-run/SKILL.md#before-going-upstream).
