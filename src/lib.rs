//! Record and replay for [`gleam::Gl`] implementations.
//!
//! This crate provides a wrapper around an OpenGL context that records all the
//! method calls performed on it and writes them to a set of files on disk. This
//! recording can then be replayed against another OpenGL context later, for
//! debugging or benchmarking.
//!
//! Specifically, this crate's [`Recorder`] type implements the [`gleam`]
//! crate's [`Gl`] trait by wrapping some other `Gl` implementation that you
//! provide, and logging details of all the method calls performed on the
//! `Recorder` to the filesystem.
//!
//! Then, the `replay` function takes a reference to the contents of a saved
//! recording, and performs the same series of calls on a new `Gl`
//! implementation you provide.
//!
//! On the filesystem, a recording is actually a directory, containing a number
//! of files. The `calls` file holds an array of fixed-size entries describing
//! the method calls, and the `large` file holds values that were too large to
//! include in the array.
//!
//! You can combine this crates' recordings with other events of your choice.
//! The `Recorder` type can use any implementation of the `Serializer` trait to
//! record calls, so you can provide your own `Serializer` implementation that
//! combines the `Gl` calls with your own data. Then, at replay time, there is a
//! `replay_one` function that replays a single `Gl` call, which you can use
//! from your own replay loop. This crate also exposes simple `Serializer` and
//! `Deserializer` traits which you can use if they meet your needs.
//!
//! [`gleam`]: https://crates.io/crates/gleam
//! [`gleam::Gl`]: https://docs.rs/gleam/0.11.0/gleam/gl/trait.Gl.html
//! [`Gl`]: https://docs.rs/gleam/0.11.0/gleam/gl/trait.Gl.html
//! [`Recorder`]: struct.Recorder.html
//! [`Replayer`]: struct.Replayer.html

mod call;
pub use call::Call;

mod file_stream;
pub use file_stream::{FileStream, FileRecording};

pub mod form;
mod recorder;
pub mod raw;
pub mod var;



//mod replay;
//pub use replay::{replay, replay_one};
