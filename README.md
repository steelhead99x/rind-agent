# rind-agent

Agent skills for contributing to [rind](https://github.com/kixel-ai/rind), the
pure-Rust cryptography library behind [MoQ](https://moq.dev).
`moq-dev/rind` redirects there.

Each skill is a `SKILL.md` file: YAML frontmatter (`name`, `description`)
followed by Markdown instructions. Claude Code, Cursor, and other agents that
load skills in this format can use them directly, and they work as plain
checklists for people too.

The skills condense rind's own
[`AGENTS.md`](https://github.com/kixel-ai/rind/blob/main/AGENTS.md) and
[`CONTRIBUTING.md`](https://github.com/kixel-ai/rind/blob/main/CONTRIBUTING.md).
Where they disagree, rind's repository is authoritative. They were last
checked against rind at
[`7ab9f85`](https://github.com/moq-dev/rind/commit/7ab9f8524e27d8a186d3eb35681d07cd2a5203d7).
Mutation testing in
[`rind-bench-and-verify`](skills/rind-bench-and-verify/SKILL.md) and the
platform-workflow sentence in
[`rind-contribute`](skills/rind-contribute/SKILL.md) were checked against
[`aee2586a`](https://github.com/moq-dev/rind/commit/aee2586a75ed4ef6838f9c0b09f13070d9661486).

This is an independent project, not maintained by or affiliated with moq-dev.

The `just qemu` shell list in
[`rind-bench-and-verify`](skills/rind-bench-and-verify/SKILL.md) was checked
against
[`aee2586a`](https://github.com/moq-dev/rind/commit/aee2586a75ed4ef6838f9c0b09f13070d9661486).

The published-benchmark skills
([`rind-bench-pin`](skills/rind-bench-pin/SKILL.md),
[`rind-bench-measure`](skills/rind-bench-measure/SKILL.md),
[`rind-bench-schema`](skills/rind-bench-schema/SKILL.md),
[`rind-bench-publish`](skills/rind-bench-publish/SKILL.md)) use upstream
[`0b1f1469`](https://github.com/kixel-ai/rind/commit/0b1f1469c8e17e361e3353432bc7fceee9e04214)
as the comparable-set pin. The rind-results commands they name were
checked against
[`8ab4c317`](https://github.com/steelhead99x/rind-results/commit/8ab4c3173953a800eb49c9d8750320defa7c589b).
`admin/bench-report` at that pin still sets `--warm-up-time 1` and
`--measurement-time 3`, and its `OPERATIONS` list has 68 entries.

## Skills

| Skill | Use it when |
| --- | --- |
| [`rind-contribute`](skills/rind-contribute/SKILL.md) | Making a change: setup, the change itself, and the local gate. |
| [`rind-git`](skills/rind-git/SKILL.md) | Branching, committing, pushing, or squash-merging a change. |
| [`fork-to-upstream`](skills/fork-to-upstream/SKILL.md) | Reviewing, cleaning up, and sending ready fork branches upstream as pull requests. |
| [`rind-crypto-review`](skills/rind-crypto-review/SKILL.md) | Reviewing a diff: correctness, hostile input, constant time, wiping, API shape. |
| [`rind-bench-and-verify`](skills/rind-bench-and-verify/SKILL.md) | Benchmarks, cross-architecture tests, constant-time tooling, proofs. |
| [`rind-bench-pin`](skills/rind-bench-pin/SKILL.md) | Choosing the upstream commit a published set measures, and recording fork patches. |
| [`rind-bench-measure`](skills/rind-bench-measure/SKILL.md) | Running that set with rind-results `cargo xtask bench`, including a board you were allowed to flash. |
| [`rind-bench-schema`](skills/rind-bench-schema/SKILL.md) | Schema 1 plus the optional fields, and `results/<host>/<upstream-sha>/`. |
| [`rind-bench-publish`](skills/rind-bench-publish/SKILL.md) | `cargo xtask verify --dir` before publishing, and fork-only benchmark pull requests. |
| [`rind-bench-hosts`](skills/rind-bench-hosts/SKILL.md) | Host notes outside `admin/bench-report`'s six. Published sets use `rind-bench-measure`. |
| [`rind-bench-results`](skills/rind-bench-results/SKILL.md) | Where a published JSON tree goes, and which older trees stay as history. |
| [`rind-bench-baremetal`](skills/rind-bench-baremetal/SKILL.md) | Backup and compile checks. Flashing waits for an explicit yes. |
| [`rind-quest-run`](skills/rind-quest-run/SKILL.md) | Working rind's `quest/` backlog with `quest skill spawn` or `quest skill run`. |
| [`rind-quest-fleet`](skills/rind-quest-fleet/SKILL.md) | Running more than one rind quest from a fork. |

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
