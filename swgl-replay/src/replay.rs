use gleam::gl::GLuint;

use crate::Call;

use std::collections::HashMap;
use std::ffi::c_void;

pub struct ReplayState {
    swgl: swgl::Context,
    borrowed_buffers: HashMap<GLuint, Vec<u8>>,
    default_frame_buffer: Option<Vec<u8>>,
}

impl ReplayState {
    pub fn from_swgl(swgl: swgl::Context) -> ReplayState {
        ReplayState {
            swgl,
            borrowed_buffers: HashMap::new(),
            default_frame_buffer: None,
        }
    }

    pub fn into_swgl(self) -> swgl::Context {
        self.swgl
    }

    pub fn replay(&mut self, calls: &[Call], variable: &[u8]) {
        for (serial, call) in calls.iter().enumerate() {
            self.replay_one(call, variable, serial);
        }
    }

    #[allow(unused_variables)]
    fn replay_one(&mut self, call: &Call, variable: &[u8], serial: usize) {
        let call = *call;
        use Call::*;
        match call {
            note(..) => (),
            fingerprint(expected) => {
                let actual = crate::fingerprinter::fingerprint(&self.swgl);
                if expected != actual {
                    panic!("SWGL fingerprints diverged by serial {}", serial);
                }
            }
            gl(gl_call) => gl_replay::replay_one(&self.swgl, &gl_call, variable, serial),
            init_default_framebuffer { width, height, stride, buf } => {
                let buf: Option<Vec<u8>> = gl_replay::replay::get_parameter(buf, variable);
                let buf = match buf {
                    None => {
                        self.default_frame_buffer = None;
                        std::ptr::null_mut()
                    }
                    Some(mut vec) => {
                        let buf = vec.as_mut_ptr() as *mut u8 as *mut c_void;
                        self.default_frame_buffer = Some(vec);
                        buf
                    }
                };
                self.swgl.init_default_framebuffer(width, height, stride, buf)
            }
            get_color_buffer {
                fbo,
                flush,
                returned: expected,
            } => {
                //(Var<Seq<u32>>, i32, i32),
                let expected_buf = {
                    let (buf, _width, height, stride) = expected;
                    let buf: &[u32] = gl_replay::replay::get_parameter(buf, variable);
                    assert!(buf.len() != stride as usize * height as usize);
                };
                let actual = self.swgl.get_color_buffer(fbo, flush);
                let actual_buf = {
                    let (buf, _width, height, stride) = actual;
                    unsafe { std::slice::from_raw_parts(buf, stride as usize * height as usize) };
                };
                if (expected_buf, expected.1, expected.2, expected.3) !=
                    (actual_buf, actual.1, actual.2, actual.3)
                {
                    panic!("get_color_buffer return value doesn't match expectations");
                }
            }
            set_texture_buffer {
                tex,
                internal_format,
                width,
                height,
                stride,
                buf,
                min_width,
                min_height,
            } => {
                let buf: Option<Vec<u8>> = gl_replay::replay::get_parameter(buf, variable);
                let buf = match buf {
                    None => {
                        self.borrowed_buffers.remove(&tex);
                        std::ptr::null_mut()
                    }
                    Some(mut vec) => {
                        let buf = vec.as_mut_ptr() as *mut u8 as *mut c_void;
                        self.borrowed_buffers.insert(tex, vec);
                        buf
                    }
                };
                self.swgl.set_texture_buffer(
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

            composite {
                src_id,     // : GLuint,
                src_x,      // : GLint,
                src_y,      // : GLint,
                src_width,  // : GLsizei,
                src_height, // : GLint,
                dst_x,      // : GLint,
                dst_y,      // : GLint,
                opaque,     // : bool,
                flip,       // : bool,
            } => self.swgl.composite(
                src_id, src_x, src_y, src_width, src_height, dst_x, dst_y, opaque, flip,
            ),
        }
    }
}
