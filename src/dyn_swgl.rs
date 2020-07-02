//! The `Swgl` trait.

use gleam::gl::{GLenum, GLint, GLsizei, GLuint, Gl};
use std::ffi::c_void;

/// A trait that covers all public methods of `swgl::Context`.
pub trait Swgl: Gl {
    fn init_default_framebuffer(&self, width: i32, height: i32, stride: i32, buf: *mut c_void);
    fn get_color_buffer(&self, fbo: GLuint, flush: bool) -> (*mut c_void, i32, i32, i32);

    fn set_texture_buffer(
        &self,
        tex: GLuint,
        internal_format: GLenum,
        width: GLsizei,
        height: GLsizei,
        stride: GLsizei,
        buf: *mut c_void,
        min_width: GLsizei,
        min_height: GLsizei,
    );

    fn composite(
        &self,
        src_id: GLuint,
        src_x: GLint,
        src_y: GLint,
        src_width: GLsizei,
        src_height: GLint,
        dst_x: GLint,
        dst_y: GLint,
        opaque: bool,
        flip: bool,
    );
}

impl Swgl for swgl::Context {
    fn init_default_framebuffer(&self, width: i32, height: i32, stride: i32, buf: *mut c_void) {
        self.init_default_framebuffer(width, height, stride, buf)
    }

    fn get_color_buffer(&self, fbo: GLuint, flush: bool) -> (*mut c_void, i32, i32, i32) {
        self.get_color_buffer(fbo, flush)
    }

    fn set_texture_buffer(
        &self,
        tex: GLuint,
        internal_format: GLenum,
        width: GLsizei,
        height: GLsizei,
        stride: GLsizei,
        buf: *mut c_void,
        min_width: GLsizei,
        min_height: GLsizei,
    ) {
        self.set_texture_buffer(
            tex,
            internal_format,
            width,
            height,
            stride,
            buf,
            min_width,
            min_height,
        )
    }

    fn composite(
        &self,
        src_id: GLuint,
        src_x: GLint,
        src_y: GLint,
        src_width: GLsizei,
        src_height: GLint,
        dst_x: GLint,
        dst_y: GLint,
        opaque: bool,
        flip: bool,
    ) {
        self.composite(
            src_id, src_x, src_y, src_width, src_height, dst_x, dst_y, opaque, flip,
        )
    }
}
