//! A representation for recorded `gleam::Gl` method calls.

#![allow(unused_imports)]

use gleam::gl::{GLenum, GLsizeiptr, GLvoid};

/// An identifier for a memory buffer passing data to GL.
///
/// An argument of this type indicates that this call only reads data from the
/// buffer.
///
/// At recording time, we take the `(*const GLVoid, GLsizeiptr)` pair passed to
/// GL, record the address in a side table, serialize the buffer's contents, and
/// return one of these identifiers to refer to the buffer in the `Call`.
///
/// At playback time, if this is the first time we've seen this ID, we allocate
/// a buffer of the right size, and associate its playback address with the ID.
/// Then we copy the serialized contents into the buffer, and pass it to GL.
#[derive(Copy, Clone, Debug)]
pub struct BufToGl(pub usize);

/// An identifier for a memory buffer receiving data from GL.
///
/// An argument of this type indicates that this call will write, but not read,
/// the buffer's contents.
#[derive(Copy, Clone, Debug)]
pub struct BufFromGl(pub usize);

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub enum Call {
    buffer_data_untyped { target: GLenum, size_data: BufToGl, usage: GLenum },
    clear_color { r: f32, g: f32, b: f32, a: f32 },
    get_string { which: GLenum },
}
