//! Types representing the form in which values are serialized.

/// Placeholder for a `T` value stored in the variable-length section.
///
/// Use the `Slice` constructor in `T` instead of `[T]` or `Vec`. T should omit
/// `&` altogether. `Var<str>` is okay.
pub struct Var<T> {
    offset: usize,
    _referent: std::marker::PhantomData<*const T>
}

// We can't derive `Copy`, `Clone`, or `Debug`, because of the `PhantomData`:
// https://github.com/rust-lang/rust/issues/26925
impl<T> Clone for Var<T> {
    fn clone(&self) -> Var<T> {
        Var {
            offset: self.offset,
            _referent: Default::default(),
        }
    }
}

impl<T> Copy for Var<T> { }

impl<T> std::fmt::Debug for Var<T> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("Var")
            .field("offset", &self.offset)
            .finish()
    }
}

impl<T> Var<T> {
    pub fn new(offset: usize) -> Var<T> {
        Var {
            offset,
            _referent: Default::default()
        }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}

/// A sequence of values: slices, vectors, strings.
///
/// A `Seq` is serialized as a `usize` length, followed by the serialized form
/// of that many values.
#[derive(Clone, Copy, Debug)]
pub struct Seq<T> {
    _referent: std::marker::PhantomData<*const T>
}

/// A UTF-8 string.
///
/// A `Str` is serialized as a `usize` length, followed by the UTF-8 form of the string.
pub struct Str;
