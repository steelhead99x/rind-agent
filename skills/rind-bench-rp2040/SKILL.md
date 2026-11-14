---
name: rind-bench-rp2040
description: Use when timing the rind RP2040 harness on one or more boards, including backup, the shared bench lock, SysTick, interleaved A/B runs, and a fork-branch sign-off checklist.
---

# Timing rind on RP2040 boards

Backup and compile checks before any write are in
[rind-bench-baremetal](../rind-bench-baremetal/SKILL.md). Host and
constant-time commands for ordinary machines are in
[rind-bench-and-verify](../rind-bench-and-verify/SKILL.md). This skill
covers flashing and timing the RP2040 bench harness.

The harness is the `rp2040` crate in rind-hardware (or an equivalent
rind-bench port). It is not a build of the rind library crate alone.
Do not flash a board that has no current, verified backup.

## Shared lock

When more than one agent or process can flash or time boards on the same
host, take a lock before any write or timed run:

```shell
mkdir <workdir>/.bench.lock
```

`mkdir` fails if the lock exists; wait. Write an owner file inside the
directory that names the process and the work. Hold the lock only while
flashing, collecting USB serial output, or timing. Release it between
rounds so other work can proceed. Do not take the lock just to compile.
Install a `trap` that releases the lock on EXIT, INT, and TERM so a
killed session does not leave it held. If the lock is held by someone
else, wait and keep `--jobs` modest (about 4).

A host reboot leaves a stale lock directory. Before clearing it, read
the owner file and check whether that process is still alive. Only
ever clear a lock you own. If the owner is another process, wait or
ask; do not delete it.

## Long runs and tools

Put a multi-round timing job under `nohup` (or an equivalent detached
session) with a log file and a short STATUS note that says which images
are queued, which rounds have finished, and how to resume. An
interrupted agent session can then pick the work up from the log
instead of starting over.

Never take bench tools or helper scripts from a shared checkout that
other agents switch branches on. Use a lane-local copy or a dedicated
worktree, and keep that copy on a fixed revision for the run.

## Identify the board

Match a board by USB VID:PID and USB location, then confirm the CDC
port belongs to that same device. A CDC path alone is not enough:
paths move, and they can belong to a different VID:PID.

When two boards report the same USB serial string, the serial is not a
selector. Pick each board by USB location, and run them one at a time.
Timing two such boards in parallel has lost a port.

Pass `picotool --ser <board-serial>` whenever more than one RP2040 is
attached and the serials are unique. Do not commit a serial, a USB
location, a hostname, or a host path.

Check that every board under test is still enumerated before each
round. A board can drop off USB between flashes; stop and ask for it
to be replugged rather than recording a partial round.

If opening the USB serial at 1200 baud does not enter the bootrom, stop
and use the BOOTSEL button. Do not keep writing.

## Back up first

Before the first write to a board, read the whole 2 MiB flash at least
twice (`0x10000000` through `0x10200000`). The sha256 values must match.
Keep the image out of git.

```shell
picotool save -r 0x10000000 0x10200000 backup.bin --ser <board-serial>
```

Restore from BOOTSEL, then confirm the readback sha256:

```shell
picotool load backup.bin -t bin -o 0x10000000 -v -x --ser <board-serial>
```

Record the restore command next to the backup. An old backup from a
different board must not be reused.

## SysTick timing

The RP2040 runs the harness at 125 MHz by default. Cycle counts come
from the 24-bit SysTick counter.

If the SysTick handler does not clear `COUNTFLAG`, a sample that spans a
counter wrap is reported as the true count plus 2^24 cycles (16,777,216).
Confirm the harness includes a wrap fix before timing on a new base.
Numbers taken with the broken timer are only evidence of the bug; label
them and do not use them as a baseline.

## Images and bases

Build each variant into its own `CARGO_TARGET_DIR`. Do not leave
`RUSTFLAGS` in the environment; it overrides the harness rustflags.

Start from current upstream `main` plus the same harness revision for
every image in a comparison. Rebase or rebuild when upstream or the
harness moves. Label any figure taken on an older rind or harness commit
as **old-base**.

Before flashing a "faster multiply" image, disassemble the functions the
patch claims to fix and confirm they contain no `bl __aeabi_lmul`. If
LLVM recombined the 16-bit splits into a widening multiply, the image is
not that variant; do not time it as one. The other codegen checks are in
[rind-armv6m-perf](../rind-armv6m-perf/SKILL.md).

## Interleaved A/B runs

For each pair (or group) of images:

1. Flash one image. The boot known-answer tests (KATs) must print OK.
   A failing or missing KAT invalidates the run; do not keep timing
   that image.
2. Collect the timed rows.
3. Flash the next image and repeat, in the same session.
4. Run at least five such rounds. Alternate which image goes first
   in each round (A then B, then B then A, and so on).
5. Report the median of each row. Record the board identity (VID:PID
   and a placeholder for location, not a committed serial), the rind
   and harness commits, and the date.

When several RP2040 boards are attached, run the same image set on each
and report per-board medians as well as a cross-board median. Do not
touch a port whose VID:PID is not an RP2040 under test.

## Label every number

Every cycle count or ratio must say whether it is:

- **measured** on a board, with the SysTick wrap fix, on a named base;
- **estimated** from a model or emulator (and is not a board timing);
- **old-base**, if the rind or harness commit is not the current one.

Do not combine those without a label. A model that ignores XIP misses
can run 5–20% low on absolute cycles (measured against earlier RP2040
rounds) even when its deltas track the board. A model that assumes
cache hits can be much closer on code that stays in cache: one AES-128
CTR 16 KiB calibration was 4,489,507 modelled cycles against 4,489,798
measured (−0.006%). Quote a model figure only with that caveat.

Layout-sensitive rows (short SHA-256, HMAC, HKDF) can swing tens of
percent from XIP conflicts alone. Treat a change in those rows as a
layout effect until the miss counts say otherwise. See
[rind-armv6m-perf](../rind-armv6m-perf/SKILL.md).

## Sign-off checklist for fork branches

Before calling an RP2040 (or thumbv6m) fork branch ready, all of the
following must pass. A missing item is a "not yet," not a sign-off.

- Release, dev, and clippy builds for `thumbv6m-none-eabi`. The dev
  profile is required: a `std`-only debug helper can fail `cargo build`
  even when `--release` is clean. See
  [rind-bench-baremetal](../rind-bench-baremetal/SKILL.md).
- Host tests.
- Board known-answer tests (KATs) on every board under test. A KAT
  checks correctness only, not timing or constant time.
- Interleaved rounds against the agreed base, with medians labelled
  **measured** or, if fewer than five rounds finished, **1-round**
  (or the actual round count). Do not treat a 1-round figure as a
  sign-off median.
- Identity: every new commit's author and committer are the fork
  owner's GitHub login and noreply address, with exactly one
  `Co-Authored-By: Cursor Agent <cursoragent@cursor.com>` trailer.
  Check with `git var GIT_AUTHOR_IDENT` and `git log -1 --format=%B`.
  The rules are in [rind-git](../rind-git/SKILL.md).
- A constant-time disassembly scan of the image, as in
  [rind-armv6m-constant-time](../rind-armv6m-constant-time/SKILL.md).
- Claim comments on a fork pull request before pushing to its branch:
  read recent comments first; if someone else holds an unexpired
  claim, do not push. Post
  `In progress: <what> (claim expires in 60 min)`, then
  `Done: <what>. New head: <sha>`, or `Released` if you stop.
  Claim comments go on fork pull requests only, never upstream.
  If the branch has no fork pull request, do not invent one just to
  claim; still do not push over someone else's unexpired work.

## After the run

Release the lock (the `trap` should already have done this). Leave the
board in a known state, or restore the backup. Do not commit backups,
serials, or host paths.

On a development fork, keep the work on a feature branch. Do not open a
pull request, comment on GitHub, or merge to `main` unless the fork
owner has asked for that specific action.
