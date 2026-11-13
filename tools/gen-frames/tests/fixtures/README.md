# gen-frames test fixtures

`assets/` holds the two SVGs named in `oled-demo/README.md`, unmodified:
the moq.dev mark from https://moq.dev/layout/favicon.svg and the rind
wordmark from moq-dev/rind `admin/wordmark-light.svg`.

`fonts/` holds DejaVu Sans and DejaVu Sans Bold 2.37 (see `fonts/LICENSE`)
so `cargo test` does not depend on a system font path.

`expected/` holds `frames.h` and the preview PNGs produced by the original
`gen_frames.py` (rind-agent `main`) with those fonts and assets. Pillow
picks its RAQM layout whenever libraqm is installed, as on Debian; the
output is the same with Pillow 10.4, 11.0, 11.3 and 12.0 with libraqm and
with Debian's Pillow 11.1. Without libraqm Pillow falls back to BASIC
layout (hinted advances, no GPOS kerning) and the text lands 1 to 2 pixels
off, so regenerate with libraqm present. The only intentional change in
`frames.h` is the generator comment, which now names `tools/gen-frames`.
