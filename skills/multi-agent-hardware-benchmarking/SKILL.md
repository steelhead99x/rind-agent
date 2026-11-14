---
name: multi-agent-hardware-benchmarking
description: Use when several agents benchmark and optimize code on the same shared physical machines (a laptop, a dev board) so their timed runs don't corrupt each other.
---

# Multi-agent benchmarking on shared hardware

Coordinate with [rind-bench-measure](../rind-bench-measure/SKILL.md) for how
to run a timed set, [rind-git](../rind-git/SKILL.md) for commits and claims,
and [rind-bench-publish](../rind-bench-publish/SKILL.md) for publishing.

1. **Give each agent its own lane.** Split work by area (a primitive, a
   module) so no two agents edit the same files or branches. Each agent gets
   its own checkout or worktree. Never touch pre-existing checkouts.
2. **Keep timed runs scarce.** At most about two agents should run
   benchmarks on the same machines. Extra agents should do work that needs
   no timing on that hardware, such as static or codegen analysis,
   cross-compiling, emulated tests, constant-time and CI verification, or
   docs.
3. **Use a bench lock on every shared machine.** Before a timed run,
   `mkdir ~/<workdir>/.bench.lock` (if it already exists, retry every 30s)
   and write an owner line with the time. Remove the lock right after the
   run. Never compile while someone else holds the lock. Keep runs short and
   release the lock between them.
4. **Measure honestly.** Build each variant into its own target or output
   directory, so stale build caches can't make two variants identical.
   Interleave rounds (variant × mode × round), take medians of 5 or more,
   and record load average and temperature. Discard rounds hit by background
   spikes (indexers, endpoint security) and label them INVALID. Pin to a
   core where the OS allows it.
5. **Check the generated code before timing.** Confirm with a disassembler
   that a source change actually changed the machine code. When using a
   cycle model, pick the model that matches the real core.
6. **Survive interruptions.** Each agent keeps a STATUS.md progress file,
   updates it after every step, and reads it first when resumed. Background
   runs should be detached (`nohup`) so they outlive the agent.
7. **Keep only verified wins.** A change stays only if it measurably helps
   on the target hardware, doesn't regress the others, and passes native
   tests. Push feature branches only, and leave pull requests and public
   comments for the owner to approve.
8. **Coordinate every write.** Before pushing to a branch that backs a fork
   pull request, read its recent comments, skip it if another agent holds an
   unexpired claim, post `In progress: <what> (claim expires in 60 min)`,
   and finish with `Done: <what>. New head: <sha>` or `Released`. Claims go
   on fork pull requests only, never upstream. Never push to a branch that
   backs a pull request a maintainer owns. For shared branches with no pull
   request (such as results or slide branches), name one owner agent per
   branch and route other agents' changes through it. Fetch right before
   each push and push fast-forward only. See also
   [rind-git](../rind-git/SKILL.md#claim-before-you-write).
9. **Commit as the public account only.** Use the identity and any
   co-author trailer the user's memory names, never a local machine, agent,
   or company identity. Run `git var GIT_AUTHOR_IDENT` in the checkout
   before committing, because a repo-local `user.name` or `user.email`
   overrides the global one. Keep local paths, hostnames, IPs, and board
   serials out of committed files.
