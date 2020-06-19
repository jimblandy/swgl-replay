//! Types representing the form in which values are serialized.

/// Placeholder for a `T` value stored in the variable-length section.
///
/// Use the `Slice` constructor in `T` instead of `[T]` or `Vec`. T should omit
/// `&` altogether. `Var<str>` is okay.
pub struct Var<T> {
    offset: usize,
    _referent: std::marker::PhantomData<*const T>
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

/// A run-length-encoded string of bytes.
///
/// An `Rle<T>` represents a stream of `T` values as alternating 'runs' and
/// 'literals':
/// - A 'run' is a count C followed by an
///   unaligned `T` value, and represents C repetitions of the `T` value.
/// - A 'literal' is a count C followed by `C` unaligned `T` values,
///   and represents the given sequence of `T` values.
///
/// All counts are stored as ULEB128 values.
///
/// An `Rle<T>` is either empty, or starts with a run. A run is always followed
/// by a literal or the end of the data. A literal is always followed by a run
/// or the end of the data.
pub struct Rle<T> {
    _referent: std::marker::PhantomData<*const T>
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
        write!(fmt, "Var at {}", self.offset)
    }
}
