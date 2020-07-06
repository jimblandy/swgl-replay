//! Serialization for variable-length values, zero-copy when possible.
//!
//! This module defines serialization and deserialization traits that permit a
//! bit more zero-copy access (and thus, hopefully, impose a bit less overhead).
//! Serde will only borrow `str` and `[u8]` directly out of the serialized data,
//! whereas this module's traits will also borrow `[GLfloat]` and things like
//! that.
//!
//! Fortunately, the breadth of types needed to record the sorts of sessions we
//! care about are pretty limited, so this module's traits can be much simpler
//! than serde's.
//!
//! Providing zero-copy access to types like `f32` entails serializing them in
//! their in-memory form, and ensuring proper alignment. This means that
//! recordings are specific to a particular endianness, word size, and
//! alignment.
//!
//! Array slices and vectors are serialized as a `usize`, followed by the
//! serialized forms of the elements. The `usize` and the elements are each
//! preceded by padding for alignment.
//!
//! String slices are serialized as a `usize` (preceded by padding), followed by
//! the slice's text as UTF-8.
//!
//! References are transparent to serialization: a reference value is simply
//! serialized the way its referent would be.

use crate::form::{Seq, Str};
use crate::raw;

use std::io;
use std::mem;

/// A byte stream which tracks the current byte offset.
///
/// Types that implement `Serialize` can write themselves to a data stream
/// that implements this trait.
pub trait MarkedWrite: io::Write {
    /// Return the current byte offset in the stream. This is the position
    /// that the next call to `write_all_marked` will return.
    fn mark(&self) -> usize;

    /// Write the contents of the buffer `buf` to the data stream. Return the
    /// byte offset of the start of the data within the stream.
    fn marked_write_all(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mark = self.mark();
        self.write_all(buf)?;
        Ok(mark)
    }

    /// Write the contents of `slice` to the variable-length data stream,
    /// starting with padding as needed to align it properly for elements of
    /// type `T`. Return its start position, after any padding.
    fn marked_write_all_aligned<T>(&mut self, slice: &[T]) -> io::Result<usize>
    where
        T: raw::Simple,
    {
        self.align_for::<T>()?;
        let pos = self.mark();
        self.marked_write_all(raw::slice_as_bytes(slice))?;
        Ok(pos)
    }

    /// Write padding bytes to align properly for values of type `T`.
    fn align_for<T>(&mut self) -> io::Result<()> {
        let padding_length = (0 - self.mark()) & (mem::align_of::<T>() - 1);
        if padding_length > 0 {
            static PADDING: [u8; 64] = [b'P'; 64];
            assert!(padding_length <= PADDING.len());
            self.marked_write_all(&PADDING[..padding_length])?;
        }
        Ok(())
    }
}

/// An extension of `MarkedWrite` which also writes a separate stream of `Call` values.
///
/// Note that `Call` here is a generic type parameter: this trait is not
/// specific to OpenGL calls or gleam. You can use this to record any stream of
/// calls for which you've prepared an enum type like this crate's `call::Call`.
///
/// Values written to `write_call` should be stored in a way that lets us obtain
/// a `&[Call]` slice from the data, with no per-element serialization needed.
/// The `Call` parameter should be something suitably simple (`Copy`, at least),
/// to make this possible.
pub trait CallStream<Call>: MarkedWrite {
    /// Append the contents of the buffer `buf` to the data stream. Return the
    /// serial number of the call just written.
    fn write_call(&mut self, call: Call) -> io::Result<usize>;

    /// Return the serial number of the next call to be written.
    /// For debugging.
    fn call_serial(&self) -> usize;
}

/// A type that can be serialized to a `var::MarkedWrite`.
pub trait Serialize {
    /// The form in which `Self` values are serialized, using the types from the
    /// `form` module.
    ///
    /// The `Form` for a base type like `i32` would simply be `i32`. Types like
    /// `&[f32]` and `Vec<f32>` would both have a `Form` of `Seq<f32>`.
    type Form;

    /// Serialize a single `Self` value. On success, return the byte offset it
    /// was written to in `stream`.
    fn serialize<S: MarkedWrite>(&self, stream: &mut S) -> io::Result<usize>;

    /// Serialize a `[Self]` slice in the `Seq` form.
    ///
    /// The default definition of this function simply uses a loop to write out
    /// each element. Implementations for types that can be written as a single
    /// block should override this to do so.
    fn serialize_seq<S: MarkedWrite>(seq: &[Self], stream: &mut S) -> io::Result<usize>
    where
        Self: Sized,
    {
        let pos = stream.marked_write_all_aligned(&[seq.len()])?;
        for elt in seq {
            elt.serialize(stream)?;
        }
        Ok(pos)
    }
}

/// A serialized form that can be deserialized to produce a value of type `T`.
///
/// This trait gets implemented for things like `Seq<T>` or `Str`, that
/// represent the serialization form. The actual Rust type that gets produced is
/// the trait's type parameter. This lets us have `Seq<u8>` implement both
/// `DeserializeAs<'b, &[u8]>` and `DeserializeAs<'b, Vec<u8>>`.
///
/// The `'b` lifetime is that of the buffer we deserialize from. This allows
/// implementations to return types that borrow from the buffer.
pub trait DeserializeAs<'b, T> {
    /// Extract a value of type `T` from `buf`, according to the serialization
    /// form `self`. Adjust `buf` to enclose the unconsumed following bytes.
    fn deserialize(buf: &mut &'b [u8]) -> Result<T, DeserializeError>;
}

macro_rules! implement_serialize_for_simple {
    ( $( $type:ty ),* ) => {
        $(
            /// Simple types are serialized as their in-memory form.
            impl Serialize for $type {
                type Form = $type;
                fn serialize<S: MarkedWrite>(&self, stream: &mut S) -> io::Result<usize> {
                    stream.marked_write_all_aligned(std::slice::from_ref(self))
                }

                /// Slices of simple types can be handled with a single write.
                fn serialize_seq<S: MarkedWrite>(seq: &[Self], stream: &mut S) -> io::Result<usize>
                where
                    Self: Sized,
                {
                    let pos = seq.len().serialize(stream)?;
                    stream.marked_write_all_aligned(seq)?;
                    Ok(pos)
                }
            }
        )*
    }
}

implement_serialize_for_simple!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64, char, bool
);

impl<'b, T: raw::Simple + 'b> DeserializeAs<'b, T> for T {
    fn deserialize(buf: &mut &'b [u8]) -> Result<T, DeserializeError> {
        Ok(borrow_aligned_slice(buf, 1)?[0])
    }
}

/// Slices are serialized using the `Seq` form: the length as a `usize`,
/// followed by the elements, all padded as necessary for alignment.
impl<T: Serialize> Serialize for [T] {
    type Form = Seq<T::Form>;
    fn serialize<S: MarkedWrite>(&self, stream: &mut S) -> io::Result<usize> {
        // Let the element type choose how to write the slice.
        <T as Serialize>::serialize_seq(self, stream)
    }
}

impl<'b, T: raw::Simple> DeserializeAs<'b, &'b [T]> for Seq<T> {
    fn deserialize(buf: &mut &'b [u8]) -> Result<&'b [T], DeserializeError> {
        let len = usize::deserialize(buf)?;
        borrow_aligned_slice(buf, len)
    }
}

/// References are transparent to serialization: `&T` is serialized just like `T`.
impl<T: Serialize + ?Sized> Serialize for &T {
    type Form = T::Form;
    fn serialize<S: MarkedWrite>(&self, stream: &mut S) -> io::Result<usize> {
        (*self).serialize(stream)
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    type Form = Seq<T::Form>;
    fn serialize<S: MarkedWrite>(&self, stream: &mut S) -> io::Result<usize> {
        // Let the element type choose how to write the slice.
        <T as Serialize>::serialize_seq(self, stream)
    }
}

impl<'b, F, T> DeserializeAs<'b, Vec<T>> for Seq<F>
where
    F: DeserializeAs<'b, T>,
{
    fn deserialize(buf: &mut &'b [u8]) -> Result<Vec<T>, DeserializeError> {
        // I thought it would be cool if we could grab a slice and call
        // `to_owned` here, but we don't know `T` is `Clone`.
        let len = usize::deserialize(buf)?;
        let mut vec = Vec::new();
        for _ in 0..len {
            vec.push(F::deserialize(buf)?);
        }
        Ok(vec)
    }
}

impl Serialize for str {
    type Form = Str;
    fn serialize<S: MarkedWrite>(&self, stream: &mut S) -> io::Result<usize> {
        self.as_bytes().serialize(stream)
    }
}

impl<'b> DeserializeAs<'b, &'b str> for Str {
    fn deserialize(buf: &mut &'b [u8]) -> Result<&'b str, DeserializeError> {
        let bytes = <Seq<u8>>::deserialize(buf)?;
        std::str::from_utf8(bytes).map_err(|_| DeserializeError::BadUTF8)
    }
}

/// Borrow a `&[T]` slice from `buf`, respecting `T`'s alignment requirements.
///
/// Skip bytes from the front of `buf` until it is aligned as required to hold a
/// `T` value. Return a slice of `len` values of type `T`, and advance `buf`
/// to the next byte.
///
/// Return an `DeserializeError` if `buf` is not large enough to hold the aligned
/// slice.
pub fn borrow_aligned_slice<'b, T: raw::Simple>(
    buf: &mut &'b [u8],
    len: usize,
) -> Result<&'b [T], DeserializeError> {
    let size: usize = mem::size_of::<T>();
    let align: usize = mem::align_of::<T>();

    let align_skip = (0_usize.wrapping_sub(buf.as_ptr() as usize)) & (align - 1);
    let full_size = align_skip + size * len;
    if buf.len() < full_size {
        return Err(DeserializeError::UnexpectedEof);
    }

    // Safe because `T : raw::Simple`.
    let slice = unsafe { std::slice::from_raw_parts(buf[align_skip..].as_ptr() as *const T, len) };

    *buf = &buf[full_size..];
    Ok(slice)
}

#[derive(Debug)]
pub enum DeserializeError {
    UnexpectedEof,
    BadUTF8,
    Leb128ReadError(leb128::read::Error),
}

impl std::fmt::Display for DeserializeError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DeserializeError::UnexpectedEof => {
                fmt.write_str("serialized OpenGL method call argument data truncated")
            }
            DeserializeError::BadUTF8 => {
                fmt.write_str("serialized OpenGL method call argument data included bad UTF-8")
            }
            DeserializeError::Leb128ReadError(err) => err.fmt(fmt),
        }
    }
}

impl std::error::Error for DeserializeError {}

impl From<leb128::read::Error> for DeserializeError {
    fn from(err: leb128::read::Error) -> DeserializeError {
        DeserializeError::Leb128ReadError(err)
    }
}
