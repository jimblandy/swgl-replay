//! Serializing and deserializing blocks of pixels.
//!
//! This module's `Pixels` type represents a rectangular block of pixels in
//! memory (up to three dimensions), with an associated OpenGL format and pixel
//! type. It can either borrow or own the pixels.
//!
//! A `Pixels` value can be serialized and deserialized using the `var` module's
//! traits. Its serialized form is `PixelsForm`.

use gleam::gl;
use crate::{var, rle};
use std::borrow::Cow;
use std::{io, mem};

/// A deserialized block of pixels.
struct Pixels<'a> {
    /// Width of block, in pixels.
    pub width: usize,

    /// Height of the block, in pixels.
    pub height: usize,

    /// Depth of the block, in pixels.
    pub depth: usize,

    /// The format of the pixel data.
    ///
    /// This is interpreted the same way as the `format` argument to the OpenGL
    /// `glReadPixels` function, and must meet the same constraints.
    pub format: gl::GLenum,

    /// The type of the data.
    ///
    /// This is interpreted the same way as the `pixel_type` argument to the
    /// OpenGL `glReadPixels` function, and must meet the same constraints.
    pub pixel_type: gl::GLenum,

    /// The actual pixel content, as bytes.
    pub bytes: Cow<'a, [u8]>,
}

/// The serialization form for `Pixels`.
///
/// The serialized form of a `Pixels` value must be four-byte aligned. It starts
/// with `width`, `height`, `depth`, `format`, `pixel_type`, and the length of
/// `bytes` as unsigned LEB128 numbers, in that order, without padding. This is
/// followed by the pixel data itself.
///
/// The exact serialization form for the pixels depends on their `format` and
/// `pixel_type` values.
///
/// -   If each pixel is a single byte, then data is written as with
///     `rle::write_rle_u8`.
///
/// -   If each pixel is a four-byte value, the stream is padded to a four-byte
///     alignment boundary, and then written as by `rle::write_rle_u32`,
///
/// Other formats aren't yet supported, since we don't use them, but the `rle`
/// module has generic functions that should make it easy.
struct PixelsForm;

impl var::Serialize for Pixels<'_> {
    type Form = PixelsForm;
    fn serialize<S: var::MarkedWrite>(&self, stream: &mut S) -> io::Result<usize> {
        stream.align_for::<u32>()?;
        let mark = stream.mark();
        leb128::write::unsigned(stream, self.width as u64)?;
        leb128::write::unsigned(stream, self.height as u64)?;
        leb128::write::unsigned(stream, self.depth as u64)?;
        leb128::write::unsigned(stream, self.format as u64)?;
        leb128::write::unsigned(stream, self.pixel_type as u64)?;
        leb128::write::unsigned(stream, self.bytes.len() as u64)?;

        let bytes_per_pixel = gl::calculate_bytes_per_pixel(self.format, self.pixel_type);
        assert_eq!(bytes_per_pixel * self.width * self.height * self.depth,
                   self.bytes.len());

        match bytes_per_pixel {
            1 => {
                rle::write_u8(stream, &self.bytes)?;
            }
            4 => {
                stream.align_for::<u32>()?;
                assert!(self.bytes.len() % mem::align_of::<u32>() == 0);
                let slice = unsafe {
                    std::slice::from_raw_parts(self.bytes.as_ptr() as *const u32,
                                               self.bytes.len() / mem::size_of::<u32>())
                };
                rle::write_u32(stream, slice)?;
            }
            _ => todo!(),
        }

        Ok(mark)
    }
}

impl<'b> var::DeserializeAs<'b, Pixels<'static>> for PixelsForm {
    fn deserialize(buf: &mut &'b [u8]) -> Result<Pixels<'static>, var::DeserializeError> {
        let width = leb128::read::unsigned(buf)? as usize;
        let height = leb128::read::unsigned(buf)? as usize;
        let depth = leb128::read::unsigned(buf)? as usize;
        let format = leb128::read::unsigned(buf)? as gl::GLenum;
        let pixel_type = leb128::read::unsigned(buf)? as gl::GLenum;
        let length = leb128::read::unsigned(buf)? as usize;

        let bytes_per_pixel = gl::calculate_bytes_per_pixel(format, pixel_type);
        assert_eq!(length, bytes_per_pixel * width * height * depth);

        let bytes = match bytes_per_pixel {
            1 => rle::read_u8(buf)?,
            4 => {
                let mut words: &[u32] = var::borrow_aligned_slice(buf, length / 4)?;
                rle::read_u32(&mut words)?
            }
            _ => todo!(),
        };

        Ok(Pixels {
            width,
            height,
            depth,
            format,
            pixel_type,
            bytes: bytes.into(),
        })
    }
}
