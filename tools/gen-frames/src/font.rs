// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

//! Monochrome text matching Pillow's FreeType driver in BASIC layout
//! (`_imagingft.c` in Pillow 11.3).

use std::path::{Path, PathBuf};

use freetype::bitmap::{Bitmap, PixelMode};
use freetype::face::{KerningMode, LoadFlag};
use freetype::{Face, Library};

/// Round a 26.6 pixel coordinate to the nearest integer (`PIXEL` in Pillow).
fn pixel(x: i32) -> i32 {
    ((x + 32) & -64) >> 6
}

fn frac(v: f64) -> f64 {
    v - v.trunc()
}

#[derive(Clone, Copy)]
pub enum FontKind {
    Sans,
    SansBold,
}

/// `true` when ImageDraw.fontmode is `"1"` (`FT_LOAD_TARGET_MONO`).
pub type MonoMode = bool;

struct GlyphInfo {
    index: u32,
    x_offset: i32,
    y_offset: i32,
    x_advance: i32,
    y_advance: i32,
}

pub struct Fonts {
    lib: Library,
    sans: PathBuf,
    bold: PathBuf,
}

impl Fonts {
    pub fn new(font_dir: &Path) -> Result<Self, String> {
        let lib = Library::init().map_err(|e| format!("freetype: {e}"))?;
        let sans = font_dir.join("DejaVuSans.ttf");
        let bold = font_dir.join("DejaVuSans-Bold.ttf");
        for path in [&sans, &bold] {
            if !path.is_file() {
                return Err(format!("missing font {}", path.display()));
            }
        }
        Ok(Self { lib, sans, bold })
    }

    fn path(&self, kind: FontKind) -> &Path {
        match kind {
            FontKind::Sans => &self.sans,
            FontKind::SansBold => &self.bold,
        }
    }

    fn face(&self, kind: FontKind, size: i32) -> Result<Face, String> {
        let face = self
            .lib
            .new_face(self.path(kind), 0)
            .map_err(|e| format!("{}: {e}", self.path(kind).display()))?;
        // Pillow: FT_Request_Size NOMINAL, width = height = size * 64,
        // resolution 0. At 0 dpi FreeType uses 72 dpi, so this matches
        // FT_Set_Char_Size.
        face.set_char_size((size as isize) * 64, (size as isize) * 64, 0, 0)
            .map_err(|e| format!("set_char_size: {e}"))?;
        Ok(face)
    }

    fn load_flags(mono: MonoMode, render: bool) -> LoadFlag {
        let mut flags = LoadFlag::DEFAULT;
        if mono {
            flags |= LoadFlag::TARGET_MONO;
        }
        if render {
            flags |= LoadFlag::RENDER;
        }
        flags
    }

    /// Pillow BASIC `getlength`, in pixels (sum of 26.6 advances / 64).
    pub fn text_length(
        &self,
        text: &str,
        kind: FontKind,
        size: i32,
        mono: MonoMode,
    ) -> Result<f64, String> {
        let face = self.face(kind, size)?;
        let glyphs = layout(&face, text, mono)?;
        let length: i32 = glyphs.iter().map(|g| g.x_advance).sum();
        Ok(length as f64 / 64.0)
    }

    /// Draw `text` with Pillow's default `"la"` anchor at `(x, y)`.
    ///
    /// `(x, y)` is the left edge and the typographic ascender line, as in
    /// `ImageDraw.text`. Fractional parts are Pillow's `start` offset.
    /// Drawing always uses `fontmode = "1"`, like `gen_frames.py`.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_text(
        &self,
        buf: &mut [u8],
        width: i32,
        height: i32,
        x: f64,
        y: f64,
        text: &str,
        kind: FontKind,
        size: i32,
    ) -> Result<(), String> {
        let mono = true;
        let face = self.face(kind, size)?;
        let glyphs = layout(&face, text, mono)?;
        if glyphs.is_empty() {
            return Ok(());
        }

        let measure = Self::load_flags(mono, false);
        let (x_min, y_max, y_anchor) = bounds_la(&face, &glyphs, measure)?;
        let paste_x = x.trunc() as i32 + x_min;
        let paste_y = y.trunc() as i32 + (y_anchor - y_max);
        let x_start = frac(x);
        let y_start = frac(y);

        let mut pen_x = ((-f64::from(x_min) + x_start) * 64.0).round() as i32;
        let mut pen_y = ((-f64::from(y_max) - y_start) * 64.0).round() as i32;

        let render = Self::load_flags(mono, true);
        for g in &glyphs {
            let px = pixel(pen_x + g.x_offset);
            let py = pixel(pen_y + g.y_offset);
            face.load_glyph(g.index, render)
                .map_err(|e| format!("load_glyph: {e}"))?;
            let slot = face.glyph();
            let bitmap = slot.bitmap();
            let xx = paste_x + px + slot.bitmap_left();
            let yy = paste_y - (py + slot.bitmap_top());
            blit_bitmap(buf, width, height, xx, yy, &bitmap);
            pen_x += g.x_advance;
            pen_y += g.y_advance;
        }
        Ok(())
    }
}

fn layout(face: &Face, text: &str, mono: MonoMode) -> Result<Vec<GlyphInfo>, String> {
    let mut flags = LoadFlag::DEFAULT;
    if mono {
        flags |= LoadFlag::TARGET_MONO;
    }
    let kerning = face.has_kerning();
    let mut last_index = 0u32;
    let mut glyphs: Vec<GlyphInfo> = Vec::new();
    for ch in text.chars() {
        let index = face.get_char_index(ch as usize).unwrap_or(0);
        face.load_glyph(index, flags)
            .map_err(|e| format!("load_glyph: {e}"))?;
        if kerning && last_index != 0 && index != 0 {
            if let Ok(delta) = face.get_kerning(last_index, index, KerningMode::KerningDefault) {
                if let Some(prev) = glyphs.last_mut() {
                    // Pillow adds PIXEL(delta) (an integer pixel) to a 26.6
                    // advance. Match that.
                    prev.x_advance += pixel(delta.x as i32);
                    prev.y_advance += pixel(delta.y as i32);
                }
            }
        }
        let metrics = face.glyph().metrics();
        glyphs.push(GlyphInfo {
            index,
            x_offset: 0,
            y_offset: 0,
            x_advance: metrics.horiAdvance as i32,
            y_advance: 0,
        });
        last_index = index;
    }
    Ok(glyphs)
}

fn bounds_la(face: &Face, glyphs: &[GlyphInfo], load: LoadFlag) -> Result<(i32, i32, i32), String> {
    let mut position = 0i32;
    let mut x_min = 0i32;
    let mut x_max = 0i32;
    let mut y_min = 0i32;
    let mut y_max = 0i32;
    for g in glyphs {
        let px = pixel(position + g.x_offset);
        let py = pixel(g.y_offset);
        position += g.x_advance;
        let advanced = pixel(position);
        if advanced > x_max {
            x_max = advanced;
        }
        face.load_glyph(g.index, load)
            .map_err(|e| format!("load_glyph: {e}"))?;
        let glyph = face
            .glyph()
            .get_glyph()
            .map_err(|e| format!("get_glyph: {e}"))?;
        let bbox = glyph.get_cbox(freetype::ffi::FT_GLYPH_BBOX_PIXELS);
        let x1 = bbox.xMax + i64::from(px);
        let x0 = bbox.xMin + i64::from(px);
        let y1 = bbox.yMax + i64::from(py);
        let y0 = bbox.yMin + i64::from(py);
        if x1 > i64::from(x_max) {
            x_max = x1 as i32;
        }
        if x0 < i64::from(x_min) {
            x_min = x0 as i32;
        }
        if y1 > i64::from(y_max) {
            y_max = y1 as i32;
        }
        if y0 < i64::from(y_min) {
            y_min = y0 as i32;
        }
    }
    let y_anchor = pixel(face.size_metrics().ok_or("missing size metrics")?.ascender as i32);
    let _ = (x_max, y_min);
    Ok((x_min, y_max, y_anchor))
}

fn blit_bitmap(buf: &mut [u8], width: i32, height: i32, xx: i32, yy: i32, bitmap: &Bitmap) {
    let rows = bitmap.rows();
    let cols = bitmap.width();
    if rows == 0 || cols == 0 {
        return;
    }
    let pitch = bitmap.pitch();
    let raw = bitmap.buffer();
    let mono = bitmap.pixel_mode().ok() == Some(PixelMode::Mono);
    for row in 0..rows {
        let dest_y = yy + row;
        if dest_y < 0 || dest_y >= height {
            continue;
        }
        let src_row = if pitch >= 0 {
            let start = (row * pitch) as usize;
            let end = start + pitch.unsigned_abs() as usize;
            if end > raw.len() {
                continue;
            }
            &raw[start..end]
        } else {
            let abs_pitch = (-pitch) as usize;
            let start = ((rows - 1 - row) * (-pitch)) as usize;
            let end = start + abs_pitch;
            if end > raw.len() {
                continue;
            }
            &raw[start..end]
        };
        for col in 0..cols {
            let dest_x = xx + col;
            if dest_x < 0 || dest_x >= width {
                continue;
            }
            let lit = if mono {
                let byte = src_row[col as usize / 8];
                (byte & (0x80 >> (col % 8))) != 0
            } else {
                // GRAY after convert, or unexpected 8-bit.
                let i = col as usize;
                i < src_row.len() && src_row[i] != 0
            };
            if lit {
                buf[(dest_y * width + dest_x) as usize] = 255;
            }
        }
    }
}
