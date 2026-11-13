// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

//! SVG to RGBA via librsvg + cairo, matching the cairosvg pipeline closely
//! enough that the 1-bit + BOX downsample matches CPython's output.

use std::fs;
use std::os::raw::{c_double, c_int, c_void};
use std::path::Path;
use std::ptr;

pub struct RgbaImage {
    pub width: u32,
    pub height: u32,
    /// Straight (non-premultiplied) RGBA, 4 bytes per pixel, row-major.
    pub data: Vec<u8>,
}

#[repr(C)]
struct RsvgRectangle {
    x: c_double,
    y: c_double,
    width: c_double,
    height: c_double,
}

// Bind only the C entry points we need. Symbols come from pkg-config in build.rs.
extern "C" {
    fn rsvg_handle_new_from_data(
        data: *const u8,
        data_len: usize,
        error: *mut *mut c_void,
    ) -> *mut c_void;
    fn rsvg_handle_set_dpi(handle: *mut c_void, dpi: c_double);
    fn rsvg_handle_get_intrinsic_size_in_pixels(
        handle: *mut c_void,
        out_width: *mut c_double,
        out_height: *mut c_double,
    ) -> c_int;
    fn rsvg_handle_render_document(
        handle: *mut c_void,
        cr: *mut c_void,
        viewport: *const RsvgRectangle,
        error: *mut *mut c_void,
    ) -> c_int;
    fn cairo_scale(cr: *mut c_void, sx: c_double, sy: c_double);
    fn cairo_image_surface_create(format: c_int, width: c_int, height: c_int) -> *mut c_void;
    fn cairo_create(target: *mut c_void) -> *mut c_void;
    fn cairo_destroy(cr: *mut c_void);
    fn cairo_surface_destroy(surface: *mut c_void);
    fn cairo_surface_flush(surface: *mut c_void);
    fn cairo_image_surface_get_data(surface: *mut c_void) -> *mut u8;
    fn cairo_image_surface_get_stride(surface: *mut c_void) -> c_int;
    fn cairo_surface_status(surface: *mut c_void) -> c_int;
    fn g_object_unref(object: *mut c_void);
}

const CAIRO_FORMAT_ARGB32: c_int = 0;

/// Render `path` to an RGBA image. `px_w` and `px_h` match cairosvg's
/// `output_width` and `output_height`: a missing height keeps aspect ratio.
pub fn svg_rgba(path: &Path, px_w: u32, px_h: Option<u32>) -> Result<RgbaImage, String> {
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let out_w = px_w;
    let out_h = match px_h {
        Some(h) => h,
        None => intrinsic_height_for_width(path, &bytes, px_w)?,
    };

    unsafe { render_with_rsvg(path, &bytes, out_w, out_h) }
}

fn intrinsic_height_for_width(path: &Path, _bytes: &[u8], px_w: u32) -> Result<u32, String> {
    // cairosvg: scale width to px_w and keep aspect ratio. For the rind
    // wordmark that is 1000x804; read the SVG viewBox so we do not have to
    // query librsvg for the intrinsic size.
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let (vw, vh) = viewbox_size(&text)
        .ok_or_else(|| format!("{}: missing viewBox or width/height", path.display()))?;
    if vw <= 0.0 || vh <= 0.0 {
        return Err(format!("{}: empty viewBox", path.display()));
    }
    let h = (vh * (px_w as f64 / vw)).round();
    Ok(if h < 1.0 { 1 } else { h as u32 })
}

fn viewbox_size(svg: &str) -> Option<(f64, f64)> {
    let key = "viewBox=\"";
    let start = svg.find(key)? + key.len();
    let end = svg[start..].find('"')? + start;
    let parts: Vec<&str> = svg[start..end].split_whitespace().collect();
    if parts.len() != 4 {
        return None;
    }
    let w = parts[2].parse().ok()?;
    let h = parts[3].parse().ok()?;
    Some((w, h))
}

unsafe fn render_with_rsvg(
    path: &Path,
    bytes: &[u8],
    out_w: u32,
    out_h: u32,
) -> Result<RgbaImage, String> {
    let mut error = ptr::null_mut();
    let handle = rsvg_handle_new_from_data(bytes.as_ptr(), bytes.len(), &mut error);
    if handle.is_null() {
        return Err(format!("{}: librsvg failed to parse", path.display()));
    }
    let surface = cairo_image_surface_create(CAIRO_FORMAT_ARGB32, out_w as c_int, out_h as c_int);
    if surface.is_null() || cairo_surface_status(surface) != 0 {
        g_object_unref(handle);
        return Err(format!("{}: cannot allocate cairo surface", path.display()));
    }
    let cr = cairo_create(surface);
    if cr.is_null() {
        cairo_surface_destroy(surface);
        g_object_unref(handle);
        return Err(format!("{}: cannot create cairo context", path.display()));
    }
    rsvg_handle_set_dpi(handle, 96.0);
    let mut iw = 0.0;
    let mut ih = 0.0;
    let got_intrinsic = rsvg_handle_get_intrinsic_size_in_pixels(handle, &mut iw, &mut ih) != 0
        && iw > 0.0
        && ih > 0.0;
    let viewport = if got_intrinsic && (iw - f64::from(out_w)).abs() > 0.01
        || got_intrinsic && (ih - f64::from(out_h)).abs() > 0.01
    {
        // Width-only sizing (cairosvg output_height=None): scale uniformly
        // from the intrinsic size so the aspect ratio is preserved.
        let sx = f64::from(out_w) / iw;
        // Prefer uniform scale when the requested height is the rounded
        // intrinsic height at that width.
        let uniform = (ih * sx).round() as u32 == out_h;
        if uniform {
            cairo_scale(cr, sx, sx);
            RsvgRectangle {
                x: 0.0,
                y: 0.0,
                width: iw,
                height: ih,
            }
        } else {
            RsvgRectangle {
                x: 0.0,
                y: 0.0,
                width: f64::from(out_w),
                height: f64::from(out_h),
            }
        }
    } else {
        RsvgRectangle {
            x: 0.0,
            y: 0.0,
            width: f64::from(out_w),
            height: f64::from(out_h),
        }
    };
    error = ptr::null_mut();
    let ok = rsvg_handle_render_document(handle, cr, &viewport, &mut error);
    cairo_surface_flush(surface);
    let result = if ok == 0 {
        Err(format!("{}: librsvg render failed", path.display()))
    } else {
        let stride = cairo_image_surface_get_stride(surface) as usize;
        let src = cairo_image_surface_get_data(surface);
        if src.is_null() {
            Err(format!("{}: cairo surface has no data", path.display()))
        } else {
            Ok(argb32_to_straight_rgba(
                std::slice::from_raw_parts(src, stride * out_h as usize),
                stride,
                out_w,
                out_h,
            ))
        }
    };
    cairo_destroy(cr);
    cairo_surface_destroy(surface);
    g_object_unref(handle);
    result
}

/// Cairo `ARGB32` is native-endian premultiplied; on little-endian that is BGRA.
fn argb32_to_straight_rgba(src: &[u8], stride: usize, width: u32, height: u32) -> RgbaImage {
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height as usize {
        let row = &src[y * stride..];
        for x in 0..width as usize {
            let i = x * 4;
            let b = row[i];
            let g = row[i + 1];
            let r = row[i + 2];
            let a = row[i + 3];
            if a == 0 {
                data.extend_from_slice(&[0, 0, 0, 0]);
            } else if a == 255 {
                data.extend_from_slice(&[r, g, b, a]);
            } else {
                let unpremul = |c: u8| ((c as u32 * 255 + (a as u32 / 2)) / a as u32) as u8;
                data.extend_from_slice(&[unpremul(r), unpremul(g), unpremul(b), a]);
            }
        }
    }
    RgbaImage {
        width,
        height,
        data,
    }
}

/// 1-bit mask (`0` or `255`) from an RGBA image and a predicate `lit(r,g,b,a)`.
pub fn to_bits(img: &RgbaImage, lit: impl Fn(u8, u8, u8, u8) -> bool) -> (Vec<u8>, u32, u32) {
    let mut out = vec![0u8; (img.width * img.height) as usize];
    for (i, px) in img.data.chunks_exact(4).enumerate() {
        if lit(px[0], px[1], px[2], px[3]) {
            out[i] = 255;
        }
    }
    (out, img.width, img.height)
}

// Silence unused on big-endian? We assume little-endian host (macOS/Linux CI).
const _: () = assert!(cfg!(target_endian = "little"));
