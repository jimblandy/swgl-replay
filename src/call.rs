//! A representation for recorded `gleam::Gl` method calls.

#![allow(unused_imports)]

use gleam::gl::{GLenum, GLfloat, GLint, GLsizei, GLsizeiptr, GLuint, GLvoid};

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

/// An identifier for a memory buffer allocated by and returned from from GL.
#[derive(Copy, Clone, Debug)]
pub struct BufFromGl(pub usize);

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
#[rustfmt::skip]
pub enum Call {
    active_texture { texture: GLenum, },
    bind_buffer { target: GLenum, buffer: GLuint, },
    bind_texture { target: GLenum, texture: GLuint, },
    bind_vertex_array { vao: GLuint, },
    buffer_data_untyped { target: GLenum, size_data: BufToGl, usage: GLenum, },
    clear_color { r: f32, g: f32, b: f32, a: f32, },
    disable_vertex_attrib_array { index: GLuint },
    enable_vertex_attrib_array { index: GLuint },
    gen_buffers { n: GLsizei, returned: BufFromGl },
    gen_framebuffers { n: GLsizei, returned: BufFromGl },
    gen_queries { n: GLsizei, returned: BufFromGl },
    gen_renderbuffers { n: GLsizei, returned: BufFromGl },
    gen_textures { n: GLsizei, returned: BufFromGl },
    gen_vertex_arrays { n: GLsizei, returned: BufFromGl },
    gen_vertex_arrays_apple { n: GLsizei, returned: BufFromGl },
    line_width { width: GLfloat },
    pixel_store_i { name: GLenum, param: GLint, },
    scissor { x: GLint, y: GLint, width: GLsizei, height: GLsizei },
    use_program { program: GLuint, },
    vertex_attrib_divisor { index: GLuint, divisor: GLuint },
    vertex_attrib_i_pointer { index: GLuint, size: GLint, type_: GLenum, stride: GLsizei, offset: GLuint },
    vertex_attrib_pointer { index: GLuint, size: GLint, type_: GLenum, normalized: bool, stride: GLsizei, offset: GLuint },
    viewport { x: GLint, y: GLint, width: GLsizei, height: GLsizei },
    tex_parameter_i { target: GLenum, pname: GLenum, param: GLint },
    tex_parameter_f { target: GLenum, pname: GLenum, param: GLfloat },
    tex_image_3d { target: GLenum, level: GLint, internal_format: GLint, width: GLsizei, height: GLsizei, depth: GLsizei, border: GLint, format: GLenum, ty: GLenum, opt_data: Option<BufToGl> },
    tex_sub_image_3d { target: GLenum, level: GLint, xoffset: GLint, yoffset: GLint, zoffset: GLint, width: GLsizei, height: GLsizei, depth: GLsizei, format: GLenum, ty: GLenum, data: BufToGl },
}
