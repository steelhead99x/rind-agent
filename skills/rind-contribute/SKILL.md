---
name: rind-contribute
description: Use when making any code change to rind (moq-dev/rind or a fork) — setup, branch, local gate, commit and PR format.
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
  dev shell's `PATH`, so you get the host's older Cargo. Use
  `nix develop --command cargo ...` or `bash -c` without `-l`.
- In an integration test or fuzz target, `use rind::foo::{self}` hides an
  external crate also called `foo`. Write `::foo` for the external crate.
- `just ci` clippy-checks every crate that depends on `rind`, including a new
  fuzz target, so lint those too.

Touching performance: run the relevant `cargo bench` before and after, and
compare against aws-lc-rs. Put the numbers in the PR, with the host and the
date.

## Commit and PR

- Conventional-commit subjects: `feat(rustls): ...`, `fix: ...`, `perf: ...`,
  `docs: ...`, `chore: ...`. Breaking change: `feat!: ...`.
- PRs squash onto `main`, so the PR title becomes the commit subject.
- AI attribution goes in a `Co-Authored-By:` trailer only, never in the
  subject, body or PR text.
- Check the trailer count with `git log -1 --format=%B`; some environments
  append their own trailer on `git commit` (a commit-msg hook, for one).
  Rewrite the message with `git -c core.hooksPath=/dev/null commit --amend`
  or build it with `git commit-tree` before the first push.
- When squash-merging, pass the documented author email explicitly
  (`gh pr merge --squash --author-email`) and a body with exactly one
  `Co-Authored-By:` trailer, then verify the resulting `main` commit's
  author, committer, and trailer count.
- No binaries or build artifacts.
- PR body, short and structured: **Problem**, **Approach**, **Impact** (one
  bullet per public API change, with migration for breaking ones),
  **Alternatives**, **Follow-ups** (issues hit, quests created).

## Working from a fork

Open PRs against the fork's `main` first. Send anything upstream only after
the fork owner has reviewed it and said so. Keep the fork's `main` a
fast-forward of upstream's (`gh repo sync <fork> --branch main`) so branches
start from what upstream will merge into.

A branch headed upstream must not carry fork-only files (a review config,
for example), even in history. After rebuilding it on upstream `main`,
check open upstream PRs for overlapping files, trial-merge those heads,
and open one draft first: lead with a standalone change and hold work that
overlaps the maintainer's open PRs. The full sequence is in
[Before going upstream](../rind-quest-run/SKILL.md#before-going-upstream).
