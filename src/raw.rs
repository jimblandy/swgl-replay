//! Utilities for raw pointer and slice handling.

use gleam::gl::{GLvoid, GLsizeiptr};

/// Given a reference, return a byte slice of the value's representation.
pub fn as_bytes<T: Copy>(r: &T) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(r as *const T as *const u8, std::mem::size_of_val(r))
    }
}

pub unsafe fn slice_from_buf<T>(_lifetime: &T, data: *const GLvoid, size: GLsizeiptr) -> &[u8] {
    std::slice::from_raw_parts(data as *const u8, size as usize)
}

