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

## Gate locally

```shell
just fix     # format and auto-fix lint, scoped to what the branch changed
just check   # lint, compile and test what the branch changed (CI's gate)
cargo test   # full suite when the change is broad
```

Touching performance: run the relevant `cargo bench` before and after, and
compare against aws-lc-rs. Put the numbers in the PR.

## Commit and PR

- Conventional-commit subjects: `feat(rustls): ...`, `fix: ...`, `perf: ...`,
  `docs: ...`, `chore: ...`. Breaking change: `feat!: ...`.
- PRs squash onto `main`, so the PR title becomes the commit subject.
- AI attribution goes in a `Co-Authored-By:` trailer only.
- No binaries or build artifacts.
- PR body, short and structured: **Problem**, **Approach**, **Impact** (one
  bullet per public API change, with migration for breaking ones),
  **Alternatives**, **Follow-ups** (issues hit, quests created).

## Working from a fork

Open PRs against the fork's `main` first. Send anything upstream only after
the fork owner has reviewed it and said so.
