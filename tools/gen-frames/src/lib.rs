// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

//! Build three 128x64 1-bit frames for an EFR32MG24 SSD1306 demo,
//! matching `oled-demo/tools/gen_frames.py`.

pub mod box_resize;
pub mod font;
pub mod generate;
pub mod png_rgb;
pub mod svg;

pub use generate::{Frame, H, W, generate};
