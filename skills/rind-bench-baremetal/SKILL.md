---
name: rind-bench-baremetal
description: Use when building rind for a microcontroller, backing up a board before a write, or checking whether a flash has been approved.
---

# Microcontrollers

Upstream [kixel-ai/rind](https://github.com/kixel-ai/rind) at the
comparable-set pin has no `#![no_std]` build. At `a4759c48` with rustc
1.99.1, `cargo build -p rind --release --target thumbv6m-none-eabi`
needs `RUSTFLAGS='--cfg getrandom_backend="custom"'`, then fails with
`can't find crate for std` (1672 errors on that build).
`std::thread_local!` is in `rind/src/low/cpu.rs` and
`rind/src/low/entry.rs`. That failure is the upstream tree. It is not
the firmware that has already been flashed.

[rind-hardware](https://github.com/steelhead99x/rind-hardware) builds
ESP32-S3, nRF52840, and RP2040 images from a fork of the pin plus
committed patches (`patches/0003` is `no_std`, `patches/0005` is the
cache without CAS). ESP32-S3 and nRF52840 run. RP2040 builds, and
known-answer checks have passed on a Pico. rind-results has no
published RP2040 timing tree yet. The compile-only matrix in
[COMPETITOR-BUILDS.md](COMPETITOR-BUILDS.md) is upstream rind without
those patches.

Ask before any flash, erase, or unlock. The commit is
[rind-bench-pin](../rind-bench-pin/SKILL.md), the run is
[rind-bench-measure](../rind-bench-measure/SKILL.md), and the publish
is [rind-bench-publish](../rind-bench-publish/SKILL.md). Timing the
RP2040 harness, including the shared lock and SysTick, is
[rind-bench-rp2040](../rind-bench-rp2040/SKILL.md). Do not add a
separate skill for another board port. The portable-backend
dependencies do compile for the targets below.

## Dev-profile no_std builds

Gate any std-only debug helper, including `thread_local`, to
`cfg(feature = "std")` or `cfg(not(target_os = "none"))`. A debug-only
counter that uses `std` will fail
`cargo build --target thumbv6m-none-eabi` (the dev profile) even when
the release image links. Include that dev build, not only `--release`,
when verifying a no_std or thumbv6m branch. The sign-off list in
[rind-bench-rp2040](../rind-bench-rp2040/SKILL.md) requires it.

## Back up first

Read the flash at least twice. The sha256 values must match. Keep the
image out of git. Do not erase. Do not unlock debug.

RP2040 (Cortex-M0+, 2 MiB at `0x10000000` through `0x10200000`, 264 KiB
RAM): open the USB serial at 1200 baud to enter the bootrom, then
`picotool save -r 0x10000000 0x10200000 backup.bin`. Save a second read.
The sha256 values must match. When several boards are attached, select
the board by USB VID:PID and USB location, then pass `picotool --ser
<board-serial>`. A CDC path alone is not enough: paths move, and they
can belong to a different device. If 1200 baud does not enter the
bootrom, stop and use the BOOTSEL button; do not keep writing.

Restore from BOOTSEL, then confirm the readback sha256:

```shell
picotool load backup.bin -t bin -o 0x10000000 -v -x
```

Add `--ser <board-serial>` when more than one board is attached. Do not
commit a serial, a USB location, or a host path.

EFR32MG24 (Cortex-M33, CMSIS-DAP, probe-rs): the part that was read has
1536 KiB of main flash at `0x08000000` through `0x08180000`. Read it three
times. A 1 KiB user-data page at `0x0FE00000` was read twice and left
unwritten. The bootloader occupies `0x08000000` through `0x08006000`. A
demo write covers only the application region after that.
[mg24-restore.sh](mg24-restore.sh) downloads `BACKUP` at `BASE` with
`--verify`, reads `LENGTH` bytes back, checks the sha256, and resets. It
does not erase or unlock. Set `CHIP` to the probe-rs part name. Set
`PROBE` only when several probes are attached, and do not commit it.

### EFR32MG24 bring-up

- rind-hardware's `efr32mg24/` port runs at 78 MHz: HFXO (39 MHz),
  DEVINFO's HFRCODPLL calibration for the 80 MHz band, DPLL0 at HFXO
  times 2000 / 1000, one flash wait state, PCLK = HCLK / 2.
- A debugger reset keeps the CMU as the last image left it. Switch
  SYSCLK back to FSRCO and stop DPLL0 before reprogramming the clocks.
- Series 2 interrupt flags clear only through the register's clear
  alias (`+0x2000`). A plain write leaves them set, so a wait on the
  flag passes at once and reads a stale result.
- Writing `CMU_CALCTRL` while a calibration runs is a precise
  BusFault. Wait for `CMU_STATUS.CALRDY` first. The up count must fit
  CALCNT's 20 bits.
- Check the clock against HFXO with the calibration counter (HCLK as
  the down clock) and put the result in the start record.
- Output over SEGGER RTT through the on-board CMSIS-DAP needs no UART
  pins. Drain the buffer before each benchmark.
- `probe-rs run` programs from `0x08000000`, over the bootloader.
  Restore the full backup afterwards. Do not run it until a flash of
  that image was approved.
- ring's Thumb-2 patch for `thumbv7em` also builds for
  `thumbv8m.main`.
- After a restore, the original app can boot before the readback and
  append NVM3 records to a page that was erased in the backup. A
  readback that differs only there (backup bytes all `0xFF`) is a good
  restore, provided `probe-rs download --verify` passed. Diff the two
  images to confirm.

### ESP32-S3 serial

If esptool or espflash reports that no serial data arrived on the
USB-Serial/JTAG port, run `probe-rs reset --chip esp32s3` on the same
USB-JTAG probe first, and only after the flash itself was approved.
On a fresh rind-hardware checkout, build `esp-idf-sys` before ring's C
files, or the C build runs before the Xtensa GCC is installed. Format
with the pinned rustfmt. `rustfmt +stable` will install a toolchain on
its own.

## What compiles

[build-matrix.sh](build-matrix.sh) release-builds each `crates/tv-*`
member and retries with `getrandom_backend="custom"` when getrandom
refuses the target. Copy it into a workspace of one `#![no_std]` rlib per
library. The 2026-11-12 compile-only results are in
[COMPETITOR-BUILDS.md](COMPETITOR-BUILDS.md). Upstream rind, with no
patches, fails on every target there. RustCrypto and dalek crates pass.
ring fails in its ARM x25519 assembly, or for lack of a C compiler.
aws-lc-rs fails on pthreads and `clock_gettime`.

## Constant time

The table and sources are in [TARGET-SURVEY.md](TARGET-SURVEY.md).

- M0, M0+, and M23: `muls` is fixed-time. A 32×32→64 product goes through
  compiler-builtins `__aeabi_lmul`, branch-free on rustc 1.99.1. libgcc's
  routine of the same name branches. In rustc 1.98.1 RP2040 bench images
  the helper was measured at 74 bytes, 37 Thumb instructions, and 6
  `muls`: a general 64×64→64 (low half) routine, not a 32×32
  specialization, so a hot path should still replace it. See
  [rind-armv6m-perf](../rind-armv6m-perf/SKILL.md) and
  [rind-armv6m-constant-time](../rind-armv6m-constant-time/SKILL.md).
- M3: `umull` and `umlal` finish early. Not constant time.
- M4 and M7: `umull` is single-cycle and fixed.
- M33, including EFR32MG24: rustc 1.99.1 emits one `umull` for a
  u32×u32→u64 product. The M33 TRM documents early termination for the
  divider, not for `umull`. Measure with a cycle counter before calling it
  constant time.
- riscv32 without M: software multiply loops exit early.
- Hazard3: constant time only with `MUL_FAST` and `MULH_FAST`.
- ESP32-C3, ESP32-C6, ESP32-H2, and Xtensa: multiply timing is
  undocumented.

## CI and benches

Compile-check the library, with and without `wipe`, and with the getrandom
custom cfg:

1. `thumbv6m-none-eabi` (RP2040 and other M0/M0+).
2. `thumbv7em-none-eabihf` (M4/M7).
3. `thumbv8m.main-none-eabihf` (M33: EFR32MG24, RP2350).
4. `riscv32imc-unknown-none-elf` (ESP32-C3, no CAS).
5. `riscv32imac-unknown-none-elf` (ESP32-C6, ESP32-H2, RP2350 Hazard3).

The compile matrix did not flash anything. Firmware that has been
flashed lives in rind-hardware: ESP32-S3 and nRF52840. RP2040, RP2350
(both cores), ESP32-C3, a Cortex-M4F part, and EFR32MG24 (software
against the secure engine) are still worth a timing run once the image
builds. Ask before that flash. An EFR32MG24 image is not a license to
erase the bootloader. Timing the RP2040 harness is
[rind-bench-rp2040](../rind-bench-rp2040/SKILL.md).

## Demo screen

Optional, display only. An SSD1306 at I2C `0x3C` (I2C1, PC4 SDA, PC5 SCL)
cycles three 128×64 frames. Logos are not in this repo. See
[oled-demo/README.md](oled-demo/README.md). Flash the application region
only, after the backup hashes match.
