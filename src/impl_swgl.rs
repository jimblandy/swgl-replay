use gl_replay::{CallStream, Stream, Parameter};
use gleam::gl::*;
use std::os::raw::c_void;

use crate::call::Call;
use crate::dyn_swgl::Swgl;
use crate::InnerRecorder;

macro_rules! check {
    ($call:expr) => {
        $call.expect("swgl-replay serialization failure")
    };
}

/// General form of a recorded call. Always makes the call, and returns its value.
macro_rules! general {
    (
        let $returned:ident = $self:ident . $method:ident ( $( $arg:ident ),* );
        lock $call_stream:ident;
        $body:expr
    ) => {
        {
            let $returned = $self .inner_gl(). $method ( $( $arg ),* );
            let $call_stream = &mut *$self .lock_call_stream();

            $body;

            // For debugging.
            $call_stream .flush()
                .expect("gl-replay serialization failure");

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
impl Swgl for InnerRecorder {
    fn init_default_framebuffer(&self, width: i32, height: i32) {
        simple!(self.init_default_framebuffer(width, height))
    }
    fn get_color_buffer(&self, fbo: GLuint, flush: bool) -> (*mut c_void, i32, i32) {
        general!(let returned = self.get_color_buffer(fbo, flush);
                 lock call_stream;
                 {
                     let color_buffer = unsafe {
                         std::slice::from_raw_parts(returned.0 as *const u8,
                                                    returned.1 as usize * returned.2 as usize * 4)
                     };
                     let var = check!(color_buffer.to_call(call_stream));
                     let call = Call::get_color_buffer {
                         fbo,
                         flush,
                         returned: (var, returned.1, returned.2)
                     };
                     check!(call_stream.write_call(call));
                }
        )
    }

    fn set_texture_buffer(
        &self,
        tex: GLuint,
        internal_format: GLenum,
        width: GLsizei,
        height: GLsizei,
        buf: *mut c_void,
        min_width: GLsizei,
        min_height: GLsizei,
    ) {
        general!(let returned = self.set_texture_buffer(tex, internal_format, width, height,
                                                        buf, min_width, min_height);

                 lock call_stream;
                 {
                     let color_buffer = if buf.is_null() {
                         None
                     } else {
                         let bytes_per_pixel = swgl::Context::bytes_for_internal_format(internal_format);
                         let size = bytes_per_pixel * width as usize * height as usize;
                         Some(unsafe {
                             std::slice::from_raw_parts(buf as *const u8, size)
                         })
                     };
                     let var = check!(color_buffer.to_call(call_stream));
                     let call = Call::set_texture_buffer {
                         tex,
                         internal_format,
                         width,
                         height,
                         buf: var,
                         min_width,
                         min_height,
                     };
                     check!(call_stream.write_call(call));
                }
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
        simple!(self.composite(src_id, src_x, src_y, src_width, src_height, dst_x, dst_y, opaque, flip))
    }
}
