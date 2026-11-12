---
name: fork-to-upstream
description: Use when reviewing, cleaning up, and sending a fork's ready branches upstream as pull requests.
---

# Fork to upstream PR workflow

Use this when ready work on a local fork checkout should go upstream as
pull requests. Day-to-day branching, commits, pushes, and squash-merges
follow [rind-git](../rind-git/SKILL.md). Setup and the local gate are in
[rind-contribute](../rind-contribute/SKILL.md). Crypto review is in
[rind-crypto-review](../rind-crypto-review/SKILL.md). Extra gates
(benchmarks, other architectures, constant-time tooling, proofs) are in
[rind-bench-and-verify](../rind-bench-and-verify/SKILL.md).

Do the work in the existing local checkout. Keep that checkout's path
and pass it as `git -C <repo>` on every git command, including when
resolving the root with `git -C <repo> rev-parse --show-toplevel`.
Don't rely on the current working directory: a routine starts in a
different directory. Don't make a new clone unless the user asks for
one.

## Ground rules

- Always run git with an explicit repo path (`git -C <repo>`),
  especially from a routine, which starts in a different directory.
- Before every step that writes (commit, push, opening or editing a
  pull request, or closing one), fetch the fork and upstream remotes.
  Other writers may be on the same branches. If a matching branch or
  pull request already exists, stop and report it; never duplicate or
  overwrite it.
- Never print tokens or credentialed remote URLs. Refer to remotes by
  name only.
- Follow [rind-git](../rind-git/SKILL.md) for history that is already
  pushed: no rebase, amend, or force-push. Rebuild an upstream-bound
  change on a fresh branch name, as
  [Upstream-bound branches](../rind-git/SKILL.md#upstream-bound-branches)
  describes, and push it normally.
- Don't open, close, comment on, or mark ready any upstream pull
  request until the user has approved the list of changes. After that,
  do only what they approved.
- A known correctness or security bug in a change must be fixed in the
  pull request before it is marked ready. Don't defer that kind of fix
  to a follow-up. Refactors and extra tests may go in a follow-up after
  the pull request lands.

## Remotes and identity

Resolve remotes at run time, not from a remembered owner or path:

```shell
git -C <repo> remote -v
git -C <repo> remote get-url origin
```

`origin` is the user's fork. The upstream remote is usually `up`
(sometimes `upstream`). Read owner and repo names from those URLs.

Strip tokens out of remote URLs. After taking the URL from
`git -C <repo> remote get-url`, set a clean `https://host/owner/repo.git`
form with `git -C <repo> remote set-url <name> <url>`. Run
`gh auth setup-git` first so pushes still authenticate, then confirm
with `git -C <repo> ls-remote`.

Scan configs, logs, shell history, `.env` files, and uncommitted files
for tokens and keys, and redact them. Report committed secrets only;
never rewrite pushed history to hide them. Tell the user to revoke any
token that was exposed.

Derive the publishing identity from the logged-in GitHub CLI account:

```shell
login=$(gh api user --jq .login)
id=$(gh api user --jq .id)
email="${id}+${login}@users.noreply.github.com"
git -C <repo> config --local user.name "$login"
git -C <repo> config --local user.email "$email"
```

Confirm that account is the one that should appear on the commits and
the pull request. Set author and committer to that name and noreply
address. For the required `Co-Authored-By` trailer and for
squash-merge `--author-email` rules, follow
[Commits](../rind-git/SKILL.md#commits) and
[Squash-merge](../rind-git/SKILL.md#squash-merge).

On outgoing branches, list commit authors (`git -C <repo> log
<upstream>/main..HEAD --format='%an <%ae> | %cn <%ce>'`) and search
the diffs for real names, home paths, hostnames, work domains, and
personal emails.

## Sync

Fetch both remotes. Fast-forward the local default branch only with
`--ff-only`, and only if the working tree is clean. Report how far the
fork is ahead of or behind upstream.

## Find what goes out

Open, non-draft fork pull requests are the candidates. Skip any whose
description says it isn't done or is blocked, and any already merged,
closed, or superseded upstream. Tell the user what evidence you used
for each.

Also list every local branch or worktree with commits that aren't on
any remote:

```shell
git -C <repo> log --branches --not --remotes --oneline
```

Unpushed fix branches often carry fixes the outgoing pull request is
missing. For each candidate, check other branches that touch the same
files for commits the candidate doesn't have. If any are missing, show
them to the user and agree what to include before squashing or
rebuilding.

## Review each candidate

Diff it against current upstream `main`. Check that it merges cleanly
with `git -C <repo> merge-tree`, without changing anything. Watch for
reverted merges that would delete upstream work.

Run the gates in [rind-contribute](../rind-contribute/SKILL.md) and,
where the change needs them,
[rind-bench-and-verify](../rind-bench-and-verify/SKILL.md). Note any
gates that need another OS or architecture. For crypto or security
code, also follow [rind-crypto-review](../rind-crypto-review/SKILL.md).

If review finds a known correctness or security bug, treat it as
blocking: it has to land in this pull request, with a regression test,
not in a later follow-up.

Parallelize by splitting worktrees between workers. Each worker owns
its own branches.

## Rebuild cleanly (only if the user approves)

When the history carries unwanted authors or unrelated merges, cut a
fresh branch from current upstream `main` that carries the change's
files as one commit, the way
[Upstream-bound branches](../rind-git/SKILL.md#upstream-bound-branches)
describes. Author and committer are the account resolved above. The
message follows [rind-git](../rind-git/SKILL.md#commits), including its
required co-author trailer. Re-run the gates on the new branch, then
push it as a new branch with a normal push.

If a pull request is already reviewed and still has a known
correctness or security bug, don't amend or force-push the reviewed
branch. Rebuild the change as one commit on a fresh branch from
current upstream `main`, include the fix and a regression test, and
name the new branch by incrementing any `-rN` suffix to `-rN+1`. Push
it normally, open a new draft that replaces the old pull request, and
close the old upstream pull request and its fork review pull request
with a one-line pointer to the new one. Re-run every gate, including
other-OS gates, on the new head, and hand the new SHA to whoever runs
those gates. Repoint any watch routine at the new pull request.

## Open upstream as drafts

Open each as a draft pull request against upstream `main` with a clean
title and body: no identifying info, no out-of-date claims, and a link
to the pull request it supersedes. Close the superseded upstream pull
request with a one-line pointer.

## Fork review

If the user's review process runs only on fork pull requests, open a
fork pull request for each new branch based on the fork's default
branch (so the diff is just the one commit), and name the matching
upstream pull request in the body. Wait for that review's verdict and
address every actionable finding with new commits, not amends. Copy
the recorded review into the upstream pull request body the way
[rind-git](../rind-git/SKILL.md#pull-requests) and the repo's
contributing guide ask.

Check what the review process posts. If its template leaks a personal
name or other identifying detail, tell the user.

## Mark ready

Mark a pull request ready only when all of these hold: no known
unfixed correctness or security bug remains, upstream CI is green
(fork pull requests usually sit at `action_required` until a
maintainer approves the runs), every local gate and any other-OS
gate has passed on the current head, every maintainer standard in
the repo's contributing and review docs is met, the current head has
a recorded passing review, and the pull request merges cleanly. If
other-OS or extra-gate results are posted on the fork or upstream
pull requests as comments, commits, or description updates, wait for
those too. A new head needs the other-OS gates rerun; hand the new
SHA to whoever runs them.

Use a scheduled check for these conditions instead of waiting
in-session, and have it stay silent until something changes. If the
head moves, wait for the review process to re-verify the new commit.
If a pull request is replaced, repoint that check at the new one. If
anything fails or falls short, report it and leave the pull request
as a draft.

## Local cleanup

Do this while waiting on reviews or other-OS gates, and only in the
local checkout. It must not change any open pull request branch.

For each local branch that still has unpushed work, create a local
tag named `backup/<branch>`. After the identity checks in
[Remotes and identity](#remotes-and-identity) pass, push those
commits to the fork as new `backup/<branch>` branches: a normal push
of a new name, never a force-push, and never a push that overwrites
an existing branch.

Remove a worktree only when its working tree is clean and
`git -C <repo> branch -r --contains <tip>` shows its tip is already
on a remote. `git -C <repo> worktree remove --force` is allowed only
to clear the checked-out-submodule refusal; don't use it to discard
uncommitted or unpushed work. After removals, run
`git -C <repo> worktree prune`. Leave worktrees that belong to open
pull requests alone.

Delete a local branch only after
`git -C <repo> branch -r --contains <tip>` shows the tip is on a
remote. Never delete a remote branch.

Scan logs and build caches for tokens before deleting them. Then run
`git -C <repo> gc --prune=now`.

## Report

For each pull request: link, head SHA, gates passed or failed, review
verdict, CI state, and what's still blocking. Also report what was
cleaned up and any secrets or identity problems that remain in pushed
history.
