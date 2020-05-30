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

use gleam::gl::Gl;
use std::ops::Deref;
use std::path::Path;
use std::{io, sync};

mod call;
mod files;
mod raw;
mod recorder_gl;
mod replay;
mod serialize;

pub use call::{BufFromGl, BufToGl, Call};
pub use files::Files;
pub use replay::replay;
pub use serialize::{Serialize, Serializer};

pub struct Recorder<G, S> {
    inner_gl: G,
    inner_recorder: sync::Mutex<InnerRecorder<S>>,
}

pub(crate) struct InnerRecorder<S> {
    serializer: S,
}

impl<G, S> Recorder<G, S> {
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
        let inner_recorder = InnerRecorder::new(serializer);

        Recorder {
            inner_gl,
            inner_recorder: sync::Mutex::new(inner_recorder),
        }
    }

    pub fn inner_gl(&self) -> &G {
        &self.inner_gl
    }
}

impl<S> InnerRecorder<S> {
    fn new(serializer: S) -> InnerRecorder<S> {
        InnerRecorder { serializer }
    }
}

impl<S: Serializer> InnerRecorder<S> {
    pub(crate) fn write_call(&mut self, call: &call::Call) -> Result<(), S::Error> {
        self.serializer.write_call(call)
    }

    pub(crate) fn write_variable<T: Serialize + ?Sized>(
        &mut self,
        var: &T,
    ) -> Result<usize, S::Error> {
        let ident = self.serializer.variable_size();
        var.write(&mut self.serializer)?;
        Ok(ident)
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
