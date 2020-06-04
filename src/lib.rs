//! Record and replay for SWGL, Firefox's software GL implementation.
//!
//! This crate's `Recorder` type wraps a `swgl::Context`, and then implements
//! `gleam::Gl` and `webrender::Compositor`, and records all method calls to a
//! file. You can then replay a recording on a SWGL context with this crate's
//! `replay` function.
//!
//! There are a few inherent methods on `swgl::Context` that wrench and other
//! clients call directly, and which must be included in recordings for replay
//! to succeed. For those methods, this crate defines a `Swgl` trait,
//! with implementations for both `swgl::Context` and `Recorder`. Clients who
//! want to optionally record SWGL calls can then use a `&dyn Swgl`
//! value, and select which implementation it borrows at run time. `Swgl`
//! extends `gleam::Gl` and `webrender::Compositor`, so it should be sufficient
//! for everything the client needs.

mod dyn_swgl;
mod files;

pub use dyn_swgl::Swgl;
pub use files::{Files, Recording};
