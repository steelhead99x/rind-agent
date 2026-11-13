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
  pull request, or closing one), fetch the fork and upstream remotes,
  and fetch again immediately before the commit and immediately before
  the push. Other writers may be on the same branches. If a matching
  branch or pull request already exists, stop and report it; never
  duplicate or overwrite it. If the branch moved while you worked,
  save your change as a patch, apply it on the new tip, and re-run
  the gates. If another writer has already pushed fixes you were
  about to add, leave their branch alone. Don't open a competing fix
  branch and don't force-push over their work. If your local work
  found a problem their fix missed, report it to the user, and with
  the user's approval comment on the pull request so that writer can
  take the fix. If another writer has made a commit or a reply on
  the pull request in the last half hour, wait until that work is
  quiet or they hand off before you push.
- Claim before you write. Before pushing to any fork pull request or
  fork branch, follow
  [Claim before you write](../rind-git/SKILL.md#claim-before-you-write):
  post a short claim comment on that pull request, wait out anyone
  else's unexpired claim, and post a done comment with the new head
  when you finish.
- Never push to a branch that backs a pull request a maintainer has
  taken over, even under the user's own account, unless the user asks
  for that exact change. See [Maintainer takeover](#maintainer-takeover).
- Never print tokens or credentialed remote URLs. Refer to remotes by
  name only.
- Follow [rind-git](../rind-git/SKILL.md) for history that is already
  pushed: no rebase, amend, or force-push. Rebuild an upstream-bound
  change on a fresh branch name, as
  [Upstream-bound branches](../rind-git/SKILL.md#upstream-bound-branches)
  describes, and push it normally. Force-updating a base branch
  closes pull requests that target it, and a pull request whose
  branch was force-updated cannot be reopened, so plan replacements.
  If the user has explicitly approved a force-update, back up first,
  use `--force-with-lease`, and restore any branch protection you
  changed.
- Don't open, close, comment on, or mark ready any pull request,
  upstream or on the fork, until the user has approved the list of
  changes and the text of any comment. After that, do only what they
  approved. Approval to work on a fork pull request covers its claim
  and done comments.
- A known correctness or security bug in a change must be fixed in the
  pull request before it is marked ready. Don't defer that kind of fix
  to a follow-up. Refactors and extra tests may go in a follow-up after
  the pull request lands.
- Check free disk space before builds. Parallel builds of a large
  project can fill a disk. Clear build caches only in superseded
  worktrees.

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

A commit that appears under the publishing identity may have come
from another of the user's machines or agents. Fetch and check its
authorship and message before replacing or rebuilding over it.

## Sync

Fetch both remotes. Fast-forward the local default branch only with
`--ff-only`, and only if the working tree is clean. Report how far the
fork is ahead of or behind upstream.

The fork's default branch may have its own commits and may not
fast-forward to upstream. Don't force-update it. Merging upstream into
the fork can restore files the fork is missing, but it can also undo
intentional fork-only changes; get the user's approval first.
Upstream-bound rebuilds still start from current upstream `main`, so a
diverging fork default branch does not block them.

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

If another writer claims to have fixed review findings but has not
run the local gates the change needs, including constant-time and
variable-time checks, run those gates on the current head before
treating the findings as closed, even when the writer says they are
already fixed. Library helpers such as sort can hide division that
those checks reject.

Parallelize by splitting worktrees between workers. Each worker owns
its own fresh worktree and branches.

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
it normally, open a replacement fork review pull request, and close
the old fork review pull request with a one-line pointer to the new
one. Open the replacement upstream pull request only when it is done,
as [Keep upstream clean](#keep-upstream-clean-open-only-when-done)
describes, then close the old upstream pull request with a one-line
pointer. Re-run every gate, including
other-OS gates, on the new head, and hand the new SHA to whoever runs
those gates. Repoint any watch routine at the new pull request.

## Keep upstream clean: open only when done

Finish everything on the fork first. Open the upstream pull request
only once the fork review of the exact commit going up has passed with
nothing open, every local and other-OS gate passes on that commit, fork
CI is green, and it merges cleanly with current upstream `main`. Don't
open upstream drafts early to get CI running or to signal work in
progress; the maintainers should only see finished work.

When it is done, open it against upstream `main` with a clean title and
body: no identifying info, no out-of-date claims, the recorded review,
the gate results for that SHA, and a link to the pull request it
supersedes. Opening as a draft and marking it ready as soon as upstream
CI goes green is fine. Close the superseded upstream pull request, and
its fork review pull request, with a one-line pointer.

If an upstream pull request is already open but the change isn't done
(new findings, a rebuild, gates pending), the user may prefer to close
it until it is finished. A closed pull request can be reopened later
from the same branch as long as that branch was never force-updated, so
close it with a short neutral note rather than replacing it.

## Maintainer takeover

If a maintainer starts working on an open draft (adds a label, rewrites
the description, pushes commits, or takes it out of draft), do not
close or replace that pull request and do not overwrite their branch.
Leave any local rebuild unused. Don't comment unless the user
approves the text.

Don't push to that pull request's branch, even under the user's own
account, unless the user asks for that exact change. Claim comments
don't go on upstream or maintainer-owned pull requests. If unrelated
commits land on the branch, revert them with new commits (never a
force-push), leave one short neutral note, and tell every agent that
works on the repo to stop pushing to it.

A watch routine should notify the user when a maintainer comments,
reviews, or pushes, and must not mark the pull request ready while a
maintainer request is still open.

After that pull request merges, rebuild any stacked follow-up from
the merged result and drop commits the merge already covered,
including changes the maintainer made on the branch.

## Fork review

If the user's review process runs only on fork pull requests, open a
fork pull request for each new branch based on the fork's default
branch (so the diff is just the one commit), and name the matching
upstream pull request in the body, for example
`Upstream: <owner>/<repo>#N`. Wait for that review's verdict and
address every actionable finding with new commits, not amends. Copy
the recorded review into the upstream pull request body the way
[rind-git](../rind-git/SKILL.md#pull-requests) and the repo's
contributing guide ask.

Check what the review process posts. If its template leaks a personal
name or other identifying detail, tell the user.

Keep a scheduled check, about every half hour and offset from the
review process, that looks at each open fork pull request: what is
the newest review, and does it cover the current head? Skip pull
requests another writer owns, ones that already have a fix in
progress, and ones a maintainer has taken over.

Fix every finding yourself (correctness, high, medium, low, and
nits) rather than only reporting it, and reason through each fix
carefully. Put the fixes in new commits, each on a fresh worktree, and
claim the pull request first. Fetch right
before the commit and again before the push, run the full local
gates, and push a fast-forward only. Do not force-push. With the
user's approval of the text, post one short reply on the fork pull
request naming what was fixed and which commit has the fix; a reply
or a new commit is what triggers a re-review.

Check the review's claims before acting on them (for example, re-read
a cited CI job). If the review, or earlier text on the pull request,
cited the wrong evidence, correct that record in the reply instead
of "fixing" something that was already right.

Findings that need a design decision, or that conflict with a
maintainer's request, go to the user with a recommendation, not into
a commit. A no-op commit already in reviewed history stays; note that
it drops out when the work is squashed for upstream.

When the newest review covers an older head, re-check each finding
against the current head and fix the ones that still apply. Also fix
failures that other-OS gate results report in pull request comments.

When a fix lands on a fork pull request that backs an upstream pull
request, the upstream head branch needs the same change, and other-OS
gates must run again on the new head.

## Mark ready

Because upstream pull requests open only when the work is done
([Keep upstream clean](#keep-upstream-clean-open-only-when-done)),
this is normally a quick check right after opening. Mark a pull
request ready only when all of these hold: no known
unfixed correctness or security bug remains, upstream CI is green
(fork pull requests usually sit at `action_required` until a
maintainer approves the runs), every local gate and any other-OS
gate has passed on the current head, every maintainer standard in
the repo's contributing and review docs is met, the current head has
a recorded passing review, and the pull request merges cleanly. If
other-OS or extra-gate results are posted on the fork or upstream
pull requests as comments, commits, or description updates, wait for
those too. A new head needs the other-OS gates rerun; hand the new
SHA to whoever runs them. If a maintainer request is still open,
leave the pull request as a draft.

When a new base-branch requirement appears, for example an automated
review the repository now requires, report it. Don't treat "still a
draft, and CI has not started" as the only remaining blocker.

Use a scheduled check for these conditions instead of waiting
in-session, and have it stay silent until something changes. If the
head moves, wait for the review process to re-verify the new commit.
If a pull request is replaced, repoint that check at the new one. If
anything fails or falls short, report it and leave the pull request
as a draft.

## Local cleanup

Do this while waiting on reviews or other-OS gates, and only in the
local checkout. It must not change any open pull request branch.

Before removing anything, save uncommitted work as a patch and
untracked files as an archive.

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

Scan logs and build caches for tokens before deleting them. After
that scan, build caches in superseded worktrees may be deleted.
Then run `git -C <repo> gc --prune=now`.

## Publishing skill updates

When these skills themselves change, land the update through a pull
request as the publishing identity, with the usual identity scrub:
no personal names, local paths, host names, machine or session IDs,
or tokens. Add a new commit to an open pull request for the same
skill when its branch is unchanged; otherwise open a fresh branch
from `main`. Don't merge it; the owner squash-merges.

## Report

For each pull request: link, head SHA, gates passed or failed, review
verdict, CI state, and what's still blocking. Also report what was
cleaned up, the backup branches, and any secrets or identity problems
that remain in pushed history.
