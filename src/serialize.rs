use crate::call;
use crate::raw;

/// A trait for types that can serialize GL method call streams.
pub trait Serializer {
    type Error: std::error::Error;

    /// Write the method call `call`.
    fn write_call(&mut self, call: &call::Call) -> Result<(), Self::Error>;

    /// Write the contents of the buffer `buf`.
    fn write_variable(&mut self, buf: &[u8]) -> Result<(), Self::Error>;

    /// Return an identifier for the next value written with `write_variable`.
    fn next_variable_id(&self) -> usize;

    /// Flush buffers, if any.
    fn flush(&mut self) -> Result<(), Self::Error>;
}

/// A type that can be serialized to a Serializer.
pub trait Serialize {
    /// Serialize a a single `Self` value.
    fn write<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error>;

    /// Serialize a `[Self]` slice. This can be overridden by implementations for types
    /// that can written as a single block.
    fn write_slice<S: Serializer>(this: &[Self], serializer: &mut S) -> Result<(), S::Error>
    where
        Self: Sized,
    {
        serializer.write_variable(raw::as_bytes(&this.len()))?;
        for elt in this {
            elt.write(serializer)?;
        }
        Ok(())
    }
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
                    serializer.write_variable(raw::as_bytes(&this.len()))?;
                    serializer.write_variable(raw::slice_as_bytes(&this))?;
                    Ok(())
                }
            }
        )*
    }
}

// These types are serialized in variable content as themselves.
simply_serialized_types!(bool, u8, u32, i32, f32, f64, usize);

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
