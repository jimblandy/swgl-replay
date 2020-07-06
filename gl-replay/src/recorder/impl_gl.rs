//! Implementation of the `gleam::Gl` trait for `Recorder`.

use gleam::gl::*;
use std::os::raw::{c_int, c_void};

use super::Recorder;
use crate::call::{Call, TexImageData};
use crate::pixels;
use crate::var::CallStream;
use crate::Parameter;

macro_rules! check {
    ($call:expr) => {
        $call.expect("gl-replay serialization failure")
    };
}

fn tex_image_data_to_call<G, Cs>(
    inner_gl: &G,
    call_stream: &mut Cs,
    width: GLsizei,
    height: GLsizei,
    depth: GLsizei,
    format: GLenum,
    ty: GLenum,
    offset: usize,
) -> TexImageData
where
    G: gleam::gl::Gl,
    Cs: CallStream<Call>,
{
    // If there is a buffer bound to PIXEL_UNPACK_BUFFER, then `offset` is an
    // offset; otherwise, it's an address.
    let mut bound_buffer = 0;
    unsafe {
        inner_gl.get_integer_v(gleam::gl::PIXEL_UNPACK_BUFFER_BINDING,
                               std::slice::from_mut(&mut bound_buffer));
    }
    if bound_buffer != 0 {
        TexImageData::Offset(offset)
    } else {
        let mut unpack_row_length = 0;
        unsafe {
            inner_gl.get_integer_v(gleam::gl::UNPACK_ROW_LENGTH,
                                   std::slice::from_mut(&mut unpack_row_length));
        }
        let actual_width = if unpack_row_length != 0 {
            assert!(width <= unpack_row_length);
            unpack_row_length
        } else {
            width
        };
        let length = gleam::gl::calculate_length(actual_width, height, depth, format, ty);
        let slice = unsafe { std::slice::from_raw_parts(offset as *const u8, length) };
        TexImageData::Buf(check!(slice.to_call(call_stream)))
    }
}

/// General form of call that has no side effects, and hence doesn't need to be
/// recorded.
macro_rules! no_side_effect {
    ($self:ident . $method:ident ( $( $arg:ident ),* )) => {
        {
            $self .inner_gl. $method ( $( $arg ),* )
        }
    }
}

/// General form of a recorded call. Always makes the call, and returns its value.
macro_rules! general {
    (
        let $returned:ident = $self:ident . $method:ident ( $( $arg:ident ),* );
        lock $call_stream:ident;
        $body:expr
    ) => {
        {
            let $returned = $self .inner_gl. $method ( $( $arg ),* );
            let $call_stream = &mut *$self .call_stream.lock().unwrap();

            $body;

            // For debugging.
            $call_stream .flush()
                .expect("gl-replay serialization failure");

            if let Some(fingerprinter) = $self .fingerprinter {
                fingerprinter(& $self .inner_gl, $call_stream);
            }

            $returned
        }
    }
}

macro_rules! simple {
    ($self:ident . $method:ident ( $( $arg:ident ),* )) => {
        general! {
            let returned = $self . $method ( $( $arg ),* );
            lock call_stream;
            {
                let call = Call:: $method {
                    $(
                        $arg : check!($arg .to_call(call_stream))
                    ),*
                };

                check!(call_stream.write_call(call));
            }
        }
    }
}

macro_rules! simple_with_return_value {
    ($self:ident . $method:ident ( $( $arg:ident ),* )) => {
        general! {
            let returned = $self . $method ( $( $arg ),* );
            lock call_stream;
            {
                let call = Call::$method {
                    $( $arg : check!($arg .to_call(call_stream)), )*
                    returned: check!(returned.to_call(call_stream))
                };
                check!(call_stream.write_call(call));
            }
        }
    }
}

#[allow(unused_variables)]
impl<G, Cs> gleam::gl::Gl for Recorder<G, Cs>
where
    G: Gl,
    Cs: CallStream<Call>,
{
    fn get_type(&self) -> GlType {
        no_side_effect!(self.get_type())
    }

    fn buffer_data_untyped(
        &self,
        target: GLenum,
        size: GLsizeiptr,
        data: *const GLvoid,
        usage: GLenum,
    ) {
        general! {
            let returned = self.buffer_data_untyped(target, size, data, usage);
            lock call_stream;
            {
                let size_data = unsafe {
                    std::slice::from_raw_parts(data as *const u8, size as usize)
                };
                let call = Call::buffer_data_untyped {
                    target,
                    size_data: check!(size_data.to_call(call_stream)),
                    usage,
                };
                check!(call_stream.write_call(call));
            }
        }
    }

    fn buffer_sub_data_untyped(
        &self,
        target: GLenum,
        offset: isize,
        size: GLsizeiptr,
        data: *const GLvoid,
    ) {
        unimplemented!("buffer_sub_data_untyped");
    }

    fn map_buffer(&self, target: GLenum, access: GLbitfield) -> *mut c_void {
        unimplemented!("map_buffer");
    }

    fn map_buffer_range(
        &self,
        target: GLenum,
        offset: GLintptr,
        length: GLsizeiptr,
        access: GLbitfield,
    ) -> *mut c_void {
        unimplemented!("map_buffer_range");
    }

    fn unmap_buffer(&self, target: GLenum) -> GLboolean {
        unimplemented!("unmap_buffer");
    }

    fn tex_buffer(&self, target: GLenum, internal_format: GLenum, buffer: GLuint) {
        unimplemented!("tex_buffer");
    }

    fn shader_source(&self, shader: GLuint, strings: &[&[u8]]) {
        simple!(self.shader_source(shader, strings))
    }

    fn read_buffer(&self, mode: GLenum) {
        simple!(self.read_buffer(mode))
    }

    fn read_pixels_into_buffer(
        &self,
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        pixel_type: GLenum,
        dst_buffer: &mut [u8],
    ) {
        general!(
            let _unit = self.read_pixels_into_buffer(x, y, width, height, format, pixel_type, dst_buffer);
            lock call_stream;
            {
                let pixels = pixels::Pixels {
                    width: width as usize,
                    height: height as usize,
                    depth: 1,
                    format,
                    pixel_type,
                    bytes: std::borrow::Cow::from(&*dst_buffer),
                };
                let call = Call::read_pixels_into_buffer {
                    x,
                    y,
                    pixels: check!(pixels.to_call(call_stream)),
                };

                check!(call_stream.write_call(call));
            }
        )
    }

    fn read_pixels(
        &self,
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        pixel_type: GLenum,
    ) -> Vec<u8> {
        simple_with_return_value!(self.read_pixels(x, y, width, height, format, pixel_type))
    }

    unsafe fn read_pixels_into_pbo(
        &self,
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        pixel_type: GLenum,
    ) {
        simple!(self.read_pixels_into_pbo(x, y, width, height, format, pixel_type))
    }

    fn sample_coverage(&self, value: GLclampf, invert: bool) {
        simple!(self.sample_coverage(value, invert))
    }

    fn polygon_offset(&self, factor: GLfloat, units: GLfloat) {
        simple!(self.polygon_offset(factor, units))
    }

    fn pixel_store_i(&self, name: GLenum, param: GLint) {
        simple!(self.pixel_store_i(name, param))
    }

    fn gen_buffers(&self, n: GLsizei) -> Vec<GLuint> {
        simple_with_return_value!(self.gen_buffers(n))
    }

    fn gen_renderbuffers(&self, n: GLsizei) -> Vec<GLuint> {
        simple_with_return_value!(self.gen_renderbuffers(n))
    }

    fn gen_framebuffers(&self, n: GLsizei) -> Vec<GLuint> {
        simple_with_return_value!(self.gen_framebuffers(n))
    }

    fn gen_textures(&self, n: GLsizei) -> Vec<GLuint> {
        simple_with_return_value!(self.gen_textures(n))
    }

    fn gen_vertex_arrays(&self, n: GLsizei) -> Vec<GLuint> {
        simple_with_return_value!(self.gen_vertex_arrays(n))
    }

    fn gen_vertex_arrays_apple(&self, n: GLsizei) -> Vec<GLuint> {
        simple_with_return_value!(self.gen_vertex_arrays_apple(n))
    }

    fn gen_queries(&self, n: GLsizei) -> Vec<GLuint> {
        simple_with_return_value!(self.gen_queries(n))
    }

    fn begin_query(&self, target: GLenum, id: GLuint) {
        simple!(self.begin_query(target, id))
    }

    fn end_query(&self, target: GLenum) {
        simple!(self.end_query(target))
    }

    fn query_counter(&self, id: GLuint, target: GLenum) {
        simple!(self.query_counter(id, target))
    }

    fn get_query_object_iv(&self, id: GLuint, pname: GLenum) -> i32 {
        simple_with_return_value!(self.get_query_object_iv(id, pname))
    }

    fn get_query_object_uiv(&self, id: GLuint, pname: GLenum) -> u32 {
        simple_with_return_value!(self.get_query_object_uiv(id, pname))
    }

    fn get_query_object_i64v(&self, id: GLuint, pname: GLenum) -> i64 {
        simple_with_return_value!(self.get_query_object_i64v(id, pname))
    }

    fn get_query_object_ui64v(&self, id: GLuint, pname: GLenum) -> u64 {
        simple_with_return_value!(self.get_query_object_ui64v(id, pname))
    }

    fn delete_queries(&self, queries: &[GLuint]) {
        simple!(self.delete_queries(queries))
    }

    fn delete_vertex_arrays(&self, vertex_arrays: &[GLuint]) {
        simple!(self.delete_vertex_arrays(vertex_arrays))
    }

    fn delete_vertex_arrays_apple(&self, vertex_arrays: &[GLuint]) {
        simple!(self.delete_vertex_arrays_apple(vertex_arrays))
    }

    fn delete_buffers(&self, buffers: &[GLuint]) {
        simple!(self.delete_buffers(buffers))
    }

    fn delete_renderbuffers(&self, renderbuffers: &[GLuint]) {
        simple!(self.delete_renderbuffers(renderbuffers))
    }

    fn delete_framebuffers(&self, framebuffers: &[GLuint]) {
        simple!(self.delete_framebuffers(framebuffers))
    }

    fn delete_textures(&self, textures: &[GLuint]) {
        simple!(self.delete_textures(textures))
    }

    fn framebuffer_renderbuffer(
        &self,
        target: GLenum,
        attachment: GLenum,
        renderbuffertarget: GLenum,
        renderbuffer: GLuint,
    ) {
        simple!(self.framebuffer_renderbuffer(target, attachment, renderbuffertarget, renderbuffer))
    }

    fn renderbuffer_storage(
        &self,
        target: GLenum,
        internalformat: GLenum,
        width: GLsizei,
        height: GLsizei,
    ) {
        simple!(self.renderbuffer_storage(target, internalformat, width, height))
    }

    fn depth_func(&self, func: GLenum) {
        simple!(self.depth_func(func))
    }

    fn active_texture(&self, texture: GLenum) {
        simple!(self.active_texture(texture))
    }

    fn attach_shader(&self, program: GLuint, shader: GLuint) {
        simple!(self.attach_shader(program, shader))
    }

    fn bind_attrib_location(&self, program: GLuint, index: GLuint, name: &str) {
        simple!(self.bind_attrib_location(program, index, name))
    }

    unsafe fn get_uniform_iv(&self, program: GLuint, location: GLint, result: &mut [GLint]) {
        unimplemented!("get_uniform_iv");
    }
    unsafe fn get_uniform_fv(&self, program: GLuint, location: GLint, result: &mut [GLfloat]) {
        unimplemented!("get_uniform_fv");
    }

    fn get_uniform_block_index(&self, program: GLuint, name: &str) -> GLuint {
        unimplemented!("get_uniform_block_index");
    }

    fn get_uniform_indices(&self, program: GLuint, names: &[&str]) -> Vec<GLuint> {
        unimplemented!("get_uniform_indices");
    }

    fn bind_buffer_base(&self, target: GLenum, index: GLuint, buffer: GLuint) {
        unimplemented!("bind_buffer_base");
    }

    fn bind_buffer_range(
        &self,
        target: GLenum,
        index: GLuint,
        buffer: GLuint,
        offset: GLintptr,
        size: GLsizeiptr,
    ) {
        unimplemented!("bind_buffer_range");
    }

    fn uniform_block_binding(
        &self,
        program: GLuint,
        uniform_block_index: GLuint,
        uniform_block_binding: GLuint,
    ) {
        unimplemented!("uniform_block_binding");
    }

    fn bind_buffer(&self, target: GLenum, buffer: GLuint) {
        simple!(self.bind_buffer(target, buffer))
    }

    fn bind_vertex_array(&self, vao: GLuint) {
        simple!(self.bind_vertex_array(vao))
    }

    fn bind_vertex_array_apple(&self, vao: GLuint) {
        simple!(self.bind_vertex_array_apple(vao))
    }

    fn bind_renderbuffer(&self, target: GLenum, renderbuffer: GLuint) {
        simple!(self.bind_renderbuffer(target, renderbuffer))
    }

    fn bind_framebuffer(&self, target: GLenum, framebuffer: GLuint) {
        simple!(self.bind_framebuffer(target, framebuffer))
    }

    fn bind_texture(&self, target: GLenum, texture: GLuint) {
        simple!(self.bind_texture(target, texture))
    }

    fn draw_buffers(&self, bufs: &[GLenum]) {
        unimplemented!("draw_buffers");
    }

    fn tex_image_2d(
        &self,
        target: GLenum,
        level: GLint,
        internal_format: GLint,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
        format: GLenum,
        ty: GLenum,
        opt_data: Option<&[u8]>,
    ) {
        simple!(self.tex_image_2d(
            target,
            level,
            internal_format,
            width,
            height,
            border,
            format,
            ty,
            opt_data
        ))
    }

    fn compressed_tex_image_2d(
        &self,
        target: GLenum,
        level: GLint,
        internal_format: GLenum,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
        data: &[u8],
    ) {
        unimplemented!("compressed_tex_image_2d");
    }

    fn compressed_tex_sub_image_2d(
        &self,
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        yoffset: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        data: &[u8],
    ) {
        unimplemented!("compressed_tex_sub_image_2d");
    }

    fn tex_image_3d(
        &self,
        target: GLenum,
        level: GLint,
        internal_format: GLint,
        width: GLsizei,
        height: GLsizei,
        depth: GLsizei,
        border: GLint,
        format: GLenum,
        ty: GLenum,
        opt_data: Option<&[u8]>,
    ) {
        simple!(self.tex_image_3d(
            target,
            level,
            internal_format,
            width,
            height,
            depth,
            border,
            format,
            ty,
            opt_data
        ))
    }

    fn copy_tex_image_2d(
        &self,
        target: GLenum,
        level: GLint,
        internal_format: GLenum,
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
    ) {
        unimplemented!("copy_tex_image_2d");
    }

    fn copy_tex_sub_image_2d(
        &self,
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        yoffset: GLint,
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
    ) {
        unimplemented!("copy_tex_sub_image_2d");
    }

    fn copy_tex_sub_image_3d(
        &self,
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        yoffset: GLint,
        zoffset: GLint,
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
    ) {
        unimplemented!("copy_tex_sub_image_3d");
    }

    fn tex_sub_image_2d(
        &self,
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        yoffset: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        ty: GLenum,
        data: &[u8],
    ) {
        unimplemented!("tex_sub_image_2d");
    }

    fn tex_sub_image_2d_pbo(
        &self,
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        yoffset: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        ty: GLenum,
        offset: usize,
    ) {
        general! {
            let returned = self.tex_sub_image_2d_pbo(
                target, level, xoffset, yoffset, width, height, format, ty, offset
            );
            lock call_stream;
            {
                let offset = tex_image_data_to_call(&self.inner_gl, call_stream,
                                                    width, height, 1, format, ty,
                                                    offset);
                check!(call_stream.write_call(Call::tex_sub_image_2d_pbo {
                    target, level, xoffset, yoffset, width, height,
                    format, ty, offset
                }));
            }
        }
    }

    fn tex_sub_image_3d(
        &self,
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        yoffset: GLint,
        zoffset: GLint,
        width: GLsizei,
        height: GLsizei,
        depth: GLsizei,
        format: GLenum,
        ty: GLenum,
        data: &[u8],
    ) {
        simple!(self.tex_sub_image_3d(
            target, level, xoffset, yoffset, zoffset, width, height, depth, format, ty, data
        ))
    }

    fn tex_sub_image_3d_pbo(
        &self,
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        yoffset: GLint,
        zoffset: GLint,
        width: GLsizei,
        height: GLsizei,
        depth: GLsizei,
        format: GLenum,
        ty: GLenum,
        offset: usize,
    ) {
        general! {
            let returned = self.tex_sub_image_3d_pbo(
                target, level, xoffset, yoffset, zoffset, width, height, depth, format, ty, offset
            );
            lock call_stream;
            {
                let offset = tex_image_data_to_call(&self.inner_gl, call_stream,
                                                    width, height, depth, format, ty,
                                                    offset);
                check!(call_stream.write_call(Call::tex_sub_image_3d_pbo {
                    target, level, xoffset, yoffset, zoffset,
                    width, height, depth,
                    format, ty, offset
                }));
            }
        }
    }

    fn tex_storage_2d(
        &self,
        target: GLenum,
        levels: GLint,
        internal_format: GLenum,
        width: GLsizei,
        height: GLsizei,
    ) {
        simple!(self.tex_storage_2d(target, levels, internal_format, width, height))
    }

    fn tex_storage_3d(
        &self,
        target: GLenum,
        levels: GLint,
        internal_format: GLenum,
        width: GLsizei,
        height: GLsizei,
        depth: GLsizei,
    ) {
        simple!(self.tex_storage_3d(target, levels, internal_format, width, height, depth))
    }

    fn get_tex_image_into_buffer(
        &self,
        target: GLenum,
        level: GLint,
        format: GLenum,
        ty: GLenum,
        output: &mut [u8],
    ) {
        simple!(self.get_tex_image_into_buffer(target, level, format, ty, output))
    }

    unsafe fn copy_image_sub_data(
        &self,
        src_name: GLuint,
        src_target: GLenum,
        src_level: GLint,
        src_x: GLint,
        src_y: GLint,
        src_z: GLint,
        dst_name: GLuint,
        dst_target: GLenum,
        dst_level: GLint,
        dst_x: GLint,
        dst_y: GLint,
        dst_z: GLint,
        src_width: GLsizei,
        src_height: GLsizei,
        src_depth: GLsizei,
    ) {
        simple!(self.copy_image_sub_data(
            src_name, src_target, src_level, src_x, src_y, src_z, dst_name, dst_target, dst_level,
            dst_x, dst_y, dst_z, src_width, src_height, src_depth
        ))
    }

    fn invalidate_framebuffer(&self, target: GLenum, attachments: &[GLenum]) {
        simple!(self.invalidate_framebuffer(target, attachments))
    }

    fn invalidate_sub_framebuffer(
        &self,
        target: GLenum,
        attachments: &[GLenum],
        xoffset: GLint,
        yoffset: GLint,
        width: GLsizei,
        height: GLsizei,
    ) {
        simple!(self.invalidate_sub_framebuffer(
            target,
            attachments,
            xoffset,
            yoffset,
            width,
            height
        ))
    }

    unsafe fn get_integer_v(&self, name: GLenum, result: &mut [GLint]) {
        no_side_effect!(self.get_integer_v(name, result))
    }
    unsafe fn get_integer_64v(&self, name: GLenum, result: &mut [GLint64]) {
        no_side_effect!(self.get_integer_64v(name, result))
    }
    unsafe fn get_integer_iv(&self, name: GLenum, index: GLuint, result: &mut [GLint]) {
        no_side_effect!(self.get_integer_iv(name, index, result))
    }
    unsafe fn get_integer_64iv(&self, name: GLenum, index: GLuint, result: &mut [GLint64]) {
        no_side_effect!(self.get_integer_64iv(name, index, result))
    }
    unsafe fn get_boolean_v(&self, name: GLenum, result: &mut [GLboolean]) {
        no_side_effect!(self.get_boolean_v(name, result))
    }
    unsafe fn get_float_v(&self, name: GLenum, result: &mut [GLfloat]) {
        no_side_effect!(self.get_float_v(name, result))
    }

    fn get_framebuffer_attachment_parameter_iv(
        &self,
        target: GLenum,
        attachment: GLenum,
        pname: GLenum,
    ) -> GLint {
        unimplemented!("get_framebuffer_attachment_parameter_iv");
    }

    fn get_renderbuffer_parameter_iv(&self, target: GLenum, pname: GLenum) -> GLint {
        unimplemented!("get_renderbuffer_parameter_iv");
    }

    fn get_tex_parameter_iv(&self, target: GLenum, name: GLenum) -> GLint {
        unimplemented!("get_tex_parameter_iv");
    }

    fn get_tex_parameter_fv(&self, target: GLenum, name: GLenum) -> GLfloat {
        unimplemented!("get_tex_parameter_fv");
    }

    fn tex_parameter_i(&self, target: GLenum, pname: GLenum, param: GLint) {
        simple!(self.tex_parameter_i(target, pname, param))
    }

    fn tex_parameter_f(&self, target: GLenum, pname: GLenum, param: GLfloat) {
        simple!(self.tex_parameter_f(target, pname, param))
    }

    fn framebuffer_texture_2d(
        &self,
        target: GLenum,
        attachment: GLenum,
        textarget: GLenum,
        texture: GLuint,
        level: GLint,
    ) {
        simple!(self.framebuffer_texture_2d(target, attachment, textarget, texture, level))
    }

    fn framebuffer_texture_layer(
        &self,
        target: GLenum,
        attachment: GLenum,
        texture: GLuint,
        level: GLint,
        layer: GLint,
    ) {
        simple!(self.framebuffer_texture_layer(target, attachment, texture, level, layer))
    }

    fn blit_framebuffer(
        &self,
        src_x0: GLint,
        src_y0: GLint,
        src_x1: GLint,
        src_y1: GLint,
        dst_x0: GLint,
        dst_y0: GLint,
        dst_x1: GLint,
        dst_y1: GLint,
        mask: GLbitfield,
        filter: GLenum,
    ) {
        simple!(self.blit_framebuffer(
            src_x0, src_y0, src_x1, src_y1, dst_x0, dst_y0, dst_x1, dst_y1, mask, filter
        ))
    }

    fn vertex_attrib_4f(&self, index: GLuint, x: GLfloat, y: GLfloat, z: GLfloat, w: GLfloat) {
        unimplemented!("vertex_attrib_4f");
    }

    fn vertex_attrib_pointer_f32(
        &self,
        index: GLuint,
        size: GLint,
        normalized: bool,
        stride: GLsizei,
        offset: GLuint,
    ) {
        unimplemented!("vertex_attrib_pointer_f32");
    }

    fn vertex_attrib_pointer(
        &self,
        index: GLuint,
        size: GLint,
        type_: GLenum,
        normalized: bool,
        stride: GLsizei,
        offset: GLuint,
    ) {
        simple!(self.vertex_attrib_pointer(index, size, type_, normalized, stride, offset))
    }

    fn vertex_attrib_i_pointer(
        &self,
        index: GLuint,
        size: GLint,
        type_: GLenum,
        stride: GLsizei,
        offset: GLuint,
    ) {
        simple!(self.vertex_attrib_i_pointer(index, size, type_, stride, offset))
    }

    fn vertex_attrib_divisor(&self, index: GLuint, divisor: GLuint) {
        simple!(self.vertex_attrib_divisor(index, divisor))
    }

    fn viewport(&self, x: GLint, y: GLint, width: GLsizei, height: GLsizei) {
        simple!(self.viewport(x, y, width, height))
    }

    fn scissor(&self, x: GLint, y: GLint, width: GLsizei, height: GLsizei) {
        simple!(self.scissor(x, y, width, height))
    }

    fn line_width(&self, width: GLfloat) {
        simple!(self.line_width(width))
    }

    fn use_program(&self, program: GLuint) {
        simple!(self.use_program(program))
    }

    fn validate_program(&self, program: GLuint) {
        unimplemented!("validate_program");
    }

    fn draw_arrays(&self, mode: GLenum, first: GLint, count: GLsizei) {
        unimplemented!("draw_arrays");
    }

    fn draw_arrays_instanced(
        &self,
        mode: GLenum,
        first: GLint,
        count: GLsizei,
        primcount: GLsizei,
    ) {
        unimplemented!("draw_arrays_instanced");
    }

    fn draw_elements(
        &self,
        mode: GLenum,
        count: GLsizei,
        element_type: GLenum,
        indices_offset: GLuint,
    ) {
        unimplemented!("draw_elements");
    }

    fn draw_elements_instanced(
        &self,
        mode: GLenum,
        count: GLsizei,
        element_type: GLenum,
        indices_offset: GLuint,
        primcount: GLsizei,
    ) {
        simple!(self.draw_elements_instanced(mode, count, element_type, indices_offset, primcount))
    }

    fn blend_color(&self, r: f32, g: f32, b: f32, a: f32) {
        simple!(self.blend_color(r, g, b, a))
    }

    fn blend_func(&self, sfactor: GLenum, dfactor: GLenum) {
        simple!(self.blend_func(sfactor, dfactor))
    }

    fn blend_func_separate(
        &self,
        src_rgb: GLenum,
        dest_rgb: GLenum,
        src_alpha: GLenum,
        dest_alpha: GLenum,
    ) {
        simple!(self.blend_func_separate(src_rgb, dest_rgb, src_alpha, dest_alpha))
    }

    fn blend_equation(&self, mode: GLenum) {
        simple!(self.blend_equation(mode))
    }

    fn blend_equation_separate(&self, mode_rgb: GLenum, mode_alpha: GLenum) {
        simple!(self.blend_equation_separate(mode_rgb, mode_alpha))
    }

    fn color_mask(&self, r: bool, g: bool, b: bool, a: bool) {
        simple!(self.color_mask(r, g, b, a))
    }

    fn cull_face(&self, mode: GLenum) {
        simple!(self.cull_face(mode))
    }

    fn front_face(&self, mode: GLenum) {
        simple!(self.front_face(mode))
    }

    fn enable(&self, cap: GLenum) {
        simple!(self.enable(cap))
    }

    fn disable(&self, cap: GLenum) {
        simple!(self.disable(cap))
    }

    fn hint(&self, param_name: GLenum, param_val: GLenum) {
        simple!(self.hint(param_name, param_val))
    }

    fn is_enabled(&self, cap: GLenum) -> GLboolean {
        simple!(self.is_enabled(cap))
    }

    fn is_shader(&self, shader: GLuint) -> GLboolean {
        simple!(self.is_shader(shader))
    }

    fn is_texture(&self, texture: GLenum) -> GLboolean {
        simple!(self.is_texture(texture))
    }

    fn is_framebuffer(&self, framebuffer: GLenum) -> GLboolean {
        simple!(self.is_framebuffer(framebuffer))
    }

    fn is_renderbuffer(&self, renderbuffer: GLenum) -> GLboolean {
        simple!(self.is_renderbuffer(renderbuffer))
    }

    fn check_frame_buffer_status(&self, target: GLenum) -> GLenum {
        simple!(self.check_frame_buffer_status(target))
    }

    fn enable_vertex_attrib_array(&self, index: GLuint) {
        simple!(self.enable_vertex_attrib_array(index));
    }

    fn disable_vertex_attrib_array(&self, index: GLuint) {
        simple!(self.disable_vertex_attrib_array(index));
    }

    fn uniform_1f(&self, location: GLint, v0: GLfloat) {
        simple!(self.uniform_1f(location, v0))
    }

    fn uniform_1fv(&self, location: GLint, values: &[f32]) {
        simple!(self.uniform_1fv(location, values))
    }

    fn uniform_1i(&self, location: GLint, v0: GLint) {
        simple!(self.uniform_1i(location, v0))
    }

    fn uniform_1iv(&self, location: GLint, values: &[i32]) {
        simple!(self.uniform_1iv(location, values))
    }

    fn uniform_1ui(&self, location: GLint, v0: GLuint) {
        simple!(self.uniform_1ui(location, v0))
    }

    fn uniform_2f(&self, location: GLint, v0: GLfloat, v1: GLfloat) {
        simple!(self.uniform_2f(location, v0, v1))
    }

    fn uniform_2fv(&self, location: GLint, values: &[f32]) {
        simple!(self.uniform_2fv(location, values))
    }

    fn uniform_2i(&self, location: GLint, v0: GLint, v1: GLint) {
        simple!(self.uniform_2i(location, v0, v1))
    }

    fn uniform_2iv(&self, location: GLint, values: &[i32]) {
        simple!(self.uniform_2iv(location, values))
    }

    fn uniform_2ui(&self, location: GLint, v0: GLuint, v1: GLuint) {
        simple!(self.uniform_2ui(location, v0, v1))
    }

    fn uniform_3f(&self, location: GLint, v0: GLfloat, v1: GLfloat, v2: GLfloat) {
        simple!(self.uniform_3f(location, v0, v1, v2))
    }

    fn uniform_3fv(&self, location: GLint, values: &[f32]) {
        simple!(self.uniform_3fv(location, values))
    }

    fn uniform_3i(&self, location: GLint, v0: GLint, v1: GLint, v2: GLint) {
        simple!(self.uniform_3i(location, v0, v1, v2))
    }

    fn uniform_3iv(&self, location: GLint, values: &[i32]) {
        simple!(self.uniform_3iv(location, values))
    }

    fn uniform_3ui(&self, location: GLint, v0: GLuint, v1: GLuint, v2: GLuint) {
        simple!(self.uniform_3ui(location, v0, v1, v2))
    }

    fn uniform_4f(&self, location: GLint, x: GLfloat, y: GLfloat, z: GLfloat, w: GLfloat) {
        simple!(self.uniform_4f(location, x, y, z, w))
    }

    fn uniform_4i(&self, location: GLint, x: GLint, y: GLint, z: GLint, w: GLint) {
        simple!(self.uniform_4i(location, x, y, z, w))
    }

    fn uniform_4iv(&self, location: GLint, values: &[i32]) {
        simple!(self.uniform_4iv(location, values))
    }

    fn uniform_4ui(&self, location: GLint, x: GLuint, y: GLuint, z: GLuint, w: GLuint) {
        simple!(self.uniform_4ui(location, x, y, z, w))
    }

    fn uniform_4fv(&self, location: GLint, values: &[f32]) {
        simple!(self.uniform_4fv(location, values))
    }

    fn uniform_matrix_2fv(&self, location: GLint, transpose: bool, value: &[f32]) {
        simple!(self.uniform_matrix_2fv(location, transpose, value))
    }

    fn uniform_matrix_3fv(&self, location: GLint, transpose: bool, value: &[f32]) {
        simple!(self.uniform_matrix_3fv(location, transpose, value))
    }

    fn uniform_matrix_4fv(&self, location: GLint, transpose: bool, value: &[f32]) {
        simple!(self.uniform_matrix_4fv(location, transpose, value))
    }

    fn depth_mask(&self, flag: bool) {
        simple!(self.depth_mask(flag))
    }

    fn depth_range(&self, near: f64, far: f64) {
        simple!(self.depth_range(near, far))
    }

    fn get_active_attrib(&self, program: GLuint, index: GLuint) -> (i32, u32, String) {
        unimplemented!("get_active_attrib");
    }

    fn get_active_uniform(&self, program: GLuint, index: GLuint) -> (i32, u32, String) {
        unimplemented!("get_active_uniform");
    }

    fn get_active_uniforms_iv(
        &self,
        program: GLuint,
        indices: Vec<GLuint>,
        pname: GLenum,
    ) -> Vec<GLint> {
        unimplemented!("get_active_uniforms_iv");
    }

    fn get_active_uniform_block_i(&self, program: GLuint, index: GLuint, pname: GLenum) -> GLint {
        unimplemented!("get_active_uniform_block_i");
    }

    fn get_active_uniform_block_iv(
        &self,
        program: GLuint,
        index: GLuint,
        pname: GLenum,
    ) -> Vec<GLint> {
        unimplemented!("get_active_uniform_block_iv");
    }

    fn get_active_uniform_block_name(&self, program: GLuint, index: GLuint) -> String {
        unimplemented!("get_active_uniform_block_name");
    }

    fn get_attrib_location(&self, program: GLuint, name: &str) -> c_int {
        simple!(self.get_attrib_location(program, name))
    }

    fn get_frag_data_location(&self, program: GLuint, name: &str) -> c_int {
        simple!(self.get_frag_data_location(program, name))
    }

    fn get_uniform_location(&self, program: GLuint, name: &str) -> c_int {
        simple_with_return_value!(self.get_uniform_location(program, name))
    }

    fn get_program_info_log(&self, program: GLuint) -> String {
        unimplemented!("get_program_info_log");
    }
    unsafe fn get_program_iv(&self, program: GLuint, pname: GLenum, result: &mut [GLint]) {
        simple!(self.get_program_iv(program, pname, result))
    }

    fn get_program_binary(&self, program: GLuint) -> (Vec<u8>, GLenum) {
        unimplemented!("get_program_binary");
    }

    fn program_binary(&self, program: GLuint, format: GLenum, binary: &[u8]) {
        unimplemented!("program_binary");
    }

    fn program_parameter_i(&self, program: GLuint, pname: GLenum, value: GLint) {
        unimplemented!("program_parameter_i");
    }

    unsafe fn get_vertex_attrib_iv(&self, index: GLuint, pname: GLenum, result: &mut [GLint]) {
        unimplemented!("get_vertex_attrib_iv");
    }

    unsafe fn get_vertex_attrib_fv(&self, index: GLuint, pname: GLenum, result: &mut [GLfloat]) {
        unimplemented!("get_vertex_attrib_fv");
    }

    fn get_vertex_attrib_pointer_v(&self, index: GLuint, pname: GLenum) -> GLsizeiptr {
        unimplemented!("get_vertex_attrib_pointer_v");
    }

    fn get_buffer_parameter_iv(&self, target: GLuint, pname: GLenum) -> GLint {
        unimplemented!("get_buffer_parameter_iv");
    }

    fn get_shader_info_log(&self, shader: GLuint) -> String {
        no_side_effect!(self.get_shader_info_log(shader))
    }

    fn get_string(&self, which: GLenum) -> String {
        no_side_effect!(self.get_string(which))
    }

    fn get_string_i(&self, which: GLenum, index: GLuint) -> String {
        no_side_effect!(self.get_string_i(which, index))
    }

    unsafe fn get_shader_iv(&self, shader: GLuint, pname: GLenum, result: &mut [GLint]) {
        simple!(self.get_shader_iv(shader, pname, result))
    }

    fn get_shader_precision_format(
        &self,
        shader_type: GLuint,
        precision_type: GLuint,
    ) -> (GLint, GLint, GLint) {
        unimplemented!("get_shader_precision_format");
    }

    fn compile_shader(&self, shader: GLuint) {
        simple!(self.compile_shader(shader))
    }

    fn create_program(&self) -> GLuint {
        simple_with_return_value!(self.create_program())
    }

    fn delete_program(&self, program: GLuint) {
        simple!(self.delete_program(program))
    }

    fn create_shader(&self, shader_type: GLenum) -> GLuint {
        simple_with_return_value!(self.create_shader(shader_type))
    }

    fn delete_shader(&self, shader: GLuint) {
        simple!(self.delete_shader(shader))
    }

    fn detach_shader(&self, program: GLuint, shader: GLuint) {
        simple!(self.detach_shader(program, shader))
    }

    fn link_program(&self, program: GLuint) {
        simple!(self.link_program(program))
    }

    fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        simple!(self.clear_color(r, g, b, a))
    }

    fn clear(&self, buffer_mask: GLbitfield) {
        simple!(self.clear(buffer_mask))
    }

    fn clear_depth(&self, depth: f64) {
        simple!(self.clear_depth(depth))
    }

    fn clear_stencil(&self, s: GLint) {
        simple!(self.clear_stencil(s))
    }

    fn flush(&self) {
        simple!(self.flush())
    }

    fn finish(&self) {
        simple!(self.finish())
    }

    fn get_error(&self) -> GLenum {
        no_side_effect!(self.get_error())
    }

    fn stencil_mask(&self, mask: GLuint) {
        unimplemented!("stencil_mask");
    }

    fn stencil_mask_separate(&self, face: GLenum, mask: GLuint) {
        unimplemented!("stencil_mask_separate");
    }

    fn stencil_func(&self, func: GLenum, ref_: GLint, mask: GLuint) {
        unimplemented!("stencil_func");
    }

    fn stencil_func_separate(&self, face: GLenum, func: GLenum, ref_: GLint, mask: GLuint) {
        unimplemented!("stencil_func_separate");
    }

    fn stencil_op(&self, sfail: GLenum, dpfail: GLenum, dppass: GLenum) {
        unimplemented!("stencil_op");
    }

    fn stencil_op_separate(&self, face: GLenum, sfail: GLenum, dpfail: GLenum, dppass: GLenum) {
        unimplemented!("stencil_op_separate");
    }

    fn egl_image_target_texture2d_oes(&self, target: GLenum, image: GLeglImageOES) {
        unimplemented!("egl_image_target_texture2d_oes");
    }

    fn egl_image_target_renderbuffer_storage_oes(&self, target: GLenum, image: GLeglImageOES) {
        unimplemented!("egl_image_target_renderbuffer_storage_oes");
    }

    fn generate_mipmap(&self, target: GLenum) {
        simple!(self.generate_mipmap(target))
    }

    fn insert_event_marker_ext(&self, message: &str) {
        unimplemented!("insert_event_marker_ext");
    }

    fn push_group_marker_ext(&self, message: &str) {
        unimplemented!("push_group_marker_ext");
    }

    fn pop_group_marker_ext(&self) {
        unimplemented!("pop_group_marker_ext");
    }

    fn debug_message_insert_khr(
        &self,
        source: GLenum,
        type_: GLenum,
        id: GLuint,
        severity: GLenum,
        message: &str,
    ) {
        unimplemented!("debug_message_insert_khr");
    }

    fn push_debug_group_khr(&self, source: GLenum, id: GLuint, message: &str) {
        unimplemented!("push_debug_group_khr");
    }

    fn pop_debug_group_khr(&self) {
        unimplemented!("pop_debug_group_khr");
    }

    fn fence_sync(&self, condition: GLenum, flags: GLbitfield) -> GLsync {
        unimplemented!("fence_sync");
    }

    fn client_wait_sync(&self, sync: GLsync, flags: GLbitfield, timeout: GLuint64) {
        unimplemented!("client_wait_sync");
    }

    fn wait_sync(&self, sync: GLsync, flags: GLbitfield, timeout: GLuint64) {
        unimplemented!("wait_sync");
    }

    fn delete_sync(&self, sync: GLsync) {
        unimplemented!("delete_sync");
    }

    fn texture_range_apple(&self, target: GLenum, data: &[u8]) {
        unimplemented!("texture_range_apple");
    }

    fn gen_fences_apple(&self, n: GLsizei) -> Vec<GLuint> {
        unimplemented!("gen_fences_apple");
    }

    fn delete_fences_apple(&self, fences: &[GLuint]) {
        unimplemented!("delete_fences_apple");
    }

    fn set_fence_apple(&self, fence: GLuint) {
        unimplemented!("set_fence_apple");
    }

    fn finish_fence_apple(&self, fence: GLuint) {
        unimplemented!("finish_fence_apple");
    }

    fn test_fence_apple(&self, fence: GLuint) {
        unimplemented!("test_fence_apple");
    }

    fn test_object_apple(&self, object: GLenum, name: GLuint) -> GLboolean {
        unimplemented!("test_object_apple");
    }

    fn finish_object_apple(&self, object: GLenum, name: GLuint) {
        unimplemented!("finish_object_apple");
    }

    // GL_KHR_blend_equation_advanced
    fn blend_barrier_khr(&self) {
        unimplemented!("blend_barrier_khr");
    }

    // GL_ARB_blend_func_extended
    fn bind_frag_data_location_indexed(
        &self,
        program: GLuint,
        color_number: GLuint,
        index: GLuint,
        name: &str,
    ) {
        unimplemented!("bind_frag_data_location_indexed");
    }

    fn get_frag_data_index(&self, program: GLuint, name: &str) -> GLint {
        unimplemented!("get_frag_data_index");
    }

    // GL_KHR_debug
    fn get_debug_messages(&self) -> Vec<DebugMessage> {
        unimplemented!("get_debug_messages");
    }

    // GL_ANGLE_provoking_vertex
    fn provoking_vertex_angle(&self, mode: GLenum) {
        unimplemented!("provoking_vertex_angle");
    }

    // GL_CHROMIUM_copy_texture
    fn copy_texture_chromium(
        &self,
        source_id: GLuint,
        source_level: GLint,
        dest_target: GLenum,
        dest_id: GLuint,
        dest_level: GLint,
        internal_format: GLint,
        dest_type: GLenum,
        unpack_flip_y: GLboolean,
        unpack_premultiply_alpha: GLboolean,
        unpack_unmultiply_alpha: GLboolean,
    ) {
        unimplemented!("copy_texture_chromium");
    }

    fn copy_sub_texture_chromium(
        &self,
        source_id: GLuint,
        source_level: GLint,
        dest_target: GLenum,
        dest_id: GLuint,
        dest_level: GLint,
        x_offset: GLint,
        y_offset: GLint,
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
        unpack_flip_y: GLboolean,
        unpack_premultiply_alpha: GLboolean,
        unpack_unmultiply_alpha: GLboolean,
    ) {
        unimplemented!("copy_sub_texture_chromium");
    }

    // GL_ANGLE_copy_texture_3d
    fn copy_texture_3d_angle(
        &self,
        source_id: GLuint,
        source_level: GLint,
        dest_target: GLenum,
        dest_id: GLuint,
        dest_level: GLint,
        internal_format: GLint,
        dest_type: GLenum,
        unpack_flip_y: GLboolean,
        unpack_premultiply_alpha: GLboolean,
        unpack_unmultiply_alpha: GLboolean,
    ) {
        unimplemented!("copy_texture_3d_angle");
    }

    fn copy_sub_texture_3d_angle(
        &self,
        source_id: GLuint,
        source_level: GLint,
        dest_target: GLenum,
        dest_id: GLuint,
        dest_level: GLint,
        x_offset: GLint,
        y_offset: GLint,
        z_offset: GLint,
        x: GLint,
        y: GLint,
        z: GLint,
        width: GLsizei,
        height: GLsizei,
        depth: GLsizei,
        unpack_flip_y: GLboolean,
        unpack_premultiply_alpha: GLboolean,
        unpack_unmultiply_alpha: GLboolean,
    ) {
        unimplemented!("copy_sub_texture_3d_angle");
    }
}
