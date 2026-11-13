# gen-frames test fixtures

`assets/` holds the two SVGs named in `oled-demo/README.md`, unmodified:
the moq.dev mark from https://moq.dev/layout/favicon.svg and the rind
wordmark from moq-dev/rind `admin/wordmark-light.svg`.

`fonts/` holds DejaVu Sans and DejaVu Sans Bold 2.37 (see `fonts/LICENSE`)
so `cargo test` does not depend on a system font path.

`expected/` holds `frames.h` and the preview PNGs produced by the original
`gen_frames.py` with Pillow's BASIC layout (no libraqm) and those fonts.
The only intentional change in `frames.h` is the generator comment, which
now names `tools/gen-frames`.
