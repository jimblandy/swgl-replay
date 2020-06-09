//! Implementation of `Gl` trait for `Recorder`.

use gleam::gl::Gl;
use crate::call::Call;
use crate::var::CallStream;

mod impl_gl;
mod parameter;

/// A trait for types that can record a `Gl` method call stream.
///
/// 
/// Usually, it's not enough to just record `Gl` calls: you have to also capture
/// other peripheral operations that affect the OpenGL implementation you want to
/// drive. 
pub trait Record {
    type GlImpl: Gl;
    type CallStreamImpl: CallStream<Call>;

    /// Return this `Recorder`'s OpenGL implementation.
    fn as_gl(&self) -> &Self::GlImpl;

    /// Obtain this `Recorder`'s `CallStream` implementation.
    ///
    /// Since this takes `&self` but returns `&mut CallStreamImpl`,
    /// implementations will probably need a `Mutex` or a `RefCell` somewhere.
    fn as_call_stream(&self) -> &mut Self::CallStreamImpl;
}

/// An implementation of `Gl` that records all method calls, given an
/// implementation of `Record` that does all the actual work.
///
/// This is an olive branch offered to Rust's orphan impl rules.
pub struct Recorder<R: Record>(R);
