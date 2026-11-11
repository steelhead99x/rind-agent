---
name: rind-quest-run
description: Use when working rind's planned backlog with quest (`/quest-spawn`), either yourself or by handing it to a coding agent.
---

# Working the rind backlog with quest

rind's planned work lives in `quest/`. The repo's `.claude/skills/quest-*`
stubs call the `quest` CLI, which carries the real instructions.

## Run it yourself

1. Install and pin `quest` per <https://github.com/moq-dev/quest/blob/main/SETUP.md>.
   Confirm `quest --version` and `quest check`. Without Nix, build the commit
   the flake pins; don't use a GitHub release the setup guide says to skip.
2. `quest context` and follow it.
3. `quest skill spawn` and follow its output to start quests in parallel.
4. Each quest: its own branch, the `rind-contribute` steps, the
   `rind-crypto-review` checklist, then a PR.

## Working from a fork

`quest start` opens its claim PR against the repo that holds `quest/README.md`,
which for a fork is moq-dev/rind. Don't run it until the work is meant to go
upstream. Until then, branch by hand and treat the quest file as the spec.

## Picking a quest

Run `quest unblocked`, then skip anything it marks held, anything with an
open upstream PR or branch (`gh pr list --repo moq-dev/rind`), and anything
waiting on an upstream release that hasn't shipped. Also skip work needing
hardware or accounts you don't have, and proofs you can't check with
`admin/prove`. Prefer missing functionality and correctness over
micro-optimizations.

## Done means the Plan is met

Read the quest's Plan section line by line and mark each item met or not met.
Every vector set and differential test it names is required. When one is
impossible, record the exact reason in the PR's Follow-ups as a question for
the maintainer, for example a reference implementation that is `pub(crate)`
in its latest release while `deny.toml` denies git dependencies. Never push a
partial quest.

A finished quest's PR also deletes its quest file and updates the docs that
list what rind offers: `ALGORITHMS.md`, `TESTING.md`, and the README's table.
When aws-lc-rs lacks the algorithm, use the reference crate the Plan names
for the differential test, and say in Alternatives that there is no aws-lc-rs
benchmark.

## Hand it to a coding agent

Give the agent the outcome, not a script:

- the repo (usually a fork) and that PRs go to the fork's `main` only;
- to install quest, run `quest skill spawn`, and pick well-scoped,
  independently mergeable quests first (correctness, verification, missing
  algorithms, build and infra before speculative micro-optimizations);
- one branch and one PR per quest, `just check` green, benchmarks where
  performance changes;
- never weaken constant-time properties or delete tests or proof checks to get
  green;
- report: quest setup result, quests picked and why, one PR link and line per
  quest, test and bench results, blocked quests (for example ones needing
  hardware the agent doesn't have), and an honest quality and risk read.

To run several agents at once, give each one quest whose `## Required`
section is empty or already merged, and check open upstream PRs first so two
efforts never share a quest. Quests that build on each other (hash-to-curve,
then FROST and the OPRF) wait for their base to merge.

Change direction mid-run with a short steer, not a restart.

## Before going upstream

1. Review each fork PR against `rind-crypto-review`.
2. Merge `main` into the branch; never rebase or force-push one that is
   already pushed upstream.
3. Run `just ci` and the ctgrind tests for new secret paths on the final head.
4. Run `/quest-review` with the other vendor's model and put its `## Review`
   record, naming the head SHA, in the PR body. If it can't run, say so in
   that section rather than leaving it out.
5. Open the upstream PR as a draft while its body lists a decision the
   maintainer still owns; otherwise mark it ready.
