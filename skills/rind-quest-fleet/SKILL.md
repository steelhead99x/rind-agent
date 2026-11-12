---
name: rind-quest-fleet
description: Use when running many rind quests in parallel from a fork, then reviewing them and sending ready PRs upstream.
---

# Running rind quests in parallel from a fork

Inputs are the fork `<fork>` and upstream `moq-dev/rind`. Follow
[rind-quest-run](../rind-quest-run/SKILL.md) for picking, overlap, and what
a finished quest contains. This skill is the parallel launch, the review
loop, and the gate before anything is opened on moq-dev/rind. When a
launched agent claims a quest, use the claim step in
[Launch one agent per quest](#launch-one-agent-per-quest): `quest start`
after the overlap check, and the fork draft PR when that upstream open
returns HTTP 403.

## Sync the fork

Ask before updating `<fork>`'s `main`, unless you were already told to
sync. Then follow
[Sync the fork first](../rind-quest-run/SKILL.md#sync-the-fork-first): a
merge commit from `moq-dev:main`, never a force-push or a reset.

## Map in-flight work

Before picking, follow
[Picking a quest](../rind-quest-run/SKILL.md#picking-a-quest), including
`quest unblocked --unclaimed` when a GitHub token is available. Add the
sources a PR list misses: work started by hand outside your agent
orchestrator, and agents that orchestrator already has running on the
backlog. Build the file list that section names (shared CI configs such
as `mutants.yml` and `fuzz.yml`, `admin/prove`, and shared directories
under `proofs/`) for every in-flight effort. Skip anything claimed or
already in progress.

## Pick independent quests

Use a read-only helper for the ranking. It does not push or open a PR.
It clones `<fork>` and `moq-dev/rind`, builds `quest` from the commit
rind's flake pins, and runs `quest check` and `quest unblocked`. Install
and pin `quest` as
[Run it yourself](../rind-quest-run/SKILL.md#run-it-yourself) says.
Without Nix, from that pinned checkout, run
`bun install --frozen-lockfile && bun run build`, then
`bun <checkout>/dist/quest.js` (quest `package.json` and README). Do not
run `src/cli/bin.ts` with a working directory outside that checkout. Bun
reads `bunfig.toml` from the working directory, and without its `.md`
text loader the skill import fails with
`skill afk does not open with frontmatter` (quest `src/cli/skills.ts`).

Skip anything
[Picking a quest](../rind-quest-run/SKILL.md#picking-a-quest) skips, plus
anything in the map above and anything that depends on another quest in
this batch. Upstream `main` is the tree that counts. Return a ranked
list: path, a one-line plan, why it is independent of the others, and
the excluded list with one reason each.

An epic README becomes ready once its last child has merged. The README
is the epic's own quest (quest README; `isEpic` in quest
`src/core/doc.ts`). Rank it like any other ready quest, and follow the
docs-only epic rule in
[Done means the Plan is met](../rind-quest-run/SKILL.md#done-means-the-plan-is-met).
Milestones are never removed.

## Launch one agent per quest

Launch one agent per quest, on `<fork>`. Each brief names exactly one
quest path and its plan in one sentence. It tells the agent to follow
the linked skills, and to claim as follows.

1. Re-check overlap as
   [Picking a quest](../rind-quest-run/SKILL.md#picking-a-quest)
   requires, immediately before the claim. If someone else holds the
   quest, stop, open no PR, and report the PR or branch that holds it.
2. Open one draft PR on `<fork>` against its `main` before writing the
   change, and name the quest path in it. The claim branch is the quest
   path without `.md`. Give the fork draft a different branch: `quest
   start` uses `--force-with-lease` on the claim branch, and a branch
   you already pushed there is a refusal. That draft is the visible
   claim on the fork.
3. Then run `quest start <path>` (or `.claude/skills/quest-start`, which
   runs `quest skill start`). `quest start` pushes
   `chore(quest): claim <path>` and opens a draft PR on the repository
   that holds `quest/README.md`. For a fork of moq-dev/rind that
   repository is moq-dev/rind, and the PR head is
   `<fork>:<quest-branch>` (quest README, "Claim a quest"; quest
   `src/cli/start.ts`).
4. Fall back only when that open fails because the token cannot create
   the upstream PR (GitHub HTTP 403). `quest start` then deletes the
   claim branch it pushed, and the fork draft from step 2 is the claim.
   A refusal is different: the quest is held, blocked, or missing on
   upstream `main`. Stop, and do not start a second effort.
5. The feature work lands on the fork draft, as one PR against
   `<fork>`'s `main`. If `quest start` succeeded, leave its claim branch
   at the empty claim commit. A later push to that branch updates the
   upstream PR. Never open a feature PR on moq-dev/rind from the agent.
   That waits for [Going upstream](#going-upstream).

The rest of the brief points at the skills that already state the rules:

- Base the branch on upstream `main` and carry no fork-only commits
  ([Remotes and base](../rind-git/SKILL.md#remotes-and-base)).
- One conventional-commit title, and `just check` green. Pass the
  upstream base (`just check <upstream>/main`; `BASE` defaults to the PR
  base, then a main upstream, per the rind `justfile`). Run it inside
  Nix as [Gate locally](../rind-contribute/SKILL.md#gate-locally)
  describes.
- Never weaken constant-time properties or delete tests or proof checks
  ([Hand it to a coding agent](../rind-quest-run/SKILL.md#hand-it-to-a-coding-agent)).
- Meet the Plan, record anything impossible under Follow-ups, and do
  not push a partial quest. The finished PR deletes the quest file and
  updates the capability docs
  ([Done means the Plan is met](../rind-quest-run/SKILL.md#done-means-the-plan-is-met)).
- Do not edit sibling quest files or shared docs an open upstream PR
  already touches
  ([Stay inside the quest](../rind-quest-run/SKILL.md#stay-inside-the-quest)).
- Note overlapping open PRs and trial-merge them
  ([Before going upstream](../rind-quest-run/SKILL.md#before-going-upstream)).
- Report the PR link, which Plan items are met and which are not, the
  test, bench, and proof results, blockers, and an honest risk read.

## Report back

Report which agents started, with links; which quests are now claimed,
so nobody starts them by hand; what was excluded, in one sentence each;
and any caveat. A large proof quest can come back blocked under the
feasibility check in
[Picking a quest](../rind-quest-run/SKILL.md#picking-a-quest). Also flag
fork PRs whose quest file is gone on upstream `main`. A finished quest
deletes its file (quest README), so those PRs are likely superseded.

## Review each result

When an agent finishes:

- Review crypto changes with
  [rind-crypto-review](../rind-crypto-review/SKILL.md). Send every
  finding back to that agent as one follow-up.
- Then give a short list for a second pass by the person who asked,
  including `/quest-review` when the VM has no reviewer CLIs
  ([Before going upstream](../rind-quest-run/SKILL.md#before-going-upstream)).
- Bring a reusable lesson back to this skills repo as one focused PR.
  Merge it only when CI is green and that merge was allowed. Hold a PR
  that reverses a rule a skill already records, even when CI is green.
- A scheduled run can reply only to agents launched in that same run.
  It cannot send follow-ups to agents an earlier interactive session
  launched. Write each follow-up in full (which agent, the PR link, the
  exact prompt) and hand it to the session that launched them. Once
  watching an agent has been approved, any session can reply to it.
- An agent that wakes because it watches CI, and finds no change, stays
  quiet. Do not report that wake.
- On every pass, re-check each running agent's quest and each open fork
  quest PR with
  [Picking a quest](../rind-quest-run/SKILL.md#picking-a-quest).
  Maintainers claim many quests at once by draft PRs on `quest/<path>`
  branches (`quest start` opens those drafts). When upstream now covers
  the same quest, flag the fork PR as superseded. Close it only when
  asked, and keep the branch. Recommend cancelling that agent and any
  agent working on the PR's follow-ups.
- Commits pushed by hand onto a quest branch, such as an upstream
  merge, stay. Follow-ups keep them
  ([Updating a pushed branch](../rind-git/SKILL.md#updating-a-pushed-branch)).
  If another open PR contradicts those commits, raise that on the other
  PR.

## Verify what pull-request CI skips

`.github/workflows/build.yml` does not run `build`, `musl`,
`i686-windows`, `wasm`, `qemu`, `ios`, or `android` on a pull request
(`if: github.event_name != 'pull_request'`). A pull request runs only
the path-filtered jobs. rind `CONTRIBUTING.md` starts the skipped
workflows on demand with `gh workflow run`. Creating a workflow
dispatch needs Actions write on moq-dev/rind; without it GitHub returns
HTTP 403
([Create a workflow dispatch event](https://docs.github.com/en/rest/actions/workflows#create-a-workflow-dispatch-event)).
Run the skipped jobs locally, at the PR head and at the parent of the
fixing commit, so the result changes with the fix.

The `just` recipes for the cross builds are in
[Other architectures](../rind-bench-and-verify/SKILL.md#other-architectures).
The `android` and `ios` jobs have no `just` recipe; follow their steps
in `build.yml`. On macOS, `xcrun` and `simctl` need the Xcode developer
directory. When `xcode-select -p` points at the Command Line Tools, set
`DEVELOPER_DIR` to `/path/to/Xcode.app/Contents/Developer`.

On a machine that already has rind checkouts, verify from a separate
clone, one worktree per PR, with an isolated `CARGO_HOME`. Do not change
that machine's other checkouts or its toolchain config. Remove each
worktree's `target/` after the check.

Run `just check` from a Nix dev shell recorded as a profile:

```shell
nix develop --profile <logs>/dev-profile --command bash --noprofile --norc -c 'just check <upstream>/main'
```

`--profile` records that shell, and the profile is a GC root, so a
garbage collection cannot delete tools such as `cargo-modules` (rind
`flake.nix`) mid-run. `nix develop --command` without `--profile` does
not leave that root (Nix manual, `nix develop`). Do not collect garbage
while the check runs. `--noprofile --norc` keeps a login `PATH` from
shadowing the flake. The same PATH rule, for `bash -lc`, is in
[Gate locally](../rind-contribute/SKILL.md#gate-locally).

## Going upstream

Open an upstream PR only when that permission was granted
([Working from a fork](../rind-contribute/SKILL.md#working-from-a-fork))
and every item below is true:

- `<fork>` CI is green.
- No review finding is still open.
- The second-pass items are resolved.
- `/quest-review` and `just check` have run
  ([Before going upstream](../rind-quest-run/SKILL.md#before-going-upstream)).
- The branch does not conflict with upstream work, and the overlap
  check shows no overlap.

Never merge an upstream PR. Landing on moq-dev/rind is the maintainer's
`quest merge` (rind `CONTRIBUTING.md`).

## When nothing unclaimed is left

Do not take claimed work
([Picking a quest](../rind-quest-run/SKILL.md#picking-a-quest)). Deepen
the open fork PRs with tests, fuzzing, and constant-time checks
([rind-bench-and-verify](../rind-bench-and-verify/SKILL.md)). Broken CI
on upstream `main` is m0 work (`quest/README.md`). A reusable lesson
still comes back here as one focused PR, as
[Review each result](#review-each-result) says.
