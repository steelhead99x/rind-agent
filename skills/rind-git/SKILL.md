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
`moq-dev/rind`. When working from a fork, `origin` is the fork.

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

## Updating a pushed branch

Merge `up/main` into the branch. Never rebase, amend, or force-push a
branch that has already been pushed.

Never merge the fork's `main` into an upstream-bound branch: the fork's
`main` can carry files upstream does not.

When merging `up/main` into a feature branch, resolve a conflict in a
shared list (the README's algorithm table, for example) by keeping both
entries, then rerun that feature's tests before pushing.

If other work may push to the same branch, fetch and merge
`origin/<branch>` immediately before every push.

Do not push to a branch that is the head of an open upstream pull request
unless that push is needed (a conflict fix, for example).

Before every push, check that the pull request is still open: a push after
merge recreates the deleted branch. Run `just ci` before every push, as
`rind-contribute` says.

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
- Attribution goes in a `Co-Authored-By:` trailer only, never in the
  subject, body, or pull request text.
- Exactly one `Co-Authored-By: Cursor Agent <cursoragent@cursor.com>`
  trailer, and no other `Co-Authored-By` trailers. Check with
  `git log -1 --format=%B`. Some environments append their own trailer
  on `git commit`. Before the first push, rewrite the message with
  `git -c core.hooksPath=/dev/null commit --amend` or build it with
  `git commit-tree`. After the branch has been pushed, do not amend;
  add a new commit instead.
- No binaries or build artifacts.

## Pull requests

Pull requests squash onto `main`, so the title becomes the commit subject.

Body, short and structured: **Problem**, **Approach**, **Impact** (one
bullet per public API change, with migration for breaking ones),
**Alternatives**, **Follow-ups**. Write "None." in a section that does
not apply. A finished quest's PR also names the quest it claims and
deletes that quest file.

Open a draft while working; mark it ready when the gate passes and the
body lists no decision the maintainer still owns.

## Squash-merge

```shell
gh pr merge --squash --author-email <github-noreply> \
  --subject '<title> (#N)' \
  --body 'Co-Authored-By: Cursor Agent <cursoragent@cursor.com>'
```

`--author-email` must be the account's GitHub noreply address. When email
privacy is on, use the `id+username@users.noreply.github.com` form; the
bare `username@users.noreply.github.com` form may be rejected.

Then verify the resulting `main` commit's author, committer, and that it
has exactly one `Co-Authored-By:` trailer.
