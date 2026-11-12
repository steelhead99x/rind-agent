---
name: rind-quest-fleet
description: Use when running more than one rind quest from a fork.
---

# Several rind quests from a fork

Each quest's implementation is one branch from a fork of moq-dev/rind.
Picking, the overlap check, and the upstream gate are in
[rind-quest-run](../rind-quest-run/SKILL.md).

`<fork>` is the fork. `up` is the remote for moq-dev/rind.

## Sync

Before choosing quests, sync `<fork>` as
[Sync the fork first](../rind-quest-run/SKILL.md#sync-the-fork-first)
describes.

## What is already taken

Run the overlap check in
[Picking a quest](../rind-quest-run/SKILL.md#picking-a-quest).
Leave any quest that check marks taken.

## Which quests can proceed together

From upstream `main`, run `quest check` and `quest unblocked`, then
apply [Picking a quest](../rind-quest-run/SKILL.md#picking-a-quest),
including its proof check. Install `quest` as
[Run it yourself](../rind-quest-run/SKILL.md#run-it-yourself) says.

Keep a quest whose files are free of in-flight work and free of every
other quest you keep. Record its path, a one-line plan, and why it is
clear. Record each quest you skip, and why.

An epic README becomes ready once its child quests have merged. Finish
it as
[Done means the Plan is met](../rind-quest-run/SKILL.md#done-means-the-plan-is-met)
says.

## Claim, then open the fork pull request

Do these in order for each quest you kept.

1. Run the overlap check again. When the quest is held, stop, and
   record the pull request or branch that holds it. Open nothing.
2. Claim with `quest start <path>`, or with the repo's `quest-start`
   stub. It pushes an empty commit to the quest branch on `<fork>` and
   opens a placeholder draft on moq-dev/rind. When the command refuses,
   the quest is blocked or already held: stop and record what it printed.
   A 403 from opening the upstream draft is the next step. Any other
   failure: stop and record the output.
3. When opening that upstream draft returns 403, open one draft pull
   request on `<fork>` from the quest branch
   ([Branch naming](../rind-git/SKILL.md#branch-naming)). The title and
   the body name the quest path. That draft is the claim.
4. When `quest start` opens the placeholder, leave its branch at the
   empty claim commit. A later push to that branch updates the upstream
   draft. Base the implementation on `up/main`, as
   [rind-git](../rind-git/SKILL.md) describes, and open it as one draft
   against `<fork>`'s `main`. Name the quest path and the claim pull
   request in the body.
5. Open a feature pull request on moq-dev/rind only in
   [Going upstream](#going-upstream).
6. Make the change as [rind-contribute](../rind-contribute/SKILL.md)
   describes, including `just check` before the pull request is ready.
   Stay inside the quest
   ([Stay inside the quest](../rind-quest-run/SKILL.md#stay-inside-the-quest)).
   Finish as
   [Done means the Plan is met](../rind-quest-run/SKILL.md#done-means-the-plan-is-met)
   says. Do not weaken constant-time code, and do not delete a test or
   a proof check.
7. When an open upstream pull request overlaps, name it for a trial
   merge, as
   [Before going upstream](../rind-quest-run/SKILL.md#before-going-upstream)
   describes.

Record the outcome as
[Hand it to a coding agent](../rind-quest-run/SKILL.md#hand-it-to-a-coding-agent)
describes.

Here, `quest start` is step 2. The placeholder it opens is the
claim
[Working from a fork](../rind-quest-run/SKILL.md#working-from-a-fork)
describes. The feature pull request still waits for
[Going upstream](#going-upstream).

## Review

Check a crypto change against
[rind-crypto-review](../rind-crypto-review/SKILL.md), and write the
findings on the pull request.

Run `/quest-review` as
[Before going upstream](../rind-quest-run/SKILL.md#before-going-upstream)
describes before marking the pull request ready.

Run the overlap check again before each push, and again before
[Going upstream](#going-upstream). When moq-dev/rind already has the
quest, on an open pull request, a `quest/*` branch, or a
`chore(quest): claim` commit, stop work on the fork pull request and
leave its branch in place.

Keep commits that are already on the branch. Update the branch as
[Updating a pushed branch](../rind-git/SKILL.md#updating-a-pushed-branch)
describes.

## Checks the pull request does not run

In `.github/workflows/build.yml`, these jobs run only when the event is
not a pull request: `build`, `musl`, `i686-windows`, `wasm`, `qemu`,
`ios`, and `android`. These workflows have no `pull_request` trigger:
`asan.yml`, `bench.yml`, `coverage.yml`, and `ctgrind-debug.yml`.
A path-filtered workflow runs the subset
[Git, commits and pull requests](../rind-contribute/SKILL.md#git-commits-and-pull-requests)
points at in `CONTRIBUTING.md`.

Run the missing checks as
[rind-bench-and-verify](../rind-bench-and-verify/SKILL.md) describes,
and run `just check` as
[Gate locally](../rind-contribute/SKILL.md#gate-locally) describes.
For a fix, run the check at the pull request head and at the parent
commit, and write both results in the pull request. The workflow file
holds the job's own steps.

## Going upstream

Follow
[Before going upstream](../rind-quest-run/SKILL.md#before-going-upstream)
and
[Working from a fork](../rind-contribute/SKILL.md#working-from-a-fork).
The fork checks are green, review findings on the head are answered,
and the overlap check is clear.

Do not merge the upstream pull request, and do not queue it.
`CONTRIBUTING.md` queues a reviewed head with `quest merge`.

## When no unclaimed quest is left

Do not start a quest the overlap check marks taken. Add the tests,
fuzzing, and constant-time checks in
[rind-bench-and-verify](../rind-bench-and-verify/SKILL.md) to the open
fork pull requests. A broken check on upstream `main` is m0 work, as
`quest/README.md` describes. Start that quest only when the overlap
check leaves it free.
