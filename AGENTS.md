# Agent instructions

[`CONTRIBUTING.md`](CONTRIBUTING.md) is the source of truth for skill
format, privacy, pull requests, and the local gate. Follow it. Where this
file and `CONTRIBUTING.md` disagree, `CONTRIBUTING.md` wins.

Before pushing, run `cargo xtask ci`. That command is the gate: format,
clippy with warnings denied, tests, an MSRV build at the `rust-version`
in `Cargo.toml`, and the checker. GitHub Actions runs the same command
as a thin backstop. GPT review or PrairieDog reviews the pull request.

## Reviews

Be terse and specific. Cite the file and the `CONTRIBUTING.md` rule that
is broken. These checks are part of the review, including on docs and
workflow changes.

Flag a title or body that breaks the Commits or Pull requests sections
of `CONTRIBUTING.md`. The title is a conventional commit. The body has
each section that file lists, with text other than an HTML comment.
"None." counts. Dependabot's body is exempt; its title is not.

### `skills/`

Review skills as documentation that agents will follow.

Accuracy: every rule should be checkable against rind or this repository
(`CONTRIBUTING.md`, a named `admin/` or `sh/` script, a workflow, or a
`justfile` recipe). Flag speculation, stale commands, or advice that
conflicts with another skill or with `CONTRIBUTING.md`. Where a skill
and rind disagree, rind wins. Link to rind's files rather than copying
long sections.

Format: one skill per directory under `skills/`, lowercase kebab-case,
`SKILL.md` with YAML frontmatter whose `name` equals the directory and
whose `description` starts with "Use when". Keep prose short and
imperative. A new skill needs a row in the README table.

Privacy: no personal names, account handles, email addresses, home
directory paths, host names, machine or session IDs, tokens, or private
repositories. Examples must use placeholders such as `<fork>` and
`/path/to/rind`. Do not invent repository history, benchmark numbers,
or dates.

### Markdown

Check accuracy and consistency with the skills and with
`CONTRIBUTING.md`. Flag private paths, hosts, account details, or
secrets. Keep the README skill table in sync with `skills/`.

### `src/`

The checker is a backstop for skill format, links, privacy, and pins.
Flag changes that weaken those checks, that disagree with
`CONTRIBUTING.md`, or that would miss a private path, host, or secret
the docs forbid. Tests must not introduce private paths, hosts, or
secrets.

### `.github/`

`Check` runs `cargo xtask ci` and nothing else. Flag a workflow that
re-implements that gate, skips it, or leaks a secret. Third-party
actions stay pinned to a full commit SHA with a trailing version
comment.
