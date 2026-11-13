// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

//! Pillow 11 `Image.BOX` downsample: two-pass, the C `box_filter`, and
//! 8-bit fixed-point rounding from `Resample.c`.

const PRECISION_BITS: i32 = 32 - 8 - 2;

fn box_filter(x: f64) -> f64 {
    if x > -0.5 && x <= 0.5 { 1.0 } else { 0.0 }
}

fn clip8(ss0: i32) -> u8 {
    let v = ss0 >> PRECISION_BITS;
    if v < 0 {
        0
    } else if v > 255 {
        255
    } else {
        v as u8
    }
}

fn precompute_coeffs(in_size: usize, out_size: usize) -> (Vec<(usize, usize)>, Vec<Vec<i32>>) {
    let scale = in_size as f64 / out_size as f64;
    let mut filterscale = scale;
    if filterscale < 1.0 {
        filterscale = 1.0;
    }
    let support = 0.5 * filterscale;
    let ksize = (support.ceil() as usize) * 2 + 1;
    let ss = 1.0 / filterscale;
    let mut bounds = Vec::with_capacity(out_size);
    let mut kk = Vec::with_capacity(out_size);
    for xx in 0..out_size {
        let center = (xx as f64 + 0.5) * scale;
        let mut xmin = (center - support + 0.5) as i32;
        if xmin < 0 {
            xmin = 0;
        }
        let mut xmax = (center + support + 0.5) as i32;
        if xmax > in_size as i32 {
            xmax = in_size as i32;
        }
        xmax -= xmin;
        let mut k = vec![0.0_f64; ksize];
        let mut ww = 0.0;
        for x in 0..xmax {
            let w = box_filter((x as f64 + xmin as f64 - center + 0.5) * ss);
            k[x as usize] = w;
            ww += w;
        }
        if ww != 0.0 {
            for slot in k.iter_mut() {
                *slot /= ww;
            }
        }
        let kq: Vec<i32> = k
            .into_iter()
            .map(|w| {
                let scaled = w * (1_i32 << PRECISION_BITS) as f64;
                if w < 0.0 {
                    (-0.5 + scaled) as i32
                } else {
                    (0.5 + scaled) as i32
                }
            })
            .collect();
        bounds.push((xmin as usize, xmax as usize));
        kk.push(kq);
    }
    (bounds, kk)
}

/// Resize an 8-bit greyscale buffer with Pillow's BOX filter.
pub fn box_resize(src: &[u8], sw: usize, sh: usize, dw: usize, dh: usize) -> Vec<u8> {
    assert_eq!(src.len(), sw * sh);
    let mut tmp = src.to_vec();
    let mut tw = sw;
    let mut th = sh;
    if dw != sw {
        let (bounds, kk) = precompute_coeffs(sw, dw);
        let mut out = vec![0u8; dw * th];
        for yy in 0..th {
            for xx in 0..dw {
                let (xmin, xmax) = bounds[xx];
                let mut ss0 = 1 << (PRECISION_BITS - 1);
                for x in 0..xmax {
                    ss0 += tmp[yy * tw + (x + xmin)] as i32 * kk[xx][x];
                }
                out[yy * dw + xx] = clip8(ss0);
            }
        }
        tmp = out;
        tw = dw;
    }
    if dh != sh {
        let (bounds, kk) = precompute_coeffs(sh, dh);
        let mut out = vec![0u8; tw * dh];
        for yy in 0..dh {
            let (ymin, ymax) = bounds[yy];
            for xx in 0..tw {
                let mut ss0 = 1 << (PRECISION_BITS - 1);
                for y in 0..ymax {
                    ss0 += tmp[(y + ymin) * tw + xx] as i32 * kk[yy][y];
                }
                out[yy * tw + xx] = clip8(ss0);
            }
        }
        tmp = out;
        th = dh;
    }
    debug_assert_eq!(tw, dw);
    debug_assert_eq!(th, dh);
    tmp
}

/// Threshold a greyscale buffer: `255` if `v >= thr`, else `0`.
pub fn threshold(src: &[u8], thr: u8) -> Vec<u8> {
    src.iter()
        .map(|&v| if v >= thr { 255 } else { 0 })
        .collect()
}
