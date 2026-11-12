# What builds for thumbv6m-none-eabi (RP2040) and nearby bare-metal targets

Run on 2026-11-12, rustc 1.99.1 (stable), on an x86_64 Linux host, with no device involved. Each library is built as a `#![no_std]` rlib in its own crate (`crates/tv-*`) by [`build-matrix.sh`](build-matrix.sh):

```
cargo build --release --target <triple> -p <crate>
```

If the first attempt fails in getrandom 0.4, `build-matrix.sh` retries with `RUSTFLAGS='--cfg getrandom_backend="custom"'`. C toolchains for the C-backed crates came from nixpkgs: Arm GNU Toolchain `arm-none-eabi-gcc` 15.3.1 for the Arm targets, and `clang` 21.1.8 (bare, with no libc headers) for riscv32.

What this checks is compilation only: no link, and nothing run. Most crates just re-export the library (`pub use`), so its generic code isn't monomorphized. The ring, sha2, and curve25519-dalek crates each call one function.

Versions are those `rind-bench` uses (`rind-bench/Cargo.toml` at a4759c48), with default features off.

| Library (version, features) | thumbv6m (M0+) | thumbv7em (M4/M7) | thumbv8m.main (M33: EFR32MG24) | riscv32imc | riscv32imac | getrandom backend / CAS |
|---|---|---|---|---|---|---|
| rind 0.3.2 (none) | no | no | no | no | no | custom cfg needed first; then `can't find crate for std` |
| rind 0.3.2's portable-backend dependencies¹ | yes | yes | yes | yes | yes | custom cfg (getrandom 0.4) |
| ring 0.17.14 (none) | no | no | no | no | no | needs getrandom 0.2 `custom` feature, then a C toolchain; see below |
| aws-lc-rs 1.19.0 (`aws-lc-sys`) | no | no | no | no | no | C build expects an OS (pthreads, `clock_gettime`) |
| rustls 0.23.46 (no provider) | no | yes | yes | no | yes | needs CAS and atomic `Arc` |
| rustls 0.23.46 + `ring` | no | no | no | no | no | fails in ring |
| rustls 0.23.46 + `aws-lc-rs` | no | no | no | no | no | fails in aws-lc-sys or CAS |
| rustls-rustcrypto 0.1.0-pre.0 (`alloc`, `tls12`) | no | yes | yes | no | yes | custom cfg; needs CAS and atomic `Arc` |
| sha2 0.11.1 | yes | yes | yes | yes | yes | none |
| hmac 0.13.0 + sha2 | yes | yes | yes | yes | yes | none |
| aes-gcm 0.11.1 (`aes`) | yes | yes | yes | yes | yes | none |
| chacha20poly1305 0.11.0 | yes | yes | yes | yes | yes | none |
| p256 0.14.0 (`arithmetic`, `ecdsa`) | yes | yes | yes | yes | yes | none |
| curve25519-dalek 5.0.0 (`precomputed-tables`) | yes | yes | yes | yes | yes | none |
| x25519-dalek 3.0.0 | yes | yes | yes | yes | yes | none |
| ed25519-dalek 3.0.0 | yes | yes | yes | yes | yes | none |

¹ curve25519-dalek 5 (`precomputed-tables`), fiat-crypto 0.3, libcrux-hacl-rs 0.0.6 (`alloc`), libcrux-poly1305 0.0.7, libcrux-sha2 0.0.9, libcrux-macros 0.0.4, getrandom 0.4. These also pass on thumbv7m, thumbv8m.base, and riscv32i. rind 0.3.2 itself fails on all three with the same `std` error.

## First errors

- **rind 0.3.2**: without the cfg, getrandom 0.4.3 fails with `error: target is not supported. You may need to define a custom backend`. With it, rind fails with `error[E0463]: can't find crate for std`.
- **ring 0.17.14**:
  - **No cross C compiler:** `ToolNotFound: failed to find tool "arm-none-eabi-gcc"`.
  - **No getrandom 0.2 `custom` feature:** `error: target is not supported, for more information see: https://docs.rs/getrandom/#unsupported-targets`.
  - **thumbv6m, with both:** the assembler rejects `crypto/curve25519/asm/x25519-asm-arm.S`: `selected processor does not support 'vpush {q4,q5,q6,q7}' in Thumb mode`.
  - **thumbv7em / thumbv8m.main:** the same file fails with `r13 not allowed here -- 'and sp,sp,#0xffffffe0'`.
  - **riscv32 with bare clang:** `include/ring-core/check.h:27:11: fatal error: 'assert.h' file not found`. A riscv32 newlib toolchain wasn't tried.
- **aws-lc-rs 1.19.0 / aws-lc-sys 0.47.0**:
  - **No cross C compiler:** `ToolNotFound: failed to find tool "arm-none-eabi-gcc"`.
  - **Arm, with the C compiler:** `include/openssl/thread.h:31:9: error: unknown type name 'pthread_rwlock_t'`, and jitterentropy `implicit declaration of function 'clock_gettime'`.
  - **riscv32 with bare clang:** `'stdlib.h' file not found`.
- **rustls 0.23.46, rustls-rustcrypto** (no-CAS targets thumbv6m and riscv32imc):
  - **First:** `error[E0599]: no method named 'compare_exchange' found for struct 'Atomic<T>'`, in once_cell 1.21.4's `race` module.
  - **With once_cell's `portable-atomic` feature and portable-atomic's `critical-section`:** `error[E0433]: cannot find 'sync' in 'alloc'` at `rustls-0.23.46/src/lib.rs:425`. rustls's own comment there says forks for targets without atomic pointers swap in `portable_atomic_util::Arc`.
  - **Without `alloc`:** rustls-rustcrypto fails with `error: Rustls currently does not support alloc-less environments`.

## Notes

- The RustCrypto crates and the dalek crates need no randomness backend and no CAS at these features. Their RNG-using APIs take a caller-supplied `rand_core` RNG.
- On thumbv6m, Rust's compiler-builtins lowers a 32×32→64 product to `__aeabi_lmul`. Disassembled on this toolchain, it's four 16×16 `muls` plus `adds`/`adcs`, with no branch. BearSSL's table notes that libgcc's routine of the same name branches; that matters for C code linked against libgcc. See [TARGET-SURVEY.md](TARGET-SURVEY.md).
- **EFR32MG24 (thumbv8m.main-none-eabihf):** the column above is this chip's target. Re-checked against the kept workspace's `logs/thumbv8m.main-none-eabihf/` on 2026-11-12: same results. The M33 has CAS (`target_has_atomic` 8/16/32/ptr), so rustls with no provider and rustls-rustcrypto build where thumbv6m fails. rind still fails only on `std`; ring fails in its Armv7 x25519 assembly; aws-lc-sys fails on pthreads/`clock_gettime`. Nothing here was run on the chip: compile only.
