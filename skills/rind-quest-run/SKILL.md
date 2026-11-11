---
name: rind-quest-run
description: Use when working rind's planned backlog with quest (`/quest-spawn`), either yourself or by handing it to a coding agent.
---

# Working the rind backlog with quest

rind's planned work lives in `quest/`. The repo's `.claude/skills/quest-*`
stubs call the `quest` CLI, which carries the real instructions.

## Run it yourself

1. Install and pin `quest` per <https://github.com/moq-dev/quest/blob/main/SETUP.md>.
   Confirm `quest --version`.
2. `quest context` and follow it.
3. `quest skill spawn` and follow its output to start quests in parallel.
4. Each quest: its own branch, the `rind-contribute` steps, the
   `rind-crypto-review` checklist, then a PR.

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

Change direction mid-run with a short steer, not a restart.

## Before going upstream

Review each fork PR against `rind-crypto-review`. Only then open the matching
PR on moq-dev/rind, with the fork PR's numbers in the body.
