//! Utilities for raw pointer and slice handling.

/// A marker trait for types that can be treated as blocks of bytes.
///
/// When `Self` implements `Simple`, that means:
///
/// - It can be serialied in one process and deserialized in a new process to an
///   equivalent value simply by copying the bytes.
///
/// - Its lifetime is `'static`.
///
/// This is stricter than `Copy + 'static`: for example, `&'static str` meets
/// those bounds and yet is not `Simple`, because an address is only meaningful
/// in the address space in which it's created.
pub unsafe trait Simple: Copy {}

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

/// Extend a vector by initializing a slice of its unused capacity.
///
/// Reserve space in `vec` for `additional` more elements, and apply
/// `initializer` to the slice of uninitialized capacity of that length. If
/// `initializer` succeeds, set `vec`'s length to include the now-initialized
/// elements. Return whatever `initializer` returns.
///
/// Safety: the caller must initialize every element of the slice to a valid `T`.
pub unsafe fn try_extend_vec_uninit<T, F, U, V>(
    vec: &mut Vec<T>,
    additional: usize,
    initializer: F,
) -> Result<U, V>
where
    T: Simple,
    F: FnOnce(&mut [T]) -> Result<U, V>,
{
    vec.reserve(additional);
    let free = vec.as_mut_ptr().offset(vec.len() as isize);
    let slice = std::slice::from_raw_parts_mut(free, additional);
    let value = initializer(slice)?;
    vec.set_len(vec.len() + additional);
    Ok(value)
}

/// Extend a vector by initializing a slice of its unused capacity.
///
/// Reserve space in `vec` for `additional` more elements, and apply
/// `initializer` to the slice of uninitialized capacity of that length. Set
/// `vec`'s length to include the now-initialized elements.
///
/// Safety: the caller must initialize every element of the slice to a valid `T`.
pub unsafe fn extend_vec_uninit<T, F>(vec: &mut Vec<T>, additional: usize, initializer: F)
where
    T: Simple,
    F: FnOnce(&mut [T]),
{
    try_extend_vec_uninit(vec, additional, |slice| -> Result<(), ()> {
        initializer(slice);
        Ok(())
    })
    .unwrap();
}

macro_rules! implement_simple {
    ( $( $type:ty ),* ) => {
        $(
            unsafe impl Simple for $type { }
        )*
    }
}

implement_simple!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64, char, bool
);
