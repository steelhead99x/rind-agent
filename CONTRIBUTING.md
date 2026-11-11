# Contributing

Changes are welcome when a skill has drifted from rind's documentation, or
when a real contribution run shows a step is wrong or missing.

- Keep skills short and imperative. Link to rind's files rather than copying
  long sections from them.
- Every command or path a skill names must exist in rind at the commit the
  README cites. Update that commit when you re-check.
- One skill per directory under `skills/`, each with a `SKILL.md` whose
  frontmatter has `name` (matching the directory) and `description` (one line
  saying when to use it).
- Nothing personal, private, or credential-like, in skills or notes.
- Run `cargo run` (and `cargo test`) before opening a pull request; CI runs the
  same check.
- Conventional-commit subjects (`docs: ...`, `fix: ...`, `chore: ...`), as rind
  uses.

Contributions are made under the licenses in [`LICENSE`](LICENSE).
