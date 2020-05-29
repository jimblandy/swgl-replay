# Record and replay for `gleam::Gl` implementations

**Experimental work in progress.**

This crate defines a wrapper for implementations of the `gleam` crate's `Gl`
trait that records all calls to disk, and then can replay them. This might be
useful for debugging and profiling the `Gl` implementation.
