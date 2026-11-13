---
name: rind-bench-port
description: Use when porting a Rust crypto bench suite to an attached microcontroller and publishing firmware and results.
---

# Porting a bench suite to a board

How rind itself compiles for microcontrollers, and how to back up a
board before a write, is in
[rind-bench-baremetal](../rind-bench-baremetal/SKILL.md). Host benches
and how to publish a side sheet are in
[rind-bench-hosts](../rind-bench-hosts/SKILL.md) and
[rind-bench-results](../rind-bench-results/SKILL.md).

## 1. Inspect the hardware

The board is plugged into the user's machine, so inspect it there.

- List serial ports and use `udevadm` to tell boards apart by
  vendor and model. Never touch a board that is not the target.
- A "BOOT" USB name does not mean the device is in its bootloader.
  Look for a mass-storage interface and check who holds the port
  (services such as bridges).
- Identify the chip with the vendor tool (`esptool chip_id` and
  `flash_id`, or the equivalent): arch, cores, clock, flash, PSRAM.
- Check which toolchains exist (`rustup`, `espup`, `espflash`,
  ESP-IDF) and install user-local only, with no `sudo`.
- For a display, run an I2C scan and read existing projects for pin
  configs. Do not guess the hardware.

## 2. Back up before flashing firmware you do not own

- Export the app config and info, copy the full current firmware
  (UF2 `CURRENT.UF2` or similar), fetch the matching official release
  as a second restore path, and checksum everything.
- Flash only the app region, and verify the SoftDevice and bootloader
  regions are byte-identical afterwards.
- Restore, diff the info before and after, and restart any services
  you stopped.

## 3. Decide what to measure

- Read the upstream project's own bench code, benchmark report, and
  any author write-up. Reuse its operations, sizes, and competitor
  set so results line up with its published numbers.
- Check whether the library has assembly for the target. If not, say
  the portable path is being measured.
- Try every competitor on the target. When one fails to build, record
  the concrete reason instead of dropping it silently.
- Add the hardware accelerators as a clearly labelled baseline when
  practical.

## 4. Benchmark honestly

- Correctness first: every candidate must pass known test vectors and
  agree with the others. Only race the ones that pass.
- Same work: same keys, nonces, sizes, core, and clock, plus the same
  seeded RNG when an op draws randomness. Radios and interrupts off,
  caches warmed up.
- Time with the cycle counter, take many samples, and report the
  median plus spread, ns/op, cycles/op, and cycles/byte. Include
  binary size.
- Run the full suite at least three times. Also rebuild once with a
  different layout: results that repeat exactly can still shift
  0.6–1.6× on small inputs after a relink.
- Keep UI, display work, and prints out of timed regions: anything
  between timed calls (even one print) can evict the instruction
  cache and inflate a measurement 10×. Time calls in a tight loop
  and print afterwards. Start runs on an explicit host go line,
  since a host modem manager can swallow serial output.
- Host benches: the upstream tool may refuse a busy host. Disclose
  any relaxed limit and keep the noisy flag, and do not build while
  a run is going.

## 5. Explain every gap with evidence

- The upstream report tool refusing undocumented gaps is the point:
  each row over its threshold needs a measured reason, not a guess.
- A large fixed per-call cost hidden at big sizes is
  instruction-cache misses until shown otherwise. Rebuild with a
  doubled instruction cache as a diagnostic; publish at the default
  config and keep the bigger-cache runs as diagnostics.
- To rule the RNG in or out, count it: wrap the platform RNG backend
  (for example getrandom's custom backend) with call, byte, and cycle
  counters, and add a seeded deterministic RNG behind a feature for
  a second build. When adding cfg rustflags, pass them via cargo
  `--config` (which merges with `config.toml`) rather than
  `RUSTFLAGS` (which replaces them).
- Profile without JTAG when debug access needs root: a high-priority
  RTOS task on the other core can sample the preempted task's saved
  PC each tick; take a few hundred samples, run `addr2line`, and
  classify by function. 64-bit-limb code on 32-bit cores is a common
  culprit.

## 6. Verify before you publish

- Recompute every published number from raw with a separate script
  that shares no code with the report generator: table cells, chart
  labels, display pages, the fastest-library call, ratios and
  spreads.
- Record provenance at build time, not report time: per-image
  sha256, source commits, patches, build time.
- Recheck by reflashing the published images (verified by hash) and
  rerunning; medians should land within max(1%, run spread).
- Build self-checks into the firmware: cycle counter vs an
  independent timer at boot, wall time vs cycles per benchmark, and
  the empty-call harness cost (report it next to the fastest row;
  do not subtract).
- Check time grows with input size; a row that gets faster with more
  input may be optimised out or a layout effect.
- Compare against public figures for the same chip (vendor minimum
  hardware throughput, third-party software numbers).
- Gaps in bench files the pinned report does not know still need a
  recorded reason.
- Re-pull upstream before finishing; rerun affected benches as a
  separate labelled set rather than mixing into the pinned set.
- ESP32: `espflash` writes the flash clock into the image header and
  defaults to 40 MHz regardless of `sdkconfig`. Always pass
  `--flash-freq 80mhz` and confirm the bootloader's `Boot SPI Speed`
  line in every run log; a wrong clock makes cache-missing rows up
  to about 2× slow.
- Relink at least once and publish per-row shifts; on small
  instruction caches even the target library's short-input rows
  move a lot, so never claim layout immunity from one relink.
- Display pages: keep headings unique and fitting; fail generation
  instead of truncating.
- A patch script running `git submodule update --init` resets a
  hand-checked-out submodule; restore the pin before rerunning.
- Shared devices: keep the window short (trim to the essential
  runs), and measure downtime from the service's own "connected"
  log line, not from `systemctl is-active` (a service can
  auto-restart and show active while it cannot reach the device).
  The first image flashed over a running vendor firmware may reset
  early, possibly from a hardware watchdog it left armed; record a
  run that fails to start rather than silently retrying.
- When the pin moves, check whether the upstream report generator
  gained operations, and measure layout sensitivity with padded
  relinks (for example shifts of 0, about 1 KB, about 2 KB) and
  publish per-layout ratios plus the median.
- nRF52 boards coming off a vendor firmware or bootloader may have
  the hardware watchdog running (it cannot be stopped short of a
  watchdog, pin, or power-on reset). Read WDT `RUNSTATUS` / `CRV` /
  `RREN` at boot, log them, and reload every enabled RR register
  between timed regions and while idle. Also stop any companion
  timer or watcher that restarts the device's bridge service during
  the window, and do not `chmod` or copy metadata onto a UF2 drive
  (it unmounts after the write).
- Check with `nm -n` what a padded relink actually moved before
  describing it: padding can land after the libraries, moving only
  the harness. Put padding ahead of the code you want to shift, and
  record each library's shift.
- Before calling a post-repin shift a code change, diff the upstream
  source for that path, the lock file, and function sizes (`nm -S`).
  If all match, compare relative placement and rerun with a cache
  large enough to hold the path; identical cycles there mean layout.
- Rerun the unpadded build in the same session as a control, and
  include a padding that differs mod 16 (paddings 1 KB apart keep
  the same alignment and may time identically). Label any relink
  that moved libraries only a few bytes as such.

## 6b. Rewriting tooling in another language

- Prove parity before deleting the old tools: regenerate every
  artifact with both and diff; record byte-identical vs wording-only
  vs pixel-identical differences in a diag folder.
- Byte-matching JSON needs the same key order, separators, and float
  formatting; Rust's `{:.2}` rounds like Python's `.2f`. PNG bytes
  only match with the same zlib, so compare pixels.
- Keep the verifier independent of the report generator (no shared
  computation code) and test it with a deliberate mutation.
- Copy instead of rename when writing outputs that may cross
  filesystems.

## 7. Debugging crashes

- Run the image in the vendor's QEMU to get clean faults when
  hardware reset-loops.
- On Xtensa, over-aligned locals (align > 16) can make LLVM realign
  the stack without `movsp`, which breaks the register windows:
  interrupt saves then corrupt locals. Search the disassembly for
  functions that realign the stack; a small patch lowering the
  alignment on that target fixes it. Verify on hardware before
  calling it fixed.
- Work out stack size from the disassembled frame sizes. ESP-IDF
  `sdkconfig` paths need to be absolute when the target dir is
  elsewhere.
- A reset-looping board can get its USB port disabled by the hub,
  and recovering it then needs a physical replug. Disable
  reboot-on-panic for debug builds to avoid this.

## 8. Structure for upstream

- Follow the upstream project's contribution rules: license, fmt,
  clippy, tests, commit style, unsafe and asm policy, and report
  rules such as gap reasons.
- Keep custom glue and patches small and in a clearly separate
  place, with minimal repros. Never open upstream PRs or issues
  without the fork owner's go-ahead.
- Firmware port and results or tools can live in separate
  repositories that link to each other. Match upstream's license
  and keep attribution.
- For rind board work the firmware repository is
  [steelhead99x/rind-hardware](https://github.com/steelhead99x/rind-hardware)
  (public). Recorded provenance SHAs from an earlier, now-archived
  checkout live in `legacy_rind_bench`; `rind_hardware` maps them
  onto the public history (`tools/legacy-rind-bench-map.json` in the
  results repository). Map by tree, or treat squash root `8d9c1bb`
  as a match when the only differences are the publish-time rename
  and secret scrub (RP2040 tip `f656f95`). Earlier SHAs are
  ancestors: write `ancestor of 8d9c1bb5e76e5ae8eb2d481535dc63ccd6a8f5f3`
  and keep the recorded string in `legacy_rind_bench`. Leave crate,
  binary, and protocol names (`rind-bench` crate, `rind-bench:go`,
  `rind-bench-esp32`, `.text.rind_bench_pad`) alone.
- Scan for home-directory paths before publishing. Other agents may
  push to the same repository, so fetch before every push, and
  commit often.
- Media: render display pages on the PC at native and 4× size, and
  record terminals with `script` and `asciinema` with names and
  paths scrubbed.
- A display or demo build that embeds the published results goes
  stale whenever results change: regenerate the embedded table and
  rebuild before flashing.
- When a results checkout has diverged after a history rewrite,
  back up first (`git bundle --all`, diffs, a tar of small
  untracked files and a manifest of huge ones). Compare local-only
  commits to origin by tree and patch-id; do not cherry-pick content
  that is already on the rewritten history or that would re-introduce
  scrubbed secrets. Reset to `origin/main` and re-apply uncommitted
  edits with `git apply --3way`; leave a conflicting patch and
  report it. Never stash or discard another agent's work. Commit in
  a clean worktree, rebase, and do not force-push if that would drop
  someone else's commits.
- A first rewrite can miss home-relative paths in rustc backtraces
  and cargo compile lines (a `~/…` path, or a cargo or rustc path
  under a home directory). Scan for `~/` and other home-directory
  paths as well as ports and MACs. A `by-id` path built from the
  firmware's own USB product string is a firmware identifier, not a
  host-specific serial.

## 9. Report

Give the repository URLs, what ran vs what failed (with reasons), a
headline table comparing the target library with the best competitor
per operation, surprises, and open issues.
