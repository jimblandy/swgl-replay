use std::mem;

use crate::call;
use crate::raw;

/// A trait for types that can serialize GL method call streams.
pub trait Serializer {
    type Error: std::error::Error;

    /// Write the method call `call` to the `Call` stream.
    fn write_call(&mut self, call: &call::Call) -> Result<(), Self::Error>;

    /// Write the contents of the buffer `buf` to the variable-length data
    /// stream.
    fn write_variable(&mut self, buf: &[u8]) -> Result<(), Self::Error>;

    /// Write `slice` to the variable-length data stream, padding as needed to
    /// align it properly.
    fn write_aligned_slice<T: Copy + 'static>(&mut self, slice: &[T]) -> Result<(), Self::Error> {
        let pos = self.variable_size();
        let align: usize = mem::align_of::<T>();

        let align_skip = (0 - pos) & (align-1);
        if align_skip > 0 {
            static PADDING: [u8; 64] = [b'P'; 64];
            assert!(align_skip <= PADDING.len());
            self.write_variable(&PADDING[..align_skip])?;
        }

        self.write_variable(raw::slice_as_bytes(slice))
    }

    /// Return the number of bytes that have been written to the variable-length
    /// data stream so far.
    fn variable_size(&self) -> usize;

    /// Flush buffers, if any.
    fn flush(&mut self) -> Result<(), Self::Error>;
}

/// A type that can be serialized to a Serializer.
pub trait Serialize {
    /// Serialize a single `Self` value.
    fn write<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error>;

    /// Serialize a `[Self]` slice. This can be overridden by implementations for types
    /// that can written as a single block.
    fn write_slice<S: Serializer>(this: &[Self], serializer: &mut S) -> Result<(), S::Error>
    where
        Self: Sized,
    {
        serializer.write_aligned_slice(&[this.len()])?;
        for elt in this {
            elt.write(serializer)?;
        }
        Ok(())
    }
}

impl<T: Serialize + ?Sized> Serialize for &T {
    fn write<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error> {
        (*self).write(serializer)
    }
}

impl<T: Serialize> Serialize for [T] {
    fn write<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error> {
        <T as Serialize>::write_slice(self, serializer)
    }
}

impl Serialize for str {
    fn write<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error> {
        self.as_bytes().write(serializer)
    }
}

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
        std::str::from_utf8(bytes)
            .map_err(|_| DeserializeError::BadUTF8)
    }
}

/// Borrow a `&[T]` slice from `buf`, respecting `T`'s alignment requirements.
///
/// Skip bytes from the front of `buf` until it is aligned as required to hold a
/// `T` value. Return a slice of `count` values of type `T`, and advance `buf`
/// to the next byte.
///
/// Return an `DeserializeError` if `buf` is not large enough to hold the aligned
/// slice.
fn take_slice<'b, T: Copy + 'static>(buf: &mut &'b [u8], count: usize) -> Result<&'b [T], DeserializeError> {
    let size: usize = mem::size_of::<T>();
    let align: usize = mem::align_of::<T>();

    let align_skip = (align - buf.as_ptr() as usize) & !(align-1);
    let full_len = align_skip + size * count;
    if buf.len() < full_len {
        return Err(DeserializeError::UnexpectedEof);
    }

    let slice = unsafe {
        std::slice::from_raw_parts(buf[align_skip..].as_ptr() as *const T, count)
    };

    *buf = &buf[full_len..];
    Ok(slice)
}

macro_rules! simply_serialized_types {
    ( $( $type:ty ),* ) => {
        $(
            impl Serialize for $type {
                fn write<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error> {
                    serializer.write_variable(raw::as_bytes(self))
                }

                /// For these types, we can write out the whole block at once.
                fn write_slice<S: Serializer>(this: &[Self], serializer: &mut S) -> Result<(), S::Error> {
                    serializer.write_aligned_slice(&[this.len()])?;
                    serializer.write_aligned_slice(this)?;
                    Ok(())
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

#[derive(Debug, Clone)]
pub enum DeserializeError {
    UnexpectedEof,
    BadUTF8,
}

impl std::fmt::Display for DeserializeError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(match self {
            DeserializeError::UnexpectedEof =>
                "serialized OpenGL method call argument data truncated",
            DeserializeError::BadUTF8 =>
                "serialized OpenGL method call argument data included bad UTF-8",
        })
    }
}

