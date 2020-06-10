//! Implementation of `Gl` trait for `Recorder`.

use std::sync;

mod impl_gl;
mod parameter;

/// An implementation of `gleam::Gl` that records method calls for later replay.
pub struct Recorder<G, Cs> {
    /// The Gl implementation calls to which we are recording.
    inner_gl: G,

    /// The CallStream to which we record calls to `inner_gl`.
    call_stream: sync::Mutex<Cs>,
}

impl<G, Cs> Recorder<G, Cs> {
    pub fn new(inner_gl: G, call_stream: Cs) -> Recorder<G, Cs> {
        Recorder {
            inner_gl,
            call_stream: sync::Mutex::new(call_stream)
        }
    }

    pub fn inner_gl(&self) -> &G {
        &self.inner_gl
    }
}
