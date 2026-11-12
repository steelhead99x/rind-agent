---
name: rind-quest-run
description: Use when working rind's planned `quest/` backlog with `quest skill spawn` or `quest skill run`.
---

# Working the rind backlog with quest

rind's planned work lives in `quest/`. The repo's `.claude/skills/quest-*`
stubs call the `quest` CLI, which carries the real instructions.

## Run it yourself

1. Install and pin `quest` per <https://github.com/moq-dev/quest/blob/main/SETUP.md>.
   Confirm `quest --version` and `quest check`. Without Nix, build the commit
   the flake pins: `bun install --frozen-lockfile && bun run build` in that
   checkout, then `bun <checkout>/dist/quest.js` from the target repo.
   Running `src/cli/bin.ts` from another directory fails. Skip a GitHub
   release the setup guide says to skip.
2. `quest context` and follow it.
3. `quest skill spawn` and follow its output to start quests in parallel.
   `quest skill run` (the repo's `quest-run` stub) works one session: it
   starts ready quests and takes open PRs toward merge until only the
   maintainer's decisions remain.
4. Each quest: its own branch named from the quest path, the
   `rind-contribute` and `rind-git` steps, the `rind-crypto-review`
   checklist, then a PR.

## Working from a fork

`quest start` (and the `quest-spawn` / `quest-start` stubs) take a lock by
opening a placeholder claim PR against the repo that holds
`quest/README.md`, which for a fork is moq-dev/rind. That is the intended
way to prevent duplicate work. A fork's `quest start` posts that PR
upstream, so only run it once the work is meant to go upstream. Until
then, branch by hand off upstream `main`, treat the quest file as the
spec, and rely on the branch and PR overlap checks. Those checks still
miss work that starts in the same hour, so run them again before
handoff.

## Sync the fork first

Before picking quests or handing them to agents, merge upstream `main` into
the fork's `main` with a merge commit (`gh repo sync <fork> --branch main`,
GitHub's Sync fork, or a PR from `moq-dev:main` into the fork's `main`
merged with a merge commit). Write the upstream tip SHA in that merge
subject as literal characters. A command substitution inside single quotes
is stored unexpanded. See [Known issues](#known-issues). Never force-push
or reset: the fork's own commits (a review config, for example) must stay.
Otherwise a lagging fork lists quests upstream already finished, and a fork
PR's diff includes those unrelated upstream commits. Quest branches still
base on upstream `main`, and fork-only commits still must not go upstream.

## Picking a quest

Run `quest unblocked`, then skip anything it marks held, anything with an
open upstream PR or branch (`gh pr list --repo moq-dev/rind`), and anything
waiting on an upstream release that hasn't shipped. Also skip work needing
hardware or accounts you don't have, and proofs you can't check with
`admin/prove`. Prefer quests the maintainer is less likely to be running
in parallel (adapter and adoption work, and test-vector or coverage gaps)
over core primitives, proofs, and CI. Prefer missing functionality and
correctness over micro-optimizations. When unsure, ask which quests are
open for help.

Before claiming a quest, whether you're a person or an agent, check every
place work can be in flight. Syncing the fork does not show it. Open and
draft PRs and branches on upstream and the fork both count, including
`cursor/*` and other branches not named for the quest. Match them by
the files and the quest path they touch, not by the branch name alone.
Also search open and merged upstream PRs by the conventional head
branch `quest/<milestone>/<area>/<name>` and by title. List upstream
quest branches with `git ls-remote up 'refs/heads/quest/*'` (`up` is
the upstream remote; it may be named `upstream`). Maintainers claim a
quest by pushing an upstream `quest/<milestone>/<area>/<name>` branch
whose first commit is `chore(quest): claim quest/...md`, before any
PR exists. An upstream branch with that name, or a commit of that
form, counts as taken, so a search of PRs alone misses them. Run the
same overlap check again immediately
before pushing, right before a branch goes to review, and right
before proposing anything upstream. If it overlaps, stop. A claim
can land while the work is in progress. Also check running cloud or
coding agents on the backlog, and files an in-flight quest already
touches (shared CI configs such as `mutants.yml` and `fuzz.yml`,
`admin/prove`, and shared directories under `proofs/`). Prefer
`quest unblocked --unclaimed` when a GitHub token is available. It
skips a quest's own claim and does not replace the checks above.
Leave a claim marker early: a draft PR on the fork that names the
quest path, so others can see it.

Before picking or handing off a formal-proof quest (`quest/m4/verified/`),
after the in-flight check above, run this feasibility check and skip the
quest if it fails. Compare the target's size to the closest landed proof
in `proofs/` (see `proofs/README.md`), by instruction count and by lines
of HOL Light or Lean. A target several times that proof's instruction
count fails, as does a proof several times its length. Check that
the spec lemmas it needs already exist and are not opaque.
`proofs/README.md` names the opaque ones, such as `edPow` and `edMul` in
`proofs/Rind/Ed25519.lean`. Check that any spec it needs is merged
upstream. An open s2n-bignum pull request counts as unmerged. Check
that it needs no new axiom of the kind the composition rule in
`quest/m3/verified/README.md` adds, one per imported routine, and
that its check mark in `benchmarks/verified.toml`
does not also depend on another unproven quest. Confirm
`nix develop .#prove --command admin/prove` runs on this machine. That
shell is Nix, Lean, Charon, Aeneas, and HOL Light. Cloud VMs usually
lack it. A failed check is a development past quest size. Prefer
correctness, testing, or missing functionality.

Pick against current upstream `main`, not the fork's. A fork that lags
upstream will still list quests the maintainer merged that day. Confirm
the quest file still exists on upstream `main`. Base a new quest branch
on upstream `main`.

Before each push to a quest branch, before each review round, and before
proposing it upstream, re-read the quest file on upstream `main`, not the
copy on the branch. Diff its `## Required` and `## Plan` sections against
the copy the branch started from. The overlap check above does not read
those sections. A new Required item that is not merged blocks the quest.
Stop adding work that the new item says should come from elsewhere, say
so in the PR as a question for the maintainer, and leave the PR a draft
until the item lands. `quest unblocked` on upstream `main` omits a quest
whose file has a `## Required` section, which is a quick signal.

[moq-dev/rind#669](https://github.com/moq-dev/rind/pull/669) (`ad43e51a`,
2026-11-12, "chore(quest): plan test coverage of the published code") added
to `quest/m2/confidence/mutants-adapters.md` the Required item
"rind-aws-lc-rs passes aws-lc-rs's own test suite"
(`quest/m2/confidence/aws-lc-rs-upstream-tests.md`): upstream's tests kill
rind-aws-lc-rs's mutants first, so none gets a hand-written test. A fork
branch for that quest was already open. It merged upstream `main` only
through `aee2586a`, the parent of `ad43e51a`, so the copy on the branch
did not gain the item, and it still added hand-written tests for
rind-aws-lc-rs mutants in `rind-aws-lc-rs/tests/surfaces.rs`. That quest's
other Required item, `quest/m0/adapters/README.md`, was still unmerged as
[moq-dev/rind#652](https://github.com/moq-dev/rind/pull/652).

Search upstream issues for the quest path and the title, as well as
pull requests. An issue that names the quest is taken even when no
pull request or quest branch exists. Repeat that search before opening
an upstream pull request.

Before starting a follow-up a PR body still lists (a SIMD path, a missing
backend), read the current branch and the later commits first. The work may
already have landed on that branch.

## Stay inside the quest

A feature PR must not edit sibling quest files or shared docs that an
open upstream PR already touches. Before editing any file outside the
quest's own scope, check open and draft upstream PRs for that path
(`gh pr list --repo moq-dev/rind --state open` includes drafts; read
each PR's files). The development fork's PR #1 (SPAKE2+) rewrote
`quest/m4/protocols/hash-to-curve.md`, which collided with upstream #632.
The same rule is in
[rind-git](../rind-git/SKILL.md#files-outside-the-change).

## Done means the Plan is met

Read the quest's Plan section line by line and mark each item met or not met.
Every vector set and differential test it names is required. When one is
impossible, record the exact reason in the PR's Follow-ups as a question for
the maintainer, for example a reference implementation that is `pub(crate)`
in its latest release while `deny.toml` denies git dependencies. Never push a
partial quest.

A docs-only quest that closes an epic still needs precise, testable rules
(exact target triples, not prose about architecture and OS pairs) and must
keep the epic's stated goal.

Check platform and target claims against `.github/workflows/build.yml`,
`Cross.toml`, and `sh/dispatch.sh`. A target that only compiles
(`aarch64-apple-ios` is `cargo test --no-run`; `wasm32-unknown-unknown`
is compile-only) is not a tested one.

When a PR defers related work, do not add a quest file. A follow-up a
quest file mentions is left for the maintainer through `/quest-plan`
(rind `CONTRIBUTING.md`, "Follow-ups") and raised as a question in the
PR. An `rs-matter` differential the PR cannot include is that kind of
follow-up. An item the Plan requires and that cannot be done stays a
question for the maintainer.

A finished quest's PR also deletes its quest file and updates the docs that
list what rind offers: `ALGORITHMS.md`, `TESTING.md`, and the README's table.
When aws-lc-rs lacks the algorithm, use the reference crate the Plan names
for the differential test, and say in Alternatives that there is no aws-lc-rs
benchmark. A CI fix that closes a quest deletes that file too
(`quest/m0/android-armv7-abi.md`). The PR names the change that broke a
job which used to pass, and why.

## Hand it to a coding agent

Sync the fork with upstream first. Give the agent the outcome, not a script:

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
then FROST and the OPRF) wait for their base to merge. Doing that from a
fork is in [rind-quest-fleet](../rind-quest-fleet/SKILL.md).

Change direction mid-run with a short steer, not a restart.

After an agent finishes, send the `rind-crypto-review` findings as one
follow-up. If upstream now covers that quest, stop the agent. A scheduled
run can reply only to agents launched in that same conversation. Write
other follow-ups into the conversation that launched them. Jobs that skip
pull requests need a separate clone, one worktree per PR, an isolated
`CARGO_HOME`, and `target/` removed afterward. Run them at the PR head and
at the parent commit. `nix develop --profile <logs>/dev-profile --command bash --noprofile --norc -c '...'`
keeps the tools from being garbage-collected and keeps a login `PATH` from
shadowing the shell. On macOS, if `xcode-select` points at the Command Line
Tools, set `DEVELOPER_DIR` to the Xcode app for `simctl` and `xcrun`. When
nothing unclaimed remains, deepen open fork PRs or fix CI on upstream
`main`.

## Salvaging a closed pull request

Re-verify each piece moved into the new pull request on its own. Resolve
a conflict from the upstream merge against upstream's current text,
including `TESTING.md`. Re-run the gate on the new head. A green result
on the closed pull request does not carry over.

## Before going upstream

Do not open, comment on, or update an upstream pull request or issue,
and do not contact the maintainer, unless the fork owner has approved
the exact text. Collect questions for the maintainer until then.

1. Review each fork PR against `rind-crypto-review`.
2. Repeat the overlap check from [Picking a quest](#picking-a-quest),
   including upstream issues: `git ls-remote up 'refs/heads/quest/*'`, a
   `chore(quest): claim` commit, and open and merged PRs searched by
   `quest/<milestone>/<area>/<name>` and by title. Also search upstream
   `main` and recently merged PRs for an equivalent change (the same
   `just` recipe, test target, or title). If it already landed or the
   work overlaps, stop and say so rather than opening a duplicate.
   Also re-read that quest's `## Required` and `## Plan` on upstream
   `main`, as [Picking a quest](#picking-a-quest) describes.
3. Rebuild the branch as [rind-git](../rind-git/SKILL.md) describes for an
   upstream-bound branch: one squashed commit on a fresh branch from
   `up/main`, with no fork-only files, even in history.
4. After rebuilding, check upstream's open PRs for overlap: list the files
   each open PR changes, intersect them with yours, and trial-merge the
   overlapping heads locally. Say in the body which PR should land first,
   or that this one rebases cleanly on either.
5. Reconcile against upstream drift (a constant or API removed upstream)
   in a separate small commit, and name it in the Review section as "not
   part of the earlier rounds". Re-run the review on the final head if
   possible, so the table names exactly what gets merged.
6. On the final head, run `just ci`, the ctgrind tests for new secret
   paths, and the full `nix develop --command just check`, with the dev
   shell rooted as
   [Gate locally](../rind-contribute/SKILL.md#gate-locally) describes
   (`just check up/main` the same way). Do not collect Nix garbage while
   it runs. Every `rind-crypto-review` item from the second pass is
   resolved. A pass on an older head is stale once the head moves.
   ctgrind under valgrind needs Linux x86, not Apple Silicon. Re-measure
   benchmark numbers on the rebuilt head, or state the hardware and date
   of the earlier figures.
7. Run `/quest-review` with the other vendor's model and put its `## Review`
   record, naming the head SHA, in the PR body. If it can't run, say so in
   that section rather than leaving it out.

   - Cloud VMs usually lack the reviewer CLIs (`claude`, `codex`), so
     `quest review` exits 2 there. A workstation with both the `claude`
     and `codex` CLIs logged in can run it; quest picks a reviewer that
     is not the last writer. A cloud VM can run the first-pass checks,
     the tests, and ctgrind under valgrind when the VM is Linux x86.
     `/quest-review` and the full `nix develop --command just check` run
     on that workstation. Say in the PR which environment each result
     came from. Use at least medium effort, base the review
     on upstream `main` when the PR targets that, and write the record
     to a file. Do not let the tooling write the record into an upstream
     PR unless that is intended. Build `quest` from the commit the repo's
     flake pins; Bun is enough to build it without Nix.
   - Review against the same base the PR targets. For an upstream PR, that
     is upstream `main`, not the fork's. Review tables must name upstream
     commits as bases. A SHA that exists only on the fork means nothing to
     the maintainer, so map it to the matching upstream `main` commit, or
     say "upstream main as of <date>".
   - A fork pull request's GitHub diff is against the fork's `main`.
     When that branch lags upstream, review `git diff up/main...HEAD`
     (`up` may be named `upstream`) and name that base in the notes.
   - If the clone's remote is a local mirror, quest can't write the record
     into the PR. Paste it in yourself, along with the head SHA it reviewed.
   - If several jobs may push to the same branch, wait for the head to
     settle before a review round. Fetch the fork and upstream before
     every step and immediately before every push, and merge anything
     that moved, as [rind-git](../rind-git/SKILL.md#updating-a-pushed-branch)
     describes.
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
   - After merging the base into a reviewed branch, compare the branch's
     own diff (`git diff <base>...<head>`) before and after. If only hunk
     offsets or context changed, the earlier review still applies.
   - A review round that could not read the repository (for example, git
     was unavailable) and returns a placeholder score does not count.
     Discard it and rerun.
   - Include before/after numbers in a response when a fix touches a hot
     path, so the next round can check the claim.
   - Don't rerun a round while its findings are still open. Fix or refute
     them first, and give the reviewer the responses file along with the
     head SHA it answers.
   - A fix after a second-pass finding resets that finding's review. On
     the fork branch, push it as its own small commit whose subject names
     the fix. Re-check the fix itself: a declassify covers only a true
     public output, such as a verify `Result`. Then run full ctgrind and
     `nix develop --command just check` on the new head. The
     upstream-bound rebuild still squashes, as rind-git says.
8. Before opening the pull request, check `UPSTREAM.md`. It lists every
   workaround for a bug or gap in a tool or dependency rind doesn't own.
   A pull request that adds one adds its entry and names that entry, by
   its heading, in the workaround's comment (`See UPSTREAM.md, "<heading>".`).
   A pull request that drops one removes it from its entry, and deletes
   the entry once it lists no workaround. The rule is rind's
   `CONTRIBUTING.md`, under Upstream.
9. Open one upstream PR as a draft first, and send the rest after the
   maintainer has responded to it. Several large drafts at once from a new
   contributor is a lot to review. Lead with a standalone change, and hold
   ones that overlap the maintainer's open work.

   - List every open maintainer decision in the PR description. Whether
     rustls-webpki and rcgen join `patch`/`fixture` in
     `quest/m2/adoption/aws-lc-rs/aws-lc-rs-patch.md` is that kind of
     question. Leave the decision to the maintainer. Keep the PR a draft
     while any of them is unanswered.
   - Ask a scoping question (for example, which differential reference is
     acceptable) as one short PR comment that names the constraint and the
     options. Don't open an issue for it when the repo tracks work in quest
     files.
   - Before publishing, check that the body has no tooling markers or
     footers (HTML comment markers, "open in" badges).

## Known issues

On Nov 12, 2026, within about an hour, the maintainer opened his own
upstream PRs for four quests the fork had already claimed with fork draft
PRs:

- [moq-dev/rind#674](https://github.com/moq-dev/rind/pull/674) (ecvrf, fork [steelhead99x/rind#19](https://github.com/steelhead99x/rind/pull/19))
- [moq-dev/rind#677](https://github.com/moq-dev/rind/pull/677) (fips/drbg, fork [steelhead99x/rind#24](https://github.com/steelhead99x/rind/pull/24))
- [moq-dev/rind#671](https://github.com/moq-dev/rind/pull/671) (rind-dart-cryptography, fork [steelhead99x/rind#36](https://github.com/steelhead99x/rind/pull/36))
- [moq-dev/rind#670](https://github.com/moq-dev/rind/pull/670) (android-armv7-abi, fork [steelhead99x/rind#21](https://github.com/steelhead99x/rind/pull/21))

Fork draft-PR claims are not visible upstream and don't reserve a quest.
Check upstream open PRs and `quest/*` branches, not just claim commits,
before claiming, again partway through, and again before marking ready.
When upstream takes a quest, stop and hold the fork PR with a comment.
Prefer quests whose files show no recent upstream activity.

1. Never add or invent a quest file. A follow-up a quest file mentions is
   left for the maintainer through `/quest-plan` (rind `CONTRIBUTING.md`,
   "Follow-ups") and raised as a question in the PR.
   `quest/m4/protocols/spake2plus.md` plans a `rind-rs-matter` adapter
   "once this lands", and `quest/m4/protocols/README.md` says a
   `rind-<consumer>` adapter is "planned when one is wanted".
   `quest/m4/protocols/rind-rs-matter.md` is not in upstream `quest/`.
2. Before starting work, map it to a quest path that already exists on
   upstream `main`. rind `AGENTS.md` says to run `quest context` and
   follow it before any work. A clean verification does not make a quest.
   Work that fits no quest of its own belongs in the PR of a quest that
   already owns that work, or it stays held: do not open or merge it on
   its own.
3. A verdict pins a head SHA. rind `CONTRIBUTING.md` ("Reviews") counts
   the `## Review` record as the required review when it names the head
   being merged. The branch can be rewritten after that verdict, so the
   tip is a new SHA. [steelhead99x/rind#1](https://github.com/steelhead99x/rind/pull/1)
   went from verified head `26ee84ec` to `64eb50db` with identical tree
   `4f96b4ef`. Before relying on the verdict, compare
   `git rev-parse <sha>^{tree}`. An identical tree means the verdict
   still covers the content. A different tree needs re-verification.
4. GitHub Actions on the fork can sit queued for a long time: dozens of
   jobs for 30 minutes or more. Wait until every required check has
   completed and passed. A queued or in-progress run is not a pass,
   required or advisory (rind `CONTRIBUTING.md`: fix failing CI checks
   before merging; a path-filtered workflow cannot be a required check).
   Do not push a merge of upstream `main` onto a branch whose CI is
   still running. rind `.github/workflows/check.yml` sets
   `cancel-in-progress: true` on `check-${{ github.ref }}`, so that push
   cancels the run and starts it over.

On Nov 12, 2026, a bulk update merged the fork's `main` into
`spake2plus-up-r7`, the head of open upstream
[moq-dev/rind#685](https://github.com/moq-dev/rind/pull/685).
The fork's `main` carried fork-only files, which then appeared on the
upstream pull request. The merge was reverted, and the history was
later rewritten with the fork owner's approval. Before any sweep,
list the head branches of the fork owner's open upstream pull requests
and exclude them. Those branches take updates from upstream `main`
only, never the fork's `main`. The rule is in
[Updating a pushed branch](../rind-git/SKILL.md#updating-a-pushed-branch).

[moq-dev/rind#652](https://github.com/moq-dev/rind/pull/652)
(merged 2026-11-12, `98568332`) deleted the adapters epic
(`quest/m0/adapters/README.md`) and rewrote several open quest
files. Before resuming work on a quest that was open before that
merge, re-read the quest file on current upstream `main`.

- `quest/m2/api/cpu-check.md` still names pyrind, rind-dart, and
  rind-cpuid in its Goal, but the Plan now says this quest removes
  only rind's own check. Adapter `check()` and `MissingFeature`
  removals ship with rind 0.4 in `quest/m2/api/release-0.4.md`.
- `quest/m2/confidence/mutants-adapters.md` no longer requires the
  adapters epic. It still requires
  `quest/m2/confidence/aws-lc-rs-upstream-tests.md`
  ([moq-dev/rind#679](https://github.com/moq-dev/rind/pull/679)).
- `quest/m2/confidence/adapter-api-diff.md` no longer requires the
  epic.
- `quest/m2/confidence/mutants-pins.md` was added by
  [moq-dev/rind#687](https://github.com/moq-dev/rind/pull/687)
  (merged 2026-11-12, `0f4e45f0`), planned from
  [moq-dev/rind#655](https://github.com/moq-dev/rind/pull/655).
  It shares the adapter `exclude_re` pins with `mutants-adapters`.
  Do not claim it; the maintainer opened it from his own work.

5. [moq-dev/rind#668](https://github.com/moq-dev/rind/pull/668)
   (merged 2026-11-12, `aee2586a`) makes `mutants.yml` run nightly and
   on `workflow_dispatch` only, not on pull requests. The workflow
   header says it never runs on pull requests, like fuzzing. A mutants
   failure or timeout on an upstream pull request is no longer a
   pull-request CI failure to fix on a fork branch. A run started
   before that merge can still fail or hit `timeout-minutes: 300`.
   [moq-dev/rind#663](https://github.com/moq-dev/rind/pull/663) still
   had armv7 shards in progress after the merge
   ([run 43190491966](https://github.com/moq-dev/rind/actions/runs/43190491966)).
   A surviving mutant found that way is a coverage gap to note, not a
   pull-request blocker. `mutants.yml` says a mutant no architecture's
   tests catch fails the nightly run, which `alert.yml` reports: kill
   it with a test, or list it under `exclude_re` in `.cargo/mutants.toml`
   with why no test can. `quest/m2/confidence/portable-fuzz-mutants.md`
   runs that campaign nightly, in `mutants.yml`, not in `just ci`.
   rind `CONTRIBUTING.md` still lists "a mutation run over the PR's
   diff" among advisory runs to fix when red. After #668, `mutants.yml`
   does not start that run on a pull request.
6. A fork Actions queue that looks frozen can be a backlog. On
   Nov 12, 2026, about 100 runs were queued while others were in
   progress, including long `mutants (armv7, ...)` jobs that already
   had a `runner_name`. A queued job can show an empty `runner_name`
   at the same time. Before calling a stall, list in-progress runs
   (`gh run list --repo <owner>/<repo> --status in_progress`) and
   read each job's `runner_name`
   (`gh api repos/<owner>/<repo>/actions/runs/<id>/jobs`). A backlog
   is waiting. It is not a billing or owner blocker. Do not re-queue
   the runs.
7. [moq-dev/rind#669](https://github.com/moq-dev/rind/pull/669)
   (merged 2026-11-12, `ad43e51a`) added
   `quest/m2/confidence/coverage-report.md`,
   `aws-lc-rs-upstream-tests.md`, and `uncovered-functions.md`. The
   maintainer opened upstream pull requests for the first two within
   minutes: [#678](https://github.com/moq-dev/rind/pull/678) and
   [#679](https://github.com/moq-dev/rind/pull/679). Each branch's
   first commit is `chore(quest): claim quest/...md`. Both pull
   requests also edit `uncovered-functions.md`. Newly added quests are
   often claimed immediately. Re-check open upstream pull requests and
   `quest/*` branches right before claiming.
8. A merge of upstream `main` into the fork's `main` names the upstream
   tip in the merge subject. Write that SHA as literal characters. A
   command substitution inside single quotes is stored unexpanded. On
   2026-11-12 the fork commit `a8389b32` stored the subject
   `Merge upstream moq-dev/rind main ($(git rev-parse --short upstream/main))`.
   The next sync, `580c2bb0`, records `a7cf7eeb`.

On Nov 12, 2026, two quest agents ended in an error state after pushing
only their empty claim commit. Roughly an hour of work was lost, and the
quests looked claimed but had no implementation:

- OPRF, fork [steelhead99x/rind#50](https://github.com/steelhead99x/rind/pull/50)
- FROST, fork [steelhead99x/rind#51](https://github.com/steelhead99x/rind/pull/51)

Push work in small, buildable commits as each Plan item lands, rather
than at the end. The hourly loop treats a fork quest draft PR whose only
commit is the claim, and whose agent is in error, as a resume item: reply
to that agent, or launch a new one on the same branch. Upstream
moq-dev/rind's `AGENTS.md`, `CONTRIBUTING.md`, `REVIEW.md`, and quest
files still override this skill.
