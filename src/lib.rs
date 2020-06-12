//! Record and replay for SWGL, Firefox's software GL implementation.
//!
//! This crate's `Recorder` type wraps a `swgl::Context`, and then implements
//! `gleam::Gl` and the compositing functions on `swgl::Context`, and records
//! all method calls to a file. You can then replay a recording on a SWGL
//! context with this crate's `replay` function.
//!
//! There are a few inherent methods on `swgl::Context` that wrench and other
//! clients call directly, and which must be included in recordings for replay
//! to succeed. For those methods, this crate defines a `Swgl` trait,
//! with implementations for both `swgl::Context` and `Recorder`. Clients who
//! want to optionally record SWGL calls can then use a `&dyn Swgl`
//! value, and select which implementation it borrows at run time. `Swgl`
//! extends `gleam::Gl` and `webrender::Compositor`, so it should be sufficient
//! for everything the client needs.

use std::io;
use std::path::Path;

mod call;
mod dyn_swgl;
mod impl_swgl;
mod replay;

pub use call::Call as Call;
pub use dyn_swgl::Swgl;
use gl_replay::Recorder;
pub use replay::ReplayState;

/// A `FileStream` for both SWGL and OpenGL calls.
pub type FileStream = gl_replay::FileStream<Call>;

/// A `gl_replay` recorder for SWGL ncalls.
pub type InnerRecorder = Recorder<swgl::Context, gl_replay::FileStream<Call>>;

/// A wrapper for `swgl::Context` that records all method calls to files.
pub struct FileRecorder(InnerRecorder);

impl FileRecorder {
    /// Create a new SWGL recorder that logs all method cals on
    /// `inner_swgl` to a recording saved as a directory named `dir`.
    pub fn create<P: AsRef<Path>>(inner_swgl: swgl::Context, dir: P) -> io::Result<FileRecorder> {
        let file_stream = FileStream::create(dir, SWGR_MAGIC)?;
        Ok(FileRecorder(Recorder::new(inner_swgl, file_stream)))
    }
}

impl gleam::gl::AsGl for FileRecorder {
    type Impl = InnerRecorder;
    fn as_gl(&self) -> &InnerRecorder {
        &self.0
    }
}

/// The magic number used to identify `gleam::Gl` file recordings.
pub const SWGR_MAGIC: u32 = (((b'S' as u32) << 8 | (b'W' as u32)) << 8 | (b'G' as u32)) << 8 | (b'R' as u32);

pub type FileRecording = gl_replay::FileRecording<Call>;
