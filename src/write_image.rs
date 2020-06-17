use gleam::gl::{GLenum, RGBA, UNSIGNED_BYTE};

use std::fs::File;
use std::path::Path;
use image::ColorType;
use image::png::PNGEncoder;

pub fn write_image<P: AsRef<Path>>(path: P, data: &[u8], width: u32, height: u32,
                                   format: GLenum, pixel_type: GLenum) {
    let color_type = match (format, pixel_type) {
        (RGBA, UNSIGNED_BYTE) => ColorType::Rgba8,
        _ => panic!("gl-replay: write_image: unsupported format/pixel type combination: 0x{:x}, 0x{:x}",
                    format, pixel_type),
    };

    let file = File::create(path)
        .expect("gl-replay: write_image: error creating file");
    let encoder = PNGEncoder::new(file);
    encoder.encode(data, width, height, color_type)
        .expect("gl-replay: write_image: error writing file");
}

