//! Serializing parameters to `Gl` calls.

use crate::form::{Var, Seq, Str};
use crate::var::{Serialize, MarkedWrite};

use std::io;

/// A `Gl` method argument type.
///
/// There are two ways we can record the value of a `Gl` method argument:
///
/// - An argument type like `bool` or `f32` we can include directly in the `Call`.
///
/// - An argument type like `&[u8]` and `&str` we must serialize out into the
///   variable-length data section, and save its offset in a `Var` that we let
///   represent the value in the `Call`.
///
/// The argument type's `Parameter` implementation determines which strategy we
/// use.
pub trait Parameter {
    type Form;

    /// If `&self` is the actual value of the parameter passed to the `Gl`
    /// method, return the value that should represent it in the `Call`,
    /// serializing any side data to `stream`.
    fn to_call<S: MarkedWrite>(&self, stream: &mut S) -> io::Result<Self::Form>;
}

/// `Simple` types, in the `var` module's sense, are included directly in the
/// `Call`, and don't need to be written to the variable-length stream.
macro_rules! direct_parameters {
    ( $( $type:ty ),*) => {
        $(
            impl Parameter for $type {
                type Form = $type;
                fn to_call<S: MarkedWrite>(&self, _stream: &mut S) -> io::Result<Self> {
                    Ok(*self)
                }
            }
        )*
    }
}

direct_parameters!(u8, u16, u32, u64, u128, usize,
                   i8, i16, i32, i64, i128, isize,
                   f32, f64,
                   char, bool);

impl<T: Serialize> Parameter for [T] {
    type Form = Var<Seq<T::Form>>;

    fn to_call<S: MarkedWrite>(&self, stream: &mut S) -> io::Result<Self::Form> {
        Ok(Var::new(self.serialize(stream)?))
    }
}

impl<T: Serialize> Parameter for Vec<T> {
    type Form = Var<Seq<T::Form>>;

    fn to_call<S: MarkedWrite>(&self, stream: &mut S) -> io::Result<Self::Form> {
        Ok(Var::new(self.serialize(stream)?))
    }
}

impl Parameter for str {
    type Form = Var<Str>;

    fn to_call<S: MarkedWrite>(&self, stream: &mut S) -> io::Result<Self::Form> {
        Ok(Var::new(self.serialize(stream)?))
    }
}

/// A parameter of type `&T` is passed just as a parameter of type `T`.
impl<T: Parameter + ?Sized> Parameter for &T {
    type Form = T::Form;

    fn to_call<S: MarkedWrite>(&self, stream: &mut S) -> io::Result<Self::Form> {
        (**self).to_call(stream)
    }
}

/// A parameter of type `&mut T` is passed just as a parameter of type `T`.
/// Although, these are usually out-parameters, so we should record their values
/// *after* the call, not before.
impl<T: Parameter + ?Sized> Parameter for &mut T {
    type Form = T::Form;

    fn to_call<S: MarkedWrite>(&self, stream: &mut S) -> io::Result<Self::Form> {
        (**self).to_call(stream)
    }
}

/// We pass `Option<T>` as `None` if it is `None`, or `Some(f)` if it is `Some(v)`,
/// where we would pass `v` as `f`.
impl<T: Parameter> Parameter for Option<T> {
    type Form = Option<T::Form>;

    fn to_call<S: MarkedWrite>(&self, stream: &mut S) -> io::Result<Self::Form> {
        self.as_ref()
            .map(|param| param.to_call(stream))
            .transpose() // from `Option<Result>` to `Result<Option>`
    }
}
