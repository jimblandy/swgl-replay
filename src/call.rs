//! The `swgl_replay::Call` type.

use gl_replay::form::{Seq, Var};
use gl_replay::raw;
use gleam::gl::{GLenum, GLint, GLsizei, GLuint};

unsafe impl raw::Simple for Call {}

impl From<gl_replay::Call> for Call {
    fn from(gl_call: gl_replay::Call) -> Call {
        Call::gl(gl_call)
    }
}

/// A call to a `swgl::Context` method.
///
/// Each variant of this enum represents a call to a method of either the
/// `gleam::Gl` trait, or the `Swgl` trait of this crate, covering all
/// recordable actions on a `swgl::Context` value.
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub enum Call {
    gl(gl_replay::Call),
    init_default_framebuffer {
        width: i32,
        height: i32,
    },
    get_color_buffer {
        fbo: GLuint,
        flush: bool,
        returned: (Var<Seq<u32>>, i32, i32),
    },
    set_texture_buffer {
        tex: GLuint,
        internal_format: GLenum,
        width: GLsizei,
        height: GLsizei,
        buf: Option<Var<Seq<u8>>>,
        min_width: GLsizei,
        min_height: GLsizei,
    },

    composite {
        src_id: GLuint,
        src_x: GLint,
        src_y: GLint,
        src_width: GLsizei,
        src_height: GLint,
        dst_x: GLint,
        dst_y: GLint,
        opaque: bool,
        flip: bool,
    },
}
