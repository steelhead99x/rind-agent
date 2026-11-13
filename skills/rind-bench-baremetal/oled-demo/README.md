# EFR32MG24 SSD1306 frames

Three 128×64 screens, 4 s each, I2C `0x3C` (I2C1, PC4 SDA, PC5 SCL):

1. The moq.dev mark, from <https://moq.dev/layout/favicon.svg>.
2. The rind wordmark, `admin/wordmark-light.svg` in
   [moq-dev/rind](https://github.com/moq-dev/rind/blob/main/admin/wordmark-light.svg).
3. One TLS handshake line from a single run at rind `b6651423`. Replace the
   numbers in `frame_bench` (rind-results
   `tools/xtask/src/gen_frames/mod.rs`) before reuse.

The frames come from `cargo xtask gen-frames` in a
[rind-results](https://github.com/steelhead99x/rind-results) checkout, which is
all Rust. Run it there and pass this directory as an absolute path:

```
cargo xtask gen-frames --dir <absolute path to this oled-demo directory>
```

It fetches the two SVGs, unmodified, into `assets/` on first run, then writes
`oled_demo/frames.h` and preview PNGs. Do not commit those outputs or the
SVGs.

`oled_demo/oled_demo.ino` probes `0x3C` then `0x3D` and prints one JSON line
at 115200 baud. Compile with the core you use for this chip. Flash only the
application region with `probe-rs download --verify`, after the backup in
[the parent skill](../SKILL.md) matches. Do not erase or unlock.
