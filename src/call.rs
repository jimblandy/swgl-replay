//! A representation for recorded `gleam::Gl` method calls.

#![allow(unused_imports)]

use gleam::gl::{GLbitfield, GLenum, GLfloat, GLint, GLsizei, GLsizeiptr, GLuint, GLvoid};

/// An untyped memory buffer, passed as a `data`/`size` pair to some methods.
pub struct GlRawBuf {
    data: *const GLvoid,
    size: GLsizeiptr,
}

impl GlRawBuf {
    /// Create a new GlRawBuf.
    ///
    /// Safety: `data` must actually point to `size` bytes, for as long as the
    /// `GlRawBuf` exists.
    pub unsafe fn new_unchecked(data: *const GLvoid, size: GLsizeiptr) -> GlRawBuf {
        GlRawBuf { data, size }
    }

    pub fn as_slice(&self) -> &[u8] {
        // Safe because of contract on GlRawBuf::new.
        unsafe {
            std::slice::from_raw_parts(self.data as *const u8, self.size as usize)
        }
    }
}

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
    disable { cap: GLenum },
    disable_vertex_attrib_array { index: GLuint },
    enable { cap: GLenum },
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
    tex_image_2d { target: GLenum, level: GLint, internal_format: GLint, width: GLsizei, height: GLsizei, border: GLint, format: GLenum, ty: GLenum, opt_data: Option<BufToGl> },
    tex_image_3d { target: GLenum, level: GLint, internal_format: GLint, width: GLsizei, height: GLsizei, depth: GLsizei, border: GLint, format: GLenum, ty: GLenum, opt_data: Option<BufToGl> },
    tex_parameter_f { target: GLenum, pname: GLenum, param: GLfloat },
    tex_parameter_i { target: GLenum, pname: GLenum, param: GLint },
    tex_sub_image_3d { target: GLenum, level: GLint, xoffset: GLint, yoffset: GLint, zoffset: GLint, width: GLsizei, height: GLsizei, depth: GLsizei, format: GLenum, ty: GLenum, data: BufToGl },
    use_program { program: GLuint, },
    vertex_attrib_divisor { index: GLuint, divisor: GLuint },
    vertex_attrib_i_pointer { index: GLuint, size: GLint, type_: GLenum, stride: GLsizei, offset: GLuint },
    vertex_attrib_pointer { index: GLuint, size: GLint, type_: GLenum, normalized: bool, stride: GLsizei, offset: GLuint },
    viewport { x: GLint, y: GLint, width: GLsizei, height: GLsizei },
    bind_vertex_array_apple { vao: GLuint },
    bind_renderbuffer { target: GLenum, renderbuffer: GLuint },
    bind_framebuffer { target: GLenum, framebuffer: GLuint },
    framebuffer_texture_2d { target: GLenum, attachment: GLenum, textarget: GLenum, texture: GLuint, level: GLint },
    framebuffer_texture_layer { target: GLenum, attachment: GLenum, texture: GLuint, level: GLint, layer: GLint },
    blit_framebuffer { src_x0: GLint, src_y0: GLint, src_x1: GLint, src_y1: GLint, dst_x0: GLint, dst_y0: GLint, dst_x1: GLint, dst_y1: GLint, mask: GLbitfield, filter: GLenum },
    hint { param_name: GLenum, param_val: GLenum },
    is_enabled { cap: GLenum },
    is_shader { shader: GLuint },
    is_texture { texture: GLenum },
    is_framebuffer { framebuffer: GLenum },
    is_renderbuffer { renderbuffer: GLenum },
    check_frame_buffer_status { target: GLenum },
    renderbuffer_storage { target: GLenum, internalformat: GLenum, width: GLsizei, height: GLsizei },
    framebuffer_renderbuffer { target: GLenum, attachment: GLenum, renderbuffertarget: GLenum, renderbuffer: GLuint },
    tex_sub_image_2d_pbo { target: GLenum, level: GLint, xoffset: GLint, yoffset: GLint, width: GLsizei, height: GLsizei, format: GLenum, ty: GLenum, offset: usize },
    flush {  },
    finish {  },
    depth_mask { flag: bool },
    create_program { returned: GLuint },
    create_shader { shader_type: GLenum, returned: GLuint },
    shader_source { shader: GLuint, strings: BufToGl },
    compile_shader { shader: GLuint },
    get_shader_iv { shader: GLuint, pname: GLenum, result: BufFromGl },
    attach_shader { program: GLuint, shader: GLuint },
    bind_attrib_location { program: GLuint, index: GLuint, name: BufToGl },
    link_program { program: GLuint },
    delete_shader { shader: GLuint },
    detach_shader { program: GLuint, shader: GLuint },
    clear { buffer_mask: GLbitfield },
    clear_depth { depth: f64 },
    clear_stencil { s: GLint },
}
