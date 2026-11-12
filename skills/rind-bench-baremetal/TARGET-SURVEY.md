# Microcontroller targets that share the RP2040's constraints for rind

Surveyed on 2026-11-12. The constraints rind's no_std quest has to handle:

- **No std / no OS:** every target here.
- **No CAS:** no compare-and-swap, so `compare_exchange`/`fetch_max` are missing and so is `alloc::sync::Arc`.
- **Multiplier:** no 32×32→64 multiply, or a multiply whose time depends on its operands.
- **Little RAM.**
- **No hardware RNG:** getrandom needs a board-supplied source.

CAS comes from `rustc --print cfg --target <triple>` (rustc 1.99.1): a target without `target_has_atomic="32"` has no CAS. The compile column is [COMPETITOR-BUILDS.md](COMPETITOR-BUILDS.md), for rind 0.3.2's portable-backend dependencies (`tv-rind-portable-deps`) and for rind itself.

| Family (chips) | Rust target | CAS | 32×32→64 multiply, timing | Crypto accelerators, RNG | SRAM | rind blockers | Compile check |
|---|---|---|---|---|---|---|---|
| Cortex-M0/M0+ (RP2040, STM32G0, STM32L0, STM32F0, nRF51, SAMD21) | `thumbv6m-none-eabi` | No | None in hardware: `muls` is 32×32→32, 1 or 32 cycles by the vendor's implementation option, fixed either way [Arm M0+ TRM][m0p], [BearSSL][ctmul]. Rust lowers 32×32→64 to compiler-builtins' `__aeabi_lmul`, which disassembles to four `muls` plus `adds`/`adcs` with no branch (checked on rustc 1.99.1). libgcc's routine of the same name branches [BearSSL][ctmul] | RP2040: none, no TRNG (ring-oscillator bit only) [RP2040][rp2040]. STM32G0/L0: AES and RNG on some parts. nRF51: AES-ECB, RNG. STM32F0, SAMD21: none | 4–264 KB (RP2040 264 KB) | std, CAS, getrandom backend; slow software 64-bit products; little RAM on most parts | deps yes, rind no (`std`) |
| Cortex-M23 (v8-M baseline) | `thumbv8m.base-none-eabi` | Yes | As M0+: no `umull` | vendor-specific | varies | std, getrandom | deps yes, rind no |
| Cortex-M3 (STM32F1/F2, LPC17xx) | `thumbv7m-none-eabi` | Yes | `umull`/`umlal` take 3–5 cycles, finishing early on small operands [Arm M3 TRM][m3]. Measured early exits on zero and power-of-two operands too [BearSSL][ctmul]: **not constant time** | vendor-specific | 8–128 KB typical | std, getrandom, variable-time multiply | deps yes, rind no |
| Cortex-M4/M7 (STM32F4, STM32F7, nRF52832/52840) | `thumbv7em-none-eabihf` | Yes | `umull` single cycle, fixed [Arm M4 TRM][m4], [BearSSL][ctmul] | STM32F4: RNG on most parts, CRYP/HASH on F415/417/437/439. nRF52840: CryptoCell-310 and RNG. nRF52832: AES-ECB/CCM and RNG [nRF52840][nrf52840] | 64–512 KB | std, getrandom | deps yes, rind no |
| Cortex-M33 (EFR32MG24, RP2350 Arm side, nRF5340, STM32U5/L5) | `thumbv8m.main-none-eabihf` | Yes | `umull`/`umlal` present (rustc 1.99.1 lowers a u32×u32→u64 product to one `umull`). Arm publishes no per-instruction cycle table for the M33; its TRM names operand-dependent early termination only for the divider [Arm M33 TRM][m33], unlike the M3 TRM, which states it for `umull`. Likely fixed-time, **not documented**: confirm with a DWT_CYCCNT sweep on the chip | EFR32MG24 (Secure Vault High): a separate Hardware Secure Engine with AES-128/192/256 (GCM, CCM, CTR…), ChaCha20-Poly1305, SHA-1/SHA-2 up to 512, ECDSA/ECDH P-192 to P-521, X25519, Ed25519, HKDF/PBKDF2, TRNG, DPA countermeasures [EFR32MG24][mg24]. RP2350: SHA-256 accelerator, TRNG [RP2350][rp2350]. nRF5340: CryptoCell-312 | 256–2500 KB (MG24 256 KB, RP2350 520 KB) | std, getrandom | deps yes, rind no |
| RP2350 RISC-V side (Hazard3) | `riscv32imac-unknown-none-elf` | Yes | `mul`/`mulhu` 1 cycle with Hazard3's `MUL_FAST` and `MULH_FAST`. Hazard3 claims Zkt constant time only in that configuration; the sequential `mulh`/`mulhsu` vary [Hazard3][hazard3]. Check RP2350's configuration in [its datasheet][rp2350] | SHA-256, TRNG (shared with the Arm side) | 520 KB | std, getrandom | deps yes, rind no |
| ESP32-C3 | `riscv32imc-unknown-none-elf` | **No** (no A extension) | `mulhu` present; timing not documented by Espressif [ESP32-C3][c3] | AES, SHA, RSA, HMAC, digital signature, RNG | 400 KB | std, CAS, getrandom; multiply timing unknown | deps yes, rind no |
| ESP32-C6, ESP32-H2 | `riscv32imac-unknown-none-elf` | Yes | timing not documented [ESP32-C6][c6], [ESP32-H2][h2] | AES, SHA, RSA, ECC, HMAC, digital signature (H2 adds ECDSA), RNG | C6 512 KB, H2 320 KB | std, getrandom; multiply timing unknown | deps yes, rind no |
| Other riscv32 with M, no A (CH32V003 is `rv32ec`; GD32VF103 is `rv32imac`) | `riscv32imc-unknown-none-elf` and others | No (imc) | implementation-defined | vendor-specific | 2–32 KB typical | as ESP32-C3, less RAM | deps yes (imc) |
| riscv32 without M | `riscv32i-unknown-none-elf` | No | Software only: compiler-builtins' `__mulsi3`/`__muldi3` are shift-and-add loops that exit when the operand runs out of bits (disassembled on rustc 1.99.1): **not constant time** | — | tiny | as above, plus a variable-time multiply | deps yes, rind no |
| ESP32, ESP32-S3 (Xtensa LX6/LX7) | `xtensa-esp32-none-elf`, `xtensa-esp32s3-none-elf` (tier 3, needs Espressif's toolchain via `espup`) | Yes (`s32c1i`) | timing not documented [ESP32][esp32], [ESP32-S3][s3] | AES, SHA, RSA, RNG (S3 adds HMAC, digital signature) | 520 KB / 512 KB | std, getrandom; out-of-tree toolchain | not compiled |
| MSP430 | `msp430-none-elf` (tier 3, nightly) | No | 16-bit; hardware multiplier only on some parts | some parts: AES | 0.5–66 KB | **out of scope**: 16-bit, too little RAM for RSA/ML-KEM | not compiled |
| AVR (ATmega) | `avr-none` (tier 3, nightly) | No | 8×8→16 only | none | 0.5–16 KB | **out of scope**: 8-bit, 16-bit pointers, too little RAM | not compiled |

CAS per `rustc --print cfg`: thumbv6m, riscv32i, riscv32imc, msp430, and avr report no `target_has_atomic`. The rest report 8/16/32/ptr.

## Suggestions for the quest

**Compile-check in CI** (library only, with and without `wipe`, with `--cfg getrandom_backend="custom"`):

1. `thumbv6m-none-eabi`: no CAS, no `umull` (RP2040, the STM32G0/L0 and nRF51 families).
2. `thumbv7em-none-eabihf`: the most common Cortex-M (M4/M7).
3. `thumbv8m.main-none-eabihf`: M33 (EFR32MG24, RP2350, nRF5340, STM32U5).
4. `riscv32imc-unknown-none-elf`: RISC-V without CAS (ESP32-C3).
5. `riscv32imac-unknown-none-elf`: RISC-V with CAS (ESP32-C6/H2, RP2350's Hazard3).

These five cover both CAS cases on both ISAs, and every hardware-multiply case except the M3's. Skip thumbv7m/thumbv8m.base (same code paths as 3 and 1) and Xtensa (out-of-tree toolchain).

**Constant-time caveats for PLATFORMS.md:**

- Cortex-M0/M0+/M23: `muls` is fixed-time in both implementation options. 32×32→64 products go through compiler-builtins' `__aeabi_lmul`, branch-free as of rustc 1.99.1. `admin/ct-vartime` should scan for it, because a C library linked against libgcc gets a branching routine of the same name.
- Cortex-M3: `umull`/`umlal` finish early on small operands (documented), and on zero and power-of-two operands (measured). The portable backend's 32-bit limb arithmetic is therefore not constant time there. This is the same class as the Cortex-A53 entry.
- Cortex-M33: `umull` timing is undocumented by Arm. The TRM lists early termination for the divider only. Treat it as unverified until a cycle counter on the chip measures it.
- riscv32 without M: software multiply loops exit early, so not constant time.
- riscv32 with M: `mulhu` timing is implementation-defined. Hazard3 is constant time only with `MUL_FAST` and `MULH_FAST`. Espressif documents no multiply timing for ESP32-C3/C6/H2 or Xtensa, so list them as unknown.
- Randomness: rind takes no position. The binary supplies getrandom's custom backend. The RP2040 has no TRNG, and its ring-oscillator bit is not a vetted source.

**Suggested benchmark chips:**

- **RP2040** (M0+, 125 MHz): the floor.
- **RP2350**: M33 and Hazard3 on one chip, the same memory, and a TRNG. It gives an Arm-vs-RISC-V comparison at equal clock.
- **EFR32MG24** (M33F, 78 MHz, 1536 KB flash, 256 KB RAM): with a CMSIS-DAP probe, after a verified full-flash backup. It's the one chip here where rind's software path can be timed against the SoC's own crypto engine (AES-GCM, ChaCha20-Poly1305, SHA-2, P-256, X25519, Ed25519).
- **ESP32-C3**: riscv32imc without CAS, the most common RISC-V MCU.
- **STM32F4 or nRF52840** (M4F): the most common Cortex-M class.

## Sources

[m0p]: https://developer.arm.com/documentation/ddi0484/latest "Arm Cortex-M0+ Technical Reference Manual (DDI 0484): MULS 1 or 32 cycles"
[m3]: https://developer.arm.com/documentation/ddi0337/latest "Arm Cortex-M3 Technical Reference Manual (DDI 0337): instruction timing, UMULL early termination"
[m4]: https://developer.arm.com/documentation/ddi0439/latest "Arm Cortex-M4 Technical Reference Manual (DDI 0439)"
[m33]: https://developer.arm.com/documentation/100230/latest "Arm Cortex-M33 Technical Reference Manual (100230)"
[ctmul]: https://www.bearssl.org/ctmul.html "BearSSL: Constant-Time Mul, per-CPU table"
[hazard3]: https://wren.wtf/hazard3/doc/ "Hazard3 Design Guide: 1.1.4 Multiply/Divide, 1.1.5 Constant-time Execution, A.2"
[rp2040]: https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf "RP2040 datasheet"
[mg24]: https://www.silabs.com/documents/public/data-sheets/efr32mg24-datasheet.pdf "EFR32MG24 data sheet, 3.8 Secure Vault features"
[rp2350]: https://datasheets.raspberrypi.com/rp2350/rp2350-datasheet.pdf "RP2350 datasheet"
[nrf52840]: https://docs.nordicsemi.com/bundle/ps_nrf52840/page/keyfeatures_html5.html "nRF52840 product specification"
[c3]: https://www.espressif.com/sites/default/files/documentation/esp32-c3_datasheet_en.pdf "ESP32-C3 datasheet"
[c6]: https://www.espressif.com/sites/default/files/documentation/esp32-c6_datasheet_en.pdf "ESP32-C6 datasheet"
[h2]: https://www.espressif.com/sites/default/files/documentation/esp32-h2_datasheet_en.pdf "ESP32-H2 datasheet"
[esp32]: https://www.espressif.com/sites/default/files/documentation/esp32_datasheet_en.pdf "ESP32 datasheet"
[s3]: https://www.espressif.com/sites/default/files/documentation/esp32-s3_datasheet_en.pdf "ESP32-S3 datasheet"

- Rust platform support (target tiers): https://doc.rust-lang.org/nightly/rustc/platform-support.html
- Wouter de Groot's Cortex-M3 multiply measurements, cited by BearSSL above.
