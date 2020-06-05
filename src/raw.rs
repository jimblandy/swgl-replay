//! Utilities for raw pointer and slice handling.

/// A marker trait for types that can be serialized by simply writing out their bytes.
pub unsafe trait Simple: Copy { }

/// Given a reference, return a byte slice of the value's representation.
pub fn as_bytes<T: Simple>(r: &T) -> &[u8] {
    slice_as_bytes(std::slice::from_ref(r))
}

/// Given a slice, return a byte slice of its contents.
pub fn slice_as_bytes<T: Simple>(r: &[T]) -> &[u8] {
    // Safe because `T::Simple`.
    unsafe { std::slice::from_raw_parts(r.as_ptr() as *const u8, std::mem::size_of_val(r)) }
}

/// Given a mutable slice, return a mutable byte slice of its contents.
///
/// Safety: the caller must ensure that the bit pattern written to the slice is
/// valid as a `[T]` slice. `Simple` doesn't imply that every bit pattern is a
/// valid `T`, so this function must be unsafe.
pub unsafe fn slice_as_bytes_mut<T: Simple>(r: &mut [T]) -> &mut [u8] {
    std::slice::from_raw_parts_mut(r.as_mut_ptr() as *mut u8, std::mem::size_of_val(r))
}

macro_rules! implement_simple {
    ( $( $type:ty ),* ) => {
        $(
            unsafe impl Simple for $type { }
        )*
    }
}

implement_simple!(u8, u16, u32, u64, u128, usize,
                  i8, i16, i32, i64, i128, isize,
                  f32, f64,
                  char, bool);
