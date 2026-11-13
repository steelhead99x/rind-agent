# Contributing

This repository holds skills for contributing to
[rind](https://github.com/moq-dev/rind) and a small Rust checker that keeps
them honest. Its process follows rind's own
[`CONTRIBUTING.md`](https://github.com/moq-dev/rind/blob/main/CONTRIBUTING.md),
scaled down to a documentation repository.

Changes are welcome when a skill has drifted from rind, or when a step in a
skill is wrong or missing. For anything larger than a fix, open an issue
first so the scope is agreed before the work.

## What belongs in a skill

- **Evidence.** Every rule traces to something you can check in rind: a file
  (`AGENTS.md`, `REVIEW.md`, `CONTRIBUTING.md`, a script under `admin/` or
  `sh/`), a CI workflow, a `justfile` recipe, or a review finding on a
  public PR. Name the source in the PR's **Testing** section. No speculation,
  and nothing that only held on one setup.
- **Pinned.** Every command and path a skill names exists in rind at the
  commit the README cites. When you re-check against a newer rind, update
  that commit in the same PR.
- **Short and imperative.** Link to rind's files rather than copying long
  sections. Where a skill and rind disagree, rind wins; fix the skill.
- **Shape.** One skill per directory under `skills/`, named in lowercase
  kebab-case. Its `SKILL.md` opens with YAML frontmatter: `name`, equal to
  the directory, and `description`, one line starting "Use when". Each skill
  has a row in the README's table.
- **Public.** Nothing about the contributor or their environment: no
  personal names, account handles, email addresses, home directory paths,
  host names, machine or session IDs, tokens, or private repositories. Write
  examples with placeholders (`<fork>`, `/path/to/rind`). The checker
  refuses the common shapes of these; it is a backstop, not a review.

## Local gate

Run the same checks CI runs before every push:

```shell
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- --deny warnings
cargo test --workspace --locked
cargo run --locked --quiet    # skills, links, privacy, action pins
```

`cargo run -- pr` checks a PR's title and body from the `PR_TITLE` and
`PR_BODY` environment variables, as the `PR format` job does. The checker
must build with the `rust-version` in `Cargo.toml` (the `MSRV` job) and has
no dependencies; adding one needs a reason in the PR. Workspace tools may
take dependencies. `gen-frames` needs FreeType and librsvg headers at build time
(`libfreetype6-dev` and `librsvg2-dev` on Debian, or Homebrew `freetype`,
`librsvg`, and `pkg-config`).

## Commits

PRs land on `main` as squash commits, so the PR title becomes the commit
subject in `git log`.

- Conventional-commit subjects: `type(scope): subject`, scope optional,
  at most 72 characters, no trailing period. `!` after the type or scope
  marks a breaking change, such as a renamed skill (`feat!: ...`).
- Types: `feat`, `fix`, `docs`, `chore`, `ci`, `test`, `refactor`, `revert`.
- Scopes: a skill without its `rind-` prefix (`docs(crypto-review): ...`),
  `check` for the checker (`feat(check): ...`, `fix(check): ...`), or `ci`.
- Co-author attribution goes in a `Co-Authored-By:` trailer, not the body.
- When squash-merging, pass `--author-email` set to the account's GitHub
  noreply address and a `--body` with exactly one `Co-Authored-By:`
  trailer, then verify the resulting `main` commit's author, committer,
  and trailer count.
- Never commit binaries or build artifacts.

## Pull requests

Every change lands through a pull request; nobody pushes to `main`. Keep
each PR to one change: one skill rule, one checker feature, one CI change.

The body is short and structured, not narrated, with these sections, which
the `PR format` job requires (write "None." where one doesn't apply):

- **Problem**: what is wrong or missing, and why this PR is needed.
- **Approach**: how the PR solves it.
- **Impact**: one bullet per skill or checker behavior that changes. A
  breaking one says how to migrate, and the title carries `!`.
- **Testing**: the local gate's result, and the rind file, commit, or PR
  that supports each changed rule.
- **Alternatives**: approaches considered and not taken.
- **Follow-ups**: anything found and left for later.

The [pull request template](.github/pull_request_template.md) has these
sections and a checklist.

- Open a draft while working; mark it ready when the gate passes.
- When pushing more commits, update the title and body if the change moved.
- Never rebase or force-push a pushed branch; merge `main` in instead.
- Resolve every review conversation before merging.
- Fix merge conflicts and failing checks; `Check`, `MSRV`, and `PR format`
  are required.

## CI

Workflows pin every third-party action to a full commit SHA with its
version in a trailing comment, and the checker fails on one that isn't.
Dependabot proposes the bumps weekly, after a week's cooldown. Workflows
start from `permissions: contents: read` and don't persist checkout
credentials.

## Security

See [`SECURITY.md`](SECURITY.md). Vulnerabilities in rind itself go to rind,
not here.

## License

Contributions are made under the licenses in [`LICENSE`](LICENSE):
Apache-2.0, ISC, or MIT-0, at the user's option.
