# Working notes

Context the agent keeps across sessions. Nothing private goes here.

- Project: rind, pure-Rust cryptography for MoQ by kixelated, forked from
  graviola. Background post: <https://moq.dev/blog/rind/>.
- Fork used for development: <https://github.com/steelhead99x/rind>.
- Policy: all work lands as PRs on the fork first. Nothing goes to
  moq-dev/rind (PRs, issues or comments) until the fork owner reviews it and
  says to send it.
- Backlog: rind's `quest/` directory, worked through `quest skill spawn`.
- Long-running agent work uses the largest context window available; the
  first backlog run used Grok 4.7 at its highest reasoning setting with a 500k
  context window.
