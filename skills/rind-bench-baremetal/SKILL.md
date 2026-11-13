---
name: rind-bench-baremetal
description: Use when building or flashing rind for a microcontroller, or backing up an RP2040 or an EFR32MG24 before any write.
---

# Microcontrollers

rind has no `#![no_std]`. At `a4759c48` with rustc 1.99.1,
`cargo build -p rind --release --target thumbv6m-none-eabi` needs
`RUSTFLAGS='--cfg getrandom_backend="custom"'`, then fails with
`can't find crate for std` (1672 errors on that build).
`std::thread_local!` is in `rind/src/low/cpu.rs` and `rind/src/low/entry.rs`.
No `no_std` quest is in the tree. The portable-backend dependencies do
compile for that target. Do not flash a rind binary.

## Back up first

Read the flash at least twice. The sha256 values must match. Keep the
image out of git. Do not erase. Do not unlock debug.

RP2040 (Cortex-M0+, 2 MiB at `0x10000000` through `0x10200000`, 264 KiB
RAM): open the USB serial at 1200 baud to enter the bootrom, then
`picotool save -r 0x10000000 0x10200000 backup.bin`. Save a second read.
Restore by writing that image back and checking the readback sha256.

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

- rind-hardware's `efr32mg24/` port runs at 78 MHz: HFXO (39 MHz), DEVINFO's
  HFRCODPLL calibration for the 80 MHz band, DPLL0 at HFXO x 2000 / 1000,
  one flash wait state, PCLK = HCLK / 2.
- A debugger reset keeps the CMU as the last image left it. Switch SYSCLK
  back to FSRCO and stop DPLL0 before reprogramming the clocks.
- Series 2 interrupt flags clear only through the register's clear alias
  (`+0x2000`). A plain write leaves them set, so a wait on the flag passes at
  once and reads a stale result.
- Writing `CMU_CALCTRL` while a calibration runs is a precise BusFault. Wait
  for `CMU_STATUS.CALRDY` first. The up count must fit CALCNT's 20 bits.
- Check the clock against HFXO with the calibration counter (HCLK as the
  down clock) and put the result in the start record.
- Output over SEGGER RTT through the on-board CMSIS-DAP needs no UART pins.
  Drain the buffer before each benchmark.
- `probe-rs run` programs from `0x08000000`, over the bootloader. Restore the
  full backup afterwards.
- ring's Thumb-2 patch for `thumbv7em` also builds for `thumbv8m.main`.
- After a restore, the original app can boot before the readback and append
  NVM3 records to a page that was erased in the backup. A readback that
  differs only there (backup bytes all `0xFF`) is a good restore, provided
  `probe-rs download --verify` passed. Diff the two images to confirm.

### ESP32-S3 on macOS

- If esptool or espflash reports "No serial data received" on the
  USB-Serial/JTAG port, run `probe-rs reset --chip esp32s3` on the same
  USB-JTAG probe first. Connections work after that.
- On a fresh rind-hardware checkout, build `esp-idf-sys` first. Otherwise
  ring's C build can run before embuild has installed the Xtensa GCC.
- `rustfmt +stable` auto-installs a missing toolchain. Use the pinned one.

### Results provenance

- Images built from rind-hardware itself carry only `rind_hardware` in
  `images.json`. A `rind_bench` field makes the report map the SHA through
  the legacy table and print "ancestor of" the squash root.
- To get the gap list for a new board, run `cargo xtask report` with a
  header-only `gaps.toml`. It names each gap over 10% that needs a decision.

### Shell

- The Mac's login shell is zsh, which does not word-split `$VAR`. Use a
  function or list the arguments when a variable holds several.
- Write scratch output inside your lane directory, never the shared work
  root.

### Several boards, one lock

Board cycle counters are not disturbed by host load. One agent may time
several of its own boards in parallel under a single lock hold. Take the
lock once per bench file and release it between files so other lanes get
turns.

## What compiles

[build-matrix.sh](build-matrix.sh) release-builds each `crates/tv-*`
member and retries with `getrandom_backend="custom"` when getrandom
refuses the target. Copy it into a workspace of one `#![no_std]` rlib per
library. The compile-only results are in
[COMPETITOR-BUILDS.md](COMPETITOR-BUILDS.md). rind fails on every target
there. RustCrypto and dalek crates pass. ring fails in its ARM x25519
assembly, or for lack of a C compiler. aws-lc-rs fails on pthreads and
`clock_gettime`.

## Constant time

The table and sources are in [TARGET-SURVEY.md](TARGET-SURVEY.md).

- M0, M0+, and M23: `muls` is fixed-time. A 32×32→64 product goes through
  compiler-builtins `__aeabi_lmul`, branch-free on rustc 1.99.1. libgcc's
  routine of the same name branches.
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

Chips worth timing once rind builds: RP2040, RP2350 (both cores),
ESP32-C3, a Cortex-M4F part, and EFR32MG24 (software against the secure
engine). Nothing rind-related was flashed.

## Demo screen

Optional, display only. An SSD1306 at I2C `0x3C` (I2C1, PC4 SDA, PC5 SCL)
cycles three 128×64 frames. Logos are not in this repo. See
[oled-demo/README.md](oled-demo/README.md). Flash the application region
only, after the backup hashes match.
