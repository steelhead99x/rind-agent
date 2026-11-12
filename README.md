# rind-agent

Agent skills for contributing to [rind](https://github.com/moq-dev/rind), the
pure-Rust cryptography library behind [MoQ](https://moq.dev).

Each skill is a `SKILL.md` file: YAML frontmatter (`name`, `description`)
followed by Markdown instructions. Claude Code, Cursor, and other agents that
load skills in this format can use them directly, and they work as plain
checklists for people too.

The skills condense rind's own
[`AGENTS.md`](https://github.com/moq-dev/rind/blob/main/AGENTS.md) and
[`CONTRIBUTING.md`](https://github.com/moq-dev/rind/blob/main/CONTRIBUTING.md).
Where they disagree, rind's repository is authoritative. They were last
checked against rind at
[`34c65da`](https://github.com/moq-dev/rind/commit/34c65da32f687b127375e83c6963327bbff389c9).

This is an independent project, not maintained by or affiliated with moq-dev.

## Skills

| Skill | Use it when |
| --- | --- |
| [`rind-contribute`](skills/rind-contribute/SKILL.md) | Making any change: setup, branching, `just check`, commit and PR format. |
| [`rind-crypto-review`](skills/rind-crypto-review/SKILL.md) | Reviewing a diff: correctness, hostile input, constant time, wiping, API shape. |
| [`rind-bench-and-verify`](skills/rind-bench-and-verify/SKILL.md) | Benchmarks, cross-architecture tests, constant-time tooling, proofs. |
| [`rind-quest-run`](skills/rind-quest-run/SKILL.md) | Working rind's `quest/` backlog with `quest skill spawn`. |

[`NOTES.md`](NOTES.md) has background on rind and where changes land.

## Install

Per project, into a rind checkout:

```shell
git clone https://github.com/steelhead99x/rind-agent
cp -r rind-agent/skills/* /path/to/rind/.claude/skills/
```

Use `~/.claude/skills/` to install for every project, or `.agents/skills/` for
agents that read that directory. The names don't collide with rind's own
`quest-*` skills.

## Contributing

Changes land through pull requests only, one change per PR, with a
conventional-commit title and every rule traced to rind. `cargo run` checks
the skills, links, and workflow pins; CI runs it with fmt, clippy, tests,
and an MSRV build. The full process is in [`CONTRIBUTING.md`](CONTRIBUTING.md),
and security reports go through [`SECURITY.md`](SECURITY.md).

## License

Licensed under any of Apache-2.0, ISC, or MIT-0, at your option, the same
terms as rind. See [`LICENSE`](LICENSE).
