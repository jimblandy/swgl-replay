//! A representation for recorded `gleam::Gl` method calls.

use gleam::gl::{GLbitfield, GLclampf, GLenum, GLfloat, GLint, GLsizei, GLuint};

use std::os::raw::c_int;

use crate::form::{Seq, Str, Var};
use crate::pixels::PixelsForm;
use crate::raw;

unsafe impl raw::Simple for Call {}

/// Either a buffer, or an offset into the currently bound PIXEL_UNPACK_BUFFER.
///
/// The `tex_image_2d`, `tex_sub_image_2d`, `tex_image_3d`, and
/// `tex_sub_image_3d` methods all take a final data pointer which gets
/// interpreted as an offset into the PIXEL_UNPACK_BUFFER if that is bound,
/// or as a raw address if it is not.
///
/// When it is a pointer to data, we want to save the data being passed in,
/// and on replay pass a pointer to the recorded data.
///
/// When it is an offset, we want to serialize the offset, and pass the
/// identical offset.
#[derive(Copy, Clone, Debug)]
pub enum TexImageData {
    Buf(Var<Seq<u8>>),
    Offset(usize),
}

/// An enum representing all possible `Gl` trait method calls.
///
/// This enum has a variant for each method of `Gl` that holds the arguments
/// passed to the method, and in some cases the return value.
///
/// Some argument types aren't stored directly in the `Call` variant. For
/// example, a `&[u8]` argument has its length and bytes written to a separate
/// data stream, and then the `Call` variant stores a `Var<Seq<u8>>` value,
/// where the `Var` holds the offset at which the length and bytes were written
/// in the other data stream.
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
#[rustfmt::skip]
pub enum Call {
    active_texture { texture: GLenum, },
    bind_buffer { target: GLenum, buffer: GLuint, },
    bind_texture { target: GLenum, texture: GLuint, },
    bind_vertex_array { vao: GLuint, },
    buffer_data_untyped { target: GLenum, size_data: Var<Seq<u8>>, usage: GLenum, },
    clear_color { r: f32, g: f32, b: f32, a: f32, },
    disable { cap: GLenum },
    disable_vertex_attrib_array { index: GLuint },
    enable { cap: GLenum },
    enable_vertex_attrib_array { index: GLuint },
    gen_buffers { n: GLsizei, returned: Var<Seq<GLuint>> },
    gen_framebuffers { n: GLsizei, returned: Var<Seq<GLuint>> },
    gen_queries { n: GLsizei, returned: Var<Seq<GLuint>> },
    gen_renderbuffers { n: GLsizei, returned: Var<Seq<GLuint>> },
    gen_textures { n: GLsizei, returned: Var<Seq<GLuint>> },
    gen_vertex_arrays { n: GLsizei, returned: Var<Seq<GLuint>> },
    gen_vertex_arrays_apple { n: GLsizei, returned: Var<Seq<GLuint>> },
    line_width { width: GLfloat },
    pixel_store_i { name: GLenum, param: GLint, },
    scissor { x: GLint, y: GLint, width: GLsizei, height: GLsizei },
    tex_image_2d { target: GLenum, level: GLint, internal_format: GLint, width: GLsizei, height: GLsizei, border: GLint, format: GLenum, ty: GLenum, opt_data: Option<Var<Seq<u8>>> },
    tex_image_3d { target: GLenum, level: GLint, internal_format: GLint, width: GLsizei, height: GLsizei, depth: GLsizei, border: GLint, format: GLenum, ty: GLenum, opt_data: Option<Var<Seq<u8>>> },
    tex_parameter_f { target: GLenum, pname: GLenum, param: GLfloat },
    tex_parameter_i { target: GLenum, pname: GLenum, param: GLint },
    tex_sub_image_3d { target: GLenum, level: GLint, xoffset: GLint, yoffset: GLint, zoffset: GLint, width: GLsizei, height: GLsizei, depth: GLsizei, format: GLenum, ty: GLenum, data: Var<Seq<u8>> },
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
    tex_sub_image_2d_pbo { target: GLenum, level: GLint, xoffset: GLint, yoffset: GLint, width: GLsizei, height: GLsizei, format: GLenum, ty: GLenum, offset: TexImageData },
    flush {  },
    finish {  },
    depth_mask { flag: bool },
    create_program { returned: GLuint },
    create_shader { shader_type: GLenum, returned: GLuint },
    shader_source { shader: GLuint, strings: Var<Seq<Seq<u8>>> },
    compile_shader { shader: GLuint },
    get_shader_iv { shader: GLuint, pname: GLenum, result: Var<Seq<GLint>> },
    attach_shader { program: GLuint, shader: GLuint },
    bind_attrib_location { program: GLuint, index: GLuint, name: Var<Str> },
    link_program { program: GLuint },
    delete_shader { shader: GLuint },
    detach_shader { program: GLuint, shader: GLuint },
    clear { buffer_mask: GLbitfield },
    clear_depth { depth: f64 },
    clear_stencil { s: GLint },
    get_attrib_location { program: GLuint, name: Var<Str> },
    get_frag_data_location { program: GLuint, name: Var<Str> },
    get_uniform_location { program: GLuint, name: Var<Str>, returned: c_int },
    get_program_iv { program: GLuint, pname: GLenum, result: Var<Seq<GLint>> },
    uniform_1i { location: GLint, v0: GLint },
    uniform_1iv { location: GLint, values: Var<Seq<i32>> },
    uniform_1f { location: GLint, v0: GLfloat },
    uniform_1fv { location: GLint, values: Var<Seq<f32>> },
    uniform_1ui { location: GLint, v0: GLuint },
    uniform_2f { location: GLint, v0: GLfloat, v1: GLfloat },
    uniform_2fv { location: GLint, values: Var<Seq<f32>> },
    uniform_2i { location: GLint, v0: GLint, v1: GLint },
    uniform_2iv { location: GLint, values: Var<Seq<i32>> },
    uniform_2ui { location: GLint, v0: GLuint, v1: GLuint },
    uniform_3f { location: GLint, v0: GLfloat, v1: GLfloat, v2: GLfloat },
    uniform_3fv { location: GLint, values: Var<Seq<f32>> },
    uniform_3i { location: GLint, v0: GLint, v1: GLint, v2: GLint },
    uniform_3iv { location: GLint, values: Var<Seq<i32>> },
    uniform_3ui { location: GLint, v0: GLuint, v1: GLuint, v2: GLuint },
    uniform_4f { location: GLint, x: GLfloat, y: GLfloat, z: GLfloat, w: GLfloat },
    uniform_4i { location: GLint, x: GLint, y: GLint, z: GLint, w: GLint },
    uniform_4iv { location: GLint, values: Var<Seq<i32>> },
    uniform_4ui { location: GLint, x: GLuint, y: GLuint, z: GLuint, w: GLuint },
    uniform_4fv { location: GLint, values: Var<Seq<f32>> },
    uniform_matrix_2fv { location: GLint, transpose: bool, value: Var<Seq<f32>> },
    uniform_matrix_3fv { location: GLint, transpose: bool, value: Var<Seq<f32>> },
    uniform_matrix_4fv { location: GLint, transpose: bool, value: Var<Seq<f32>> },
    depth_range { near: f64, far: f64 },
    draw_elements_instanced { mode: GLenum, count: GLsizei, element_type: GLenum, indices_offset: GLuint, primcount: GLsizei },
    blend_color { r: f32, g: f32, b: f32, a: f32 },
    blend_func { sfactor: GLenum, dfactor: GLenum },
    blend_func_separate { src_rgb: GLenum, dest_rgb: GLenum, src_alpha: GLenum, dest_alpha: GLenum },
    blend_equation { mode: GLenum },
    blend_equation_separate { mode_rgb: GLenum, mode_alpha: GLenum },
    color_mask { r: bool, g: bool, b: bool, a: bool },
    cull_face { mode: GLenum },
    front_face { mode: GLenum },
    depth_func { func: GLenum },
    invalidate_framebuffer { target: GLenum, attachments: Var<Seq<GLenum>> },
    invalidate_sub_framebuffer { target: GLenum, attachments: Var<Seq<GLenum>>, xoffset: GLint, yoffset: GLint, width: GLsizei, height: GLsizei },
    read_buffer { mode: GLenum },
    read_pixels_into_buffer { x: GLint, y: GLint, pixels: Var<PixelsForm> },
    read_pixels { x: GLint, y: GLint, width: GLsizei, height: GLsizei, format: GLenum, pixel_type: GLenum, returned: Var<Seq<u8>> },
    read_pixels_into_pbo { x: GLint, y: GLint, width: GLsizei, height: GLsizei, format: GLenum, pixel_type: GLenum },
    sample_coverage { value: GLclampf, invert: bool },
    polygon_offset { factor: GLfloat, units: GLfloat },
    begin_query { target: GLenum, id: GLuint },
    end_query { target: GLenum },
    query_counter { id: GLuint, target: GLenum },
    get_query_object_iv { id: GLuint, pname: GLenum, returned: i32 },
    get_query_object_uiv { id: GLuint, pname: GLenum, returned: u32 },
    get_query_object_i64v { id: GLuint, pname: GLenum, returned: i64 },
    get_query_object_ui64v { id: GLuint, pname: GLenum, returned: u64 },
    delete_queries { queries: Var<Seq<GLuint>> },
    delete_vertex_arrays { vertex_arrays: Var<Seq<GLuint>> },
    delete_vertex_arrays_apple { vertex_arrays: Var<Seq<GLuint>> },
    delete_buffers { buffers: Var<Seq<GLuint>> },
    delete_renderbuffers { renderbuffers: Var<Seq<GLuint>> },
    delete_framebuffers { framebuffers: Var<Seq<GLuint>> },
    delete_textures { textures: Var<Seq<GLuint>> },
    delete_program { program: GLuint },
    tex_sub_image_3d_pbo { target: GLenum, level: GLint, xoffset: GLint, yoffset: GLint, zoffset: GLint, width: GLsizei, height: GLsizei, depth: GLsizei, format: GLenum, ty: GLenum, offset: TexImageData },
    tex_storage_2d { target: GLenum, levels: GLint, internal_format: GLenum, width: GLsizei, height: GLsizei },
    tex_storage_3d { target: GLenum, levels: GLint, internal_format: GLenum, width: GLsizei, height: GLsizei, depth: GLsizei },
    get_tex_image_into_buffer { target: GLenum, level: GLint, format: GLenum, ty: GLenum, output: Var<Seq<u8>> },
    copy_image_sub_data { src_name: GLuint, src_target: GLenum, src_level: GLint, src_x: GLint, src_y: GLint, src_z: GLint, dst_name: GLuint, dst_target: GLenum, dst_level: GLint, dst_x: GLint, dst_y: GLint, dst_z: GLint, src_width: GLsizei, src_height: GLsizei, src_depth: GLsizei },
    generate_mipmap { target: GLenum },
}
