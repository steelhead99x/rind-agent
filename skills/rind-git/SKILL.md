---
name: rind-git
description: Use when branching, committing, pushing, or squash-merging a change to rind or a fork.
---

# Git workflow for rind

Follow these rules for every rind change. Setup, the change itself, and the
local gate are in [rind-contribute](../rind-contribute/SKILL.md). Picking
and finishing a quest is in [rind-quest-run](../rind-quest-run/SKILL.md).

## Remotes and base

Keep an upstream remote, usually `up` (sometimes `upstream`), pointing at
`kixel-ai/rind`. `https://github.com/moq-dev/rind` redirects there; use
`kixel-ai/rind` in remotes, `gh` commands, and new URLs. When working
from a fork, `origin` is the fork.

Base every new branch on current upstream `main` (`up/main`), not the
fork's `main`. Before starting, and again before handoff, run the overlap
check in [Picking a quest](../rind-quest-run/SKILL.md#picking-a-quest).
An upstream `quest/<milestone>/<area>/<name>` branch, or a first commit
of the form `chore(quest): claim quest/...md`, counts as taken. If it
overlaps, stop.

## Branch naming

One quest or one problem per branch. For a quest, name the branch from the
quest path: `quest/<milestone>/<area>/<name>` (the path under `quest/`
without the `.md`). Do not reuse a name that already exists on upstream.

## Files outside the change

A feature PR must not edit sibling quest files or shared docs that an
open upstream PR already touches. Before editing any file outside the
quest's own scope, check open and draft upstream PRs for that path
(`gh pr list --repo kixel-ai/rind --state open` includes drafts; read
each PR's files). The development fork's PR #1 (SPAKE2+) rewrote
`quest/m4/protocols/hash-to-curve.md`, which collided with upstream #632.

## Updating a pushed branch

Merge `up/main` into the branch. Never rebase, amend, or force-push a
branch that has already been pushed. Do not use `--force` or
`--force-with-lease` unless the fork owner has explicitly approved
rewriting the named branch. A revert removes the unwanted tree from the
tip but leaves the commits in the pull request's history; cleaning that
history is a force-push and needs the same approval.

Never merge the fork's `main` into an upstream-bound branch: the fork's
`main` can carry fork-only files (an extra algorithm, for example) that
upstream does not, and those files would appear on an upstream pull
request.

A fork branch that is already the head of an open upstream pull request
is in that category even when the change first landed on the fork.
Never merge the fork's `main` into it, and never include it in a bulk
update of fork branches. Before any such sweep, list those heads and
exclude them:

```shell
gh pr list --repo kixel-ai/rind --author <fork-owner> --state open --json headRefName
```

`<fork-owner>` is the account that opened the fork's upstream pull
requests (`gh api user --jq .login` when that account is logged in).
Branches meant to go upstream take updates from upstream `main` only.

List those heads from the fork owner's open upstream pull requests
each time, immediately before a sweep or a push. The names change:
when a change is rebuilt onto a new `-rN` branch and a new pull
request replaces the old one, protect only the current head. Leave
older `-rN` branches alone: do not merge the fork's `main` into
them, include them in a sweep, or push to them.

Before every push, confirm the target branch is not one of those
current heads.

When merging `up/main` into a feature branch, resolve a conflict in a
shared list (the README's algorithm table, for example) by keeping both
entries, then rerun that feature's tests before pushing.

When more than one agent or person may be working on the fork or on
the same branch, fetch the fork (`origin`) and upstream (`up`) before
every step, and again immediately before every push. Merge anything that
moved: `origin/<branch>` into the branch you are about to push, and
`up/main` into the fork's `main` only when you were told to sync. Never
overwrite another writer's commits. Wait for the head to settle before a
review round.

Do not push to a branch that is the head of an open upstream pull request
unless the fork owner has approved that push. Any push to that branch
appears on the upstream PR and reruns its CI, so a routine sync should
skip it.

Before every push, check that the pull request is still open: a push after
merge recreates the deleted branch. Run `just ci` before every push, as
`rind-contribute` says.

## Claim before you write

Before pushing to any fork pull request or fork branch, post a short
claim comment on that pull request, for example
`In progress: <what> (claim expires in 60 min)`. When you finish, post
`Done: <what>. New head: <sha>`, or `Released` if you stop without
finishing. Before claiming, read the pull request's recent comments:
if another writer has an unexpired claim, don't push. Wait, or comment
and hand off. A claim past its expiry with no follow-up may be taken
over with a note saying so. Claim comments go on fork pull requests
only, never on upstream or maintainer-owned pull requests.

Never push to a branch that backs a pull request a maintainer has
taken over, even under the fork owner's account, unless the owner asks
for that exact change. If unrelated commits land there, revert them
with new commits (never a force-push), leave one short neutral note,
and tell every agent that works on the repo to stop pushing to it.

## Upstream-bound branches

An upstream-bound branch is one squashed commit on a fresh branch from
`up/main`. Rebuild it that way: squash or cherry-pick only the change's
own commits onto current `up/main`, drop merge commits from the fork and
any fork-only files (a review config, for example), and check that
`git diff up/main --stat` shows only the change's files. Push the rebuilt
branch under a new name; never force-push a branch that has already been
reviewed. Once a branch is already pushed upstream, merge updates into
it; never rebase or force-push it.

## Commits

- Conventional-commit subjects: `feat(rustls): ...`, `fix: ...`, `perf: ...`,
  `docs: ...`, `chore: ...`. Breaking change: `feat!: ...`.
- The author and the committer are the fork owner's GitHub login and that
  account's noreply address. Use the
  `id+username@users.noreply.github.com` form when email privacy is on, not
  the bare `username@users.noreply.github.com` form, a nickname, an
  organization name, or a tool or model name. Set this in the checkout
  before the first commit:

  ```shell
  git config user.name '<github-login>'
  git config user.email '<id>+<login>@users.noreply.github.com'
  ```

  If `GIT_AUTHOR_NAME`, `GIT_AUTHOR_EMAIL`, `GIT_COMMITTER_NAME`, or
  `GIT_COMMITTER_EMAIL` name a tool or model, unset or override them.
  `git -c user.name=...` is not enough when those variables are set.
- Attribution goes in a `Co-Authored-By:` trailer only, never in the
  subject, body, or pull request text. Exactly one
  `Co-Authored-By: Cursor Agent <cursoragent@cursor.com>` trailer, and no
  other `Co-Authored-By` trailers: no model names, and do not list the
  fork owner as a co-author. Check with `git log -1 --format=%B`. Some
  environments append their own trailer on `git commit`. Before the first
  push, rewrite the message with
  `git -c core.hooksPath=/dev/null commit --amend` or build it with
  `git commit-tree`. After the branch has been pushed, do not amend;
  add a new commit instead.
- Hosted cloud agents author and commit as
  `Cursor Agent <cursoragent@cursor.com>` and may add the person who
  started the run as a `Co-Authored-By` trailer. That platform identity
  cannot be changed in git config. Do not rewrite those commits. Correct
  the author and trailer list when squash-merging, as below.
- No binaries or build artifacts.
- On a fork branch still in review, a fix for a review finding is its own
  small commit, and the subject names the fix. Leave the earlier commits
  as they are. The upstream-bound rebuild still squashes those into one
  commit.

## Pull requests

Pull requests squash onto `main`, so the title becomes the commit subject.

Body, short and structured: **Problem**, **Approach**, **Impact** (one
bullet per public API change, with migration for breaking ones),
**Alternatives**, **Follow-ups**. Write "None." in a section that does
not apply. A finished quest's PR also names the quest it claims and
deletes that quest file.

Before opening the pull request, check `UPSTREAM.md`. It lists every
workaround for a bug or gap in a tool or dependency rind doesn't own.
A pull request that adds one adds its entry and names that entry, by
its heading, in the workaround's comment (`See UPSTREAM.md, "<heading>".`).
A pull request that drops one removes it from its entry, and deletes
the entry once it lists no workaround. The rule is rind's
`CONTRIBUTING.md`, under Upstream.

Open a draft while working; mark it ready when the gate passes and the
body lists no decision the maintainer still owns. List each open
decision in the description, and leave the decision to the maintainer.

The body says which environment each result came from, and which base a
review diff used (`up/main...HEAD` when the pull request targets a fork).
Carrying commits out of a closed pull request is in
[Salvaging a closed pull request](../rind-quest-run/SKILL.md#salvaging-a-closed-pull-request).

## Squash-merge

```shell
gh pr merge --squash --author-email <github-noreply> \
  --subject '<title> (#N)' \
  --body 'Co-Authored-By: Cursor Agent <cursoragent@cursor.com>'
```

`--author-email` must be the account's GitHub noreply address. When email
privacy is on, use the `id+username@users.noreply.github.com` form; the
bare `username@users.noreply.github.com` form may be rejected. Merge as
the fork owner's GitHub account so the squash commit's author name is
that login, not a tool, bot, or organization.

If a commit was authored under a tool's own name, or has extra
`Co-Authored-By` trailers, fix the author and trailer list in the
squash-merge subject and body. Do not rewrite pushed history to correct
them.

Then verify the resulting `main` commit's author, committer, and that it
has exactly one `Co-Authored-By:` trailer.
