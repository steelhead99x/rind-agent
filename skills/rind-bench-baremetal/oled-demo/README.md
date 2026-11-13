# EFR32MG24 SSD1306 frames

Three 128×64 screens, 4 s each, I2C `0x3C` (I2C1, PC4 SDA, PC5 SCL):

1. The moq.dev mark, from <https://moq.dev/layout/favicon.svg>.
2. The rind wordmark, `admin/wordmark-light.svg` in
   [moq-dev/rind](https://github.com/moq-dev/rind/blob/main/admin/wordmark-light.svg).
3. One TLS handshake line from a single run at rind `b6651423`. Replace the
   numbers in the `gen-frames` tool (`tools/gen-frames/`) before reuse.

Save the SVGs, unmodified, as `assets/moqdev-layout_favicon.svg` and
`assets/wordmark-light.svg`. Then, from the repository root:

```shell
cargo run -p gen-frames --locked -- skills/rind-bench-baremetal/oled-demo
```

Pass `--font-dir` if DejaVu Sans is not at `/usr/share/fonts/truetype/dejavu`.
The tool writes `oled_demo/frames.h` and preview PNGs. Do not commit those
outputs.

`oled_demo/oled_demo.ino` probes `0x3C` then `0x3D` and prints one JSON line
at 115200 baud. Compile with the core you use for this chip. Flash only the
application region with `probe-rs download --verify`, after the backup in
[the parent skill](../SKILL.md) matches. Do not erase or unlock.
