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
upstream. Until then, branch by hand off upstream `main` and treat the quest
file as the spec.

## Picking a quest

Run `quest unblocked`, then skip anything it marks held, anything with an
open upstream PR or branch (`gh pr list --repo moq-dev/rind`), and anything
waiting on an upstream release that hasn't shipped. Also skip work needing
hardware or accounts you don't have, and proofs you can't check with
`admin/prove`. Prefer missing functionality and correctness over
micro-optimizations.

Pick against current upstream `main`, not the fork's. A fork that lags
upstream will still list quests the maintainer merged that day. Confirm
the quest file still exists on upstream `main`, and search recently
merged upstream PRs for it. Base a new quest branch on upstream `main`.

Before starting a follow-up a PR body still lists (a SIMD path, a missing
backend), read the current branch and the later commits first. The work may
already have landed on that branch.

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
2. Search upstream `main` and recently merged PRs for an equivalent change
   (the same `just` recipe, test target, or title). If it already landed,
   stop and say so rather than opening a duplicate.
3. Rebuild the branch on current upstream `main`. If the fork's `main`
   carries files upstream does not (a review config, for example), the
   branch must not include them, even in history. Cherry-pick the quest's
   own commits onto upstream `main`, and drop merge commits from the fork
   and stray artifacts (for example `mutants.out`). Then check that
   `git diff upstream/main --stat` shows only the quest's files. Push the
   rebuilt branch under a new name; don't force-push a branch someone has
   already reviewed. Once a branch is already pushed upstream, merge
   updates into it; never rebase or force-push it.
4. After rebuilding, check upstream's open PRs for overlap: list the files
   each open PR changes, intersect them with yours, and trial-merge the
   overlapping heads locally. Say in the body which PR should land first,
   or that this one rebases cleanly on either.
5. Reconcile against upstream drift (a constant or API removed upstream)
   in a separate small commit, and name it in the Review section as "not
   part of the earlier rounds". Re-run the review on the final head if
   possible, so the table names exactly what gets merged.
6. Run `just ci` and the ctgrind tests for new secret paths on the final head.
   Re-measure benchmark numbers on the rebuilt head, or state the hardware
   and date of the earlier figures.
7. Run `/quest-review` with the other vendor's model and put its `## Review`
   record, naming the head SHA, in the PR body. If it can't run, say so in
   that section rather than leaving it out.

   - Cloud VMs usually lack the reviewer CLIs (`claude`, `codex`), so
     `quest review` exits 2 there. A workstation with Claude Code or Codex
     logged in can run it. Build `quest` from the commit the repo's flake
     pins; Bun is enough to build it without Nix.
   - Review against the same base the PR targets. For an upstream PR, that
     is upstream `main`, not the fork's. Review tables must name upstream
     commits as bases. A SHA that exists only on the fork means nothing to
     the maintainer, so map it to the matching upstream `main` commit, or
     say "upstream main as of <date>".
   - If the clone's remote is a local mirror, quest can't write the record
     into the PR. Paste it in yourself, along with the head SHA it reviewed.
   - A low-effort round finds obvious bugs, but it skips the constant-time,
     vector and wiping depth that `REVIEW.md` asks for. Ask the reviewer for
     that depth explicitly, or follow the round with a `rind-crypto-review`
     pass.
   - Low-severity findings are fixed or refuted in new commits, with no
     further round. Anything medium or higher gets another round on the new
     head.
   - After a round, answer every finding by its number: fixed (with the
     commit) or refuted (with evidence). Then run the next round with
     `--responses`. A medium-effort round found real bugs that a
     low-effort round missed, so don't treat a clean low-effort round as
     a pass.
   - When only low findings remain, fix them and add a line under the
     review record naming the new head's full SHA and how each was
     handled. A medium or higher finding means another round on the new
     head, even when the fix is one line.
   - If the base branch moves between rounds, review against the new base
     and record which base each round used.
   - Include before/after numbers in a response when a fix touches a hot
     path, so the next round can check the claim.
   - Don't rerun a round while its findings are still open. Fix or refute
     them first, and give the reviewer the responses file along with the
     head SHA it answers.
8. Open one upstream PR as a draft first, and send the rest after the
   maintainer has responded to it. Several large drafts at once from a new
   contributor is a lot to review. Lead with a standalone change, and hold
   ones that overlap the maintainer's open work.

   - Keep it a draft while its body lists a decision the maintainer still
     owns; otherwise mark it ready.
   - Ask a scoping question (for example, which differential reference is
     acceptable) as one short PR comment that names the constraint and the
     options. Don't open an issue for it when the repo tracks work in quest
     files.
   - Before publishing, check that the body has no tooling markers or
     footers (HTML comment markers, "open in" badges).
