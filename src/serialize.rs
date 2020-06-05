//! Custom Serialize and Deserialize traits.
//!
//! This module defines simplified variants of the serde serialization and
//! deserialization traits that permit a bit more zero-copy access (and thus,
//! hopefully, impose a bit less overhead). Serde will only borrow `str` and
//! `[u8]` directly out of the serialized data, whereas this module's traits
//! will also borrow `[GLfloat]` and things like that. Also, the breadth of
//! types needed to record a `Gl` session are pretty limited, so we can avoid
//! some of serde's complications.
//!
//! A GL recording includes two streams of serialized data:
//!
//! - There is a stream of fixed-size `Call` values. The `Call` enum identifies
//!   the `Gl` method being invoked, and can hold arguments to that method if
//!   they are integers, floats, or similarly simple-minded types. But `Call` is
//!   `Copy + 'static`, so it can't own or borrow any recording of variable-
//!   length buffers being passed to and from Gl.
//!
//! - Thus, a second stream of variable-length values holds buffers, strings,
//!   and the like. The `Call` then stores the offset at which the associated
//!   data is stored in the variable-length stream. (Also, some methods take so
//!   many arguments that the `Call` enum would become quite large if it stored
//!   them directly, so instead we stash their arguments in the variable-length
//!   stream as well.)
//!
//! To keep deserialization overhead down, we want to be able to borrow values
//! directly from the buffer of serialized data whenever possible. The `Call`
//! type is designed to be used in this way: the bytes of the call stream are
//! interpreted directly as a `[Call]` slice. (I wonder if touching the enum's
//! padding bytes when we write the stream out counts as UB.)
//!
//! Using the variable-length data is not so simple, however. Although we are
//! able to borrow slices of simple `Copy + 'static` types directly out of the
//! variable-length data, more complex types like `&[&[T]]` take more work.
//! That is serialized as:
//!
//!     <length of outer slice> ( <length of inner slice> ( <T value> ) * ) *
//!
//! where each 'length' is a `usize`. We deserialize this as a `Vec<&[T]>`,
//! where the `Vec` is produced element-by-element by iterating over the data,
//! and the `&[T]` slices borrow from the data. (The variable-length stream
//! includes padding before each value for alignment, not shown.)

use std::mem;

use crate::forms::{Seq, Str};
use crate::raw;

/// A trait for types that can serialize GL method call streams.
pub trait Serializer {
    type Error: std::error::Error;

    /// Write the contents of the buffer `buf` to the variable-length data
    /// stream.
    fn write_variable_unaligned(&mut self, buf: &[u8]) -> Result<(), Self::Error>;

    /// Write the contents of `slice` to the variable-length data stream,
    /// starting with padding as needed to align it properly for elements of
    /// type `T`. Return its post-alignment start position.
    fn write_variable_aligned<T: Copy + 'static>(&mut self, slice: &[T]) -> Result<usize, Self::Error> {
        let pos = self.variable_size();
        let align: usize = mem::align_of::<T>();

        let align_skip = (0 - pos) & (align - 1);
        if align_skip > 0 {
            static PADDING: [u8; 64] = [b'P'; 64];
            assert!(align_skip <= PADDING.len());
            self.write_variable_unaligned(&PADDING[..align_skip])?;
        }

        let pos = self.variable_size();
        self.write_variable_unaligned(raw::slice_as_bytes(slice))?;
        Ok(pos)
    }

    /// Return the number of bytes that have been written to the variable-length
    /// data stream so far.
    fn variable_size(&self) -> usize;

    /// Flush buffers, if any.
    fn flush(&mut self) -> Result<(), Self::Error>;
}

/// A type that can be serialized to a Serializer.

/// A type that can be deserialized from a block of bytes.
pub trait Deserialize<'b>: Sized {
    fn deserialize(buf: &mut &'b [u8]) -> Result<Self, DeserializeError>;
}

impl<'b, T: Deserialize<'b> + Copy + 'static> Deserialize<'b> for &'b [T] {
    fn deserialize(buf: &mut &'b [u8]) -> Result<&'b [T], DeserializeError> {
        let len: usize = Deserialize::deserialize(buf)?;
        take_slice(buf, len)
    }
}

impl<'b, T: Deserialize<'b>> Deserialize<'b> for Vec<T> {
    fn deserialize(buf: &mut &'b [u8]) -> Result<Vec<T>, DeserializeError> {
        let len: usize = Deserialize::deserialize(buf)?;
        let mut vec = Vec::new();
        for _ in 0..len {
            vec.push(Deserialize::deserialize(buf)?);
        }
        Ok(vec)
    }
}

impl<'b> Deserialize<'b> for &'b str {
    fn deserialize(buf: &mut &'b [u8]) -> Result<&'b str, DeserializeError> {
        let bytes: &[u8] = Deserialize::deserialize(buf)?;
        std::str::from_utf8(bytes).map_err(|_| DeserializeError::BadUTF8)
    }
}

macro_rules! simply_serialized_types {
    ( $( $type:ty ),* ) => {
        $(
            impl Serialize for $type {
                type Form = $type;

                fn write<S: Serializer>(&self, serializer: &mut S) -> Result<usize, S::Error> {
                    serializer.write_variable_aligned(std::slice::from_ref(self))
                }

                /// For these types, we can write out the whole block at once.
                fn write_seq<S: Serializer>(this: &[Self], serializer: &mut S) -> Result<usize, S::Error> {
                    let pos = serializer.write_variable_aligned(&[this.len()])?;
                    serializer.write_variable_aligned(this)?;
                    Ok(pos)
                }
            }

            impl<'b> Deserialize<'b> for $type {
                fn deserialize(buf: &mut &'b [u8]) -> Result<$type, DeserializeError> {
                    Ok(take_slice(buf, 1)?[0])
                }
            }
        )*
    }
}

// These types are serialized in variable content as themselves.
simply_serialized_types!(bool, u8, u32, i32, f32, f64, usize);
