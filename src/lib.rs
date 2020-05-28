//! Record and replay for [`gleam::Gl`] implementations.
//!
//! This crate provides a wrapper around an OpenGL context that records the
//! series of method calls it receives to a set of files on disk. Then, the
//! recording can be replayed against another OpenGL context later, for
//! debugging or benchmarking.
//!
//! Specifically, this crate's [`Recorder`] type implements the [`gleam`]
//! crate's [`Gl`] trait, by wrapping some other `Gl` implementation that you
//! provide. Details of all the method calls performed on the `Recorder` are
//! written to the filesystem.
//!
//! A `Replayer` value refers to one of these saved recordings on the
//! filesystem. You can pass a `Gl` implementation to a `Recording`, and have it
//! perform the same series of method calls on it as before.
//!
//! On the filesystem, a recording is actually a directory, containing a number
//! of files. The `calls` file holds an array of fixed-size entries describing
//! the method calls, and the `large` file holds values that were too large to
//! include in the array.
//!
//! [`gleam`]: https://crates.io/crates/gleam
//! [`gleam::Gl`]: https://docs.rs/gleam/0.11.0/gleam/gl/trait.Gl.html
//! [`Gl`]: https://docs.rs/gleam/0.11.0/gleam/gl/trait.Gl.html
//! [`Recorder`]: struct.Recorder.html
//! [`Replayer`]: struct.Replayer.html

use gleam::gl::{GLsizeiptr, GLvoid, Gl};
use std::ops::Deref;
use std::path::Path;
use std::{io, sync};

mod call;
mod files;
mod raw;
mod recorder_impl;

pub use call::{BufFromGl, BufToGl, Call};
pub use files::Files;

/// A trait for types that can serialize GL method call streams.
pub trait Serializer {
    type Error: std::error::Error;

    /// Write the method call `call`.
    fn write_call(&mut self, call: &call::Call) -> Result<(), Self::Error>;

    /// Write the contents of the buffer `buf`, and return an identifier for it.
    fn write_buffer(&mut self, buf: &[u8]) -> Result<usize, Self::Error>;

    /// Write the contents of the buffers `bufs`, and return an identifier for them.
    fn write_buffers(&mut self, bufs: &[&[u8]]) -> Result<usize, Self::Error>;

    /// Flush buffers, if any.
    fn flush(&mut self) -> Result<(), Self::Error>;
}

pub struct Recorder<G, S> {
    inner_gl: G,
    locked: sync::Mutex<Locked<S>>,
}

pub(crate) struct Locked<S> {
    serializer: S,
}

impl<G, S> Recorder<G, S> {
    /// Return a new `Recorder` for `inner_gl`, using `serializer`.
    ///
    /// The `inner_gl` argument must be a pointer to a value that implements
    /// [`gleam::Gl`], like `Rc<dyn Gl>`. (Specifically, `inner_gl` must
    /// implement `Deref<Target=Gl>`.)
    ///
    /// [`gleam::Gl`]: https://docs.rs/gleam/0.11.0/gleam/gl/trait.Gl.html
    pub fn with_serializer(inner_gl: G, serializer: S) -> Recorder<G, S>
    where
        S: Serializer,
        G: Deref,
        G::Target: Gl,
    {
        let locked = Locked::new(serializer);

        Recorder {
            inner_gl,
            locked: sync::Mutex::new(locked),
        }
    }

    /// Record a new `Recorder` for `inner_gl`, logging calls to disk.
    ///
    /// The `path` argument is the name of a directory to create to hold the
    /// various recording files.
    pub fn to_file<P>(inner_gl: G, path: P) -> io::Result<Recorder<G, Files>>
    where
        P: AsRef<Path>,
        G: Deref,
        G::Target: Gl,
    {
        Ok(Recorder::with_serializer(inner_gl, Files::create(path)?))
    }

    pub fn inner_gl(&self) -> &G {
        &self.inner_gl
    }
}

impl<S> Locked<S> {
    fn new(serializer: S) -> Locked<S> {
        Locked { serializer }
    }
}

impl<S: Serializer> Locked<S> {
    pub(crate) fn write_call(&mut self, call: &call::Call) -> Result<(), S::Error>
    {
        self.serializer.write_call(call)
    }

    pub(crate) fn write_slice<T: Copy>(&mut self, slice: &[T]) -> Result<usize, S::Error>
    {
        self.serializer.write_buffer(raw::slice_as_bytes(slice))
    }

    pub(crate) fn write_str(&mut self, s: &str) -> Result<usize, S::Error>
    {
        self.write_slice(s.as_bytes())
    }

    pub(crate) fn write_buffers(&mut self, bufs: &[&[u8]]) -> Result<call::BufToGl, S::Error>
    {
        Ok(call::BufToGl(self.serializer.write_buffers(bufs)?))
    }

    pub(crate) fn write_gl_buffer(
        &mut self,
        data: *const GLvoid,
        size: GLsizeiptr,
    ) -> Result<call::BufToGl, S::Error>
    {
        let scope = ();
        let buf = unsafe { raw::slice_from_gl_buffer(&scope, data, size) };
        Ok(call::BufToGl(self.write_slice(buf)?))
    }
}

impl<G, S> Recorder<UnboxedGl<G>, S> {
    /// Return a new `Recorder` for `inner_gl`, using `serializer`.
    ///
    /// Whereas `with_serializer` requires a pointer to a `Gl` implementation,
    /// this constructor takes an `inner_gl` value that implements `Gl`
    /// directly.
    pub fn with_serializer_unboxed(inner_gl: G, serializer: S) -> Recorder<UnboxedGl<G>, S>
    where
        S: Serializer,
        G: Gl,
    {
        Recorder::with_serializer(UnboxedGl(inner_gl), serializer)
    }
}

impl<G> Recorder<UnboxedGl<G>, Files> {
    pub fn to_file_unboxed<P>(inner_gl: G, path: P) -> io::Result<Recorder<UnboxedGl<G>, Files>>
    where
        P: AsRef<Path>,
        G: Gl,
    {
        Ok(Recorder::with_serializer_unboxed(
            inner_gl,
            Files::create(path)?,
        ))
    }
}

pub struct UnboxedGl<T>(T);

impl<T> Deref for UnboxedGl<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}
