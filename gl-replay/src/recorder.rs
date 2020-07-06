//! Implementation of `Gl` trait for `Recorder`.

use std::sync;

mod impl_gl;

/// An implementation of `gleam::Gl` that records method calls for later replay.
pub struct Recorder<G, Cs> {
    /// The Gl implementation calls to which we are recording.
    inner_gl: G,

    /// The CallStream to which we record calls to `inner_gl`.
    // I assume complex recording designs will have a single call stream shared
    // by various recorders, and implementing `CallStream<T>` for various call
    // types T. In that case, it would make sense for this to be an
    // `Arc<Mutex<Cs>>`. But we don't need that for SWGL.
    call_stream: sync::Mutex<Cs>,

    fingerprinter: Option<fn(&G, &mut Cs)>,
}

impl<G, Cs> Recorder<G, Cs> {
    pub fn new(inner_gl: G, call_stream: Cs) -> Recorder<G, Cs> {
        Recorder {
            inner_gl,
            call_stream: sync::Mutex::new(call_stream),
            fingerprinter: None
        }
    }

    pub fn with_fingerprinter(self, fingerprinter: fn(&G, &mut Cs)) -> Self {
        Recorder {
            fingerprinter: Some(fingerprinter),
            .. self
        }
    }

    pub fn inner_gl(&self) -> &G {
        &self.inner_gl
    }

    pub fn lock_call_stream(&self) -> sync::MutexGuard<Cs> {
        self.call_stream.lock().unwrap()
    }
}
