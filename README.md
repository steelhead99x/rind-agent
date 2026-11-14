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
The wasm runner in `sh/wasm.sh`, `REVIEW.md`'s merge-bar list, and the
ECVRF row in `TESTING.md` were checked against
[`c333c59`](https://github.com/kixel-ai/rind/commit/c333c59ef7f5d2fd7846e11974f9d7301d7865df).
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
[`d6ca9a75`](https://github.com/kixel-ai/rind/commit/d6ca9a756d3d1e71489e61e44c04f33c3c893589)
as the comparable-set pin. `0b1f1469` is the previous set and stays
history. The rind-results commands they name were checked against
[`d4a1d803`](https://github.com/steelhead99x/rind-results/commit/d4a1d803519e8c393f908d9342c8961cf0f8c4d6)
on `main`, which has `cargo xtask bench` and the host runners.
`admin/bench-report` at
the pin still sets `--warm-up-time 1` and `--measurement-time 3`, and
its `OPERATIONS` list has 68 entries. The `public_slice` guidance and
the 30-minute `hol-light` and `cosim` limits in
[`rind-bench-and-verify`](skills/rind-bench-and-verify/SKILL.md) were
checked against that same pin.

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
conventional-commit title and every rule traced to rind. `cargo xtask ci`
is the local gate (format, clippy, tests, an MSRV build, and the checker).
GPT review or PrairieDog reviews the pull request. GitHub Actions runs
the same command as a thin backstop. The full process is in
[`CONTRIBUTING.md`](CONTRIBUTING.md),
and security reports go through [`SECURITY.md`](SECURITY.md).

## License

Licensed under any of Apache-2.0, ISC, or MIT-0, at your option, the same
terms as rind. See [`LICENSE`](LICENSE).
