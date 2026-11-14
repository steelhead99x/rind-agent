---
name: rind-bench-shared-hardware
description: Use when more than one agent builds or benchmarks rind on the same machine or board, so their timed runs do not disturb each other.
---

# Benchmarking on shared hardware

rind's own report refuses a busy host: `admin/bench-report` measures how
many CPUs other work keeps busy before the build, before the run, and
during it, and records the host as busy or noisy above `busy_limit()`.
[rind-bench-hosts](../rind-bench-hosts/SKILL.md) allows one speed run per
machine and makes other heavy builds wait. How to run a timed set is
[rind-bench-measure](../rind-bench-measure/SKILL.md). Publishing it is
[rind-bench-publish](../rind-bench-publish/SKILL.md). When several agents
share a machine or a board, these rules keep their runs from breaking that.

## Lanes

- Give each agent its own lane: an area (a primitive, a module, a board)
  and its own checkout or worktree. No two agents edit the same files or
  branches. Do not touch checkouts another lane created.
- Keep timed runs scarce. At most about two agents time on one machine.
  The others do work that needs no timing there: static or codegen
  analysis, cross-compiling, emulated tests, constant-time and CI checks,
  docs.

## The bench lock

- Before every timed run, take a lock on that machine with an atomic
  `mkdir <lock-dir>`, and write an owner line in it: the lane, what is
  running, the start time, and the process ID.
- If the directory exists, someone else holds it. Retry every 30 s. Never
  remove another lane's lock, even a stale-looking one; ask its owner.
- Do not compile on that machine, or start heavy work that shares its
  power or cooling, while someone else holds the lock.
- Keep each run short and release the lock right after it, between
  rounds as well, so other lanes can get in.

## Measuring honestly

- Build each variant into its own target directory, so a stale build
  cache cannot make two variants identical.
- Interleave rounds (variant × mode × round), rotate the order, and take
  the median of five or more. Pin each harness to one core where the OS
  allows it (`taskset -c <cpu>`).
- Record the load average, the temperature, the clock, and any throttle
  state before and after each round. A clock cap or thermal throttling
  changes absolute times; name the clock in the published set.
- Label a round hit by background work (an indexer or endpoint
  security), a reboot, or a power loss INVALID, keep it out of every
  median, rerun it in full, and say so in the set's notes.
- Before timing a source change, check with a disassembler that the
  machine code actually changed. When the timing comes from a cycle
  model, use the model of the real core.

## Interruptions

- Each agent keeps `STATUS.md` in its lane checkout and updates it after
  every step. Read it first when resumed.
- Start long runs detached on the machine (`nohup setsid ...`) so they
  survive a dropped connection, and write their logs there.

## Writes

- Keep a change only if it measurably helps on the target hardware, does
  not regress the others, and passes native tests there.
- Before pushing to a branch that backs a fork pull request,
  [claim before you write](../rind-git/SKILL.md#claim-before-you-write).
  A shared branch with no pull request (results, slides) has one owner
  agent; route other lanes' changes through it.
- Fetch right before each push, and push fast-forward only.
- Commit identity and trailers follow [rind-git](../rind-git/SKILL.md).
  Before committing, run `git var GIT_AUTHOR_IDENT` in that checkout: a
  repo-local `user.name` or `user.email` overrides the global one.
  Keep local paths, host names, addresses, and device IDs out
  of committed files, results included.
- Push the feature branch. Opening its pull request, and a comment on a
  public thread, waits until the owner asks.
