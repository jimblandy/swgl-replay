//! Utilities for raw pointer and slice handling.

/// Given a reference, return a byte slice of the value's representation.
pub fn as_bytes<T: Copy + 'static>(r: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts(r as *const T as *const u8, std::mem::size_of_val(r)) }
}

/// Given a slice, return a byte slice of its contents.
pub fn slice_as_bytes<T: Copy + 'static>(r: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(r.as_ptr() as *const u8, std::mem::size_of_val(r)) }
}

/// Given a mutable slice, return a mutable byte slice of its contents.
pub fn slice_as_bytes_mut<T: Copy + 'static>(r: &mut [T]) -> &mut [u8] {
    unsafe { std::slice::from_raw_parts_mut(r.as_mut_ptr() as *mut u8, std::mem::size_of_val(r)) }
}
