// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

//! RGB8 PNG writer and reader for preview images.

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub fn write_rgb(path: &Path, width: u32, height: u32, data: &[u8]) -> Result<(), String> {
    if data.len() != (width * height * 3) as usize {
        return Err(format!(
            "RGB buffer {} does not match {width}x{height}",
            data.len()
        ));
    }
    let file = File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Default);
    let mut writer = encoder
        .write_header()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    writer
        .write_image_data(data)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(())
}

pub fn read_rgb(path: &Path) -> Result<(u32, u32, Vec<u8>), String> {
    let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let decoder = png::Decoder::new(file);
    let mut reader = decoder
        .read_info()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    buf.truncate(info.buffer_size());
    let data = match info.color_type {
        png::ColorType::Rgb => buf,
        png::ColorType::Rgba => buf
            .chunks_exact(4)
            .flat_map(|p| [p[0], p[1], p[2]])
            .collect(),
        other => {
            return Err(format!(
                "{}: unsupported color type {other:?}",
                path.display()
            ));
        }
    };
    Ok((info.width, info.height, data))
}
