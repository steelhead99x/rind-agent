# rind-agent

Agent skills for contributing to [rind](https://github.com/moq-dev/rind), the
pure-Rust cryptography library behind [MoQ](https://moq.dev).

These are plain Markdown instructions (`SKILL.md` with YAML frontmatter). Any
agent that reads skills in that format can use them, and they read fine as a
human checklist too. They summarize rind's own `AGENTS.md` and
`CONTRIBUTING.md`; when the two disagree, rind's repo wins.

## Skills

| Skill | Use it when |
| --- | --- |
| [`rind-contribute`](skills/rind-contribute/SKILL.md) | Making any change: setup, branch, `just check`, commit and PR format. |
| [`rind-crypto-review`](skills/rind-crypto-review/SKILL.md) | Reviewing a diff: correctness, hostile input, constant time, wiping, API. |
| [`rind-bench-and-verify`](skills/rind-bench-and-verify/SKILL.md) | Benchmarks, cross-arch tests, constant-time tooling, proofs. |
| [`rind-quest-run`](skills/rind-quest-run/SKILL.md) | Working the `quest/` backlog with `quest skill spawn`. |

[`NOTES.md`](NOTES.md) holds the public working notes the agent keeps between
sessions.

## Install

Claude Code, per project:

```shell
git clone https://github.com/steelhead99x/rind-agent
cp -r rind-agent/skills/* /path/to/rind/.claude/skills/
```

Or user-wide with `~/.claude/skills/`. For agents that read `.agents/skills/`,
copy them there instead. The skill names don't collide with rind's `quest-*`
stubs.

## Contributing

Fix a skill when it drifts from rind's docs or when a real run shows a step is
wrong or missing. Keep them short and imperative, and link to rind's files
instead of copying long sections. Don't add anything personal or private.

## License

Apache-2.0 OR ISC OR MIT-0, at your option, the same as rind. See
[`LICENSE`](LICENSE).
