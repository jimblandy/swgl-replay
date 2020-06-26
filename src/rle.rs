//! Run-length encoding for pixel data.
//!
//! This module provides functions for reading and writing slices of simple
//! values, and using run-length encoding to compress repetitive stretches of
//! data. (If this module isn't significantly faster than full PNG compression,
//! then we should just use that by default, and drop this.)
//!
//! This module provides separate functions for byte slices and `u32` slices,
//! because run-length encoding can benefit greatly from even limited knowledge
//! of the structure of the data. For example, a stream of identical 32-bit
//! values, when viewed as bytes, looks like a repeating four-byte pattern. If
//! we compress that data as bytes, it may not have any runs at all, but if we
//! compress it as 32-bit values, it compresses perfectly. If we need to, it's
//! easy to generalize this to other pixel sizes.
//!
//! ## Format
//!
//! The slice's contents are written as alternating 'runs' and 'literals':
//!
//! - A 'run' is a count C followed by a `T` value, and represents C
//!   repetitions of the `T` value.
//!
//! - A 'literal' is a count C followed by that many `T` values, and
//!   represents the given sequence of `T` values.
//!
//! The overall stream is either empty, or starts with a run. A run is always
//! followed by a literal or the end of the data. A literal is always followed
//! by a run or the end of the data.
//!
//! Note that literal counts may be zero, if the encoding really just wants to
//! switch from one run to another run.
//!
//! The exact format of the count depends on the data being written. Byte slices
//! use the LEB128 format for the counts. `u32` slices store the count as a
//! `u32`, and keep the stream aligned on a four-byte boundary.

use crate::raw;
use crate::var::DeserializeError;
use std::{io, mem};

/// Write a slice of bytes with run-length encoding.
pub fn write_u8<S>(stream: &mut S, data: &[u8]) -> Result<(), io::Error>
where
    S: io::Write,
{
    write_general(stream, data,
                  |stream, count| {
                      leb128::write::unsigned(stream, count as u64)
                  })
}

/// Write a slice of `u32` values with run-length encoding.
///
/// If the stream is aligned on a four-byte boundary, this writes aligned
/// values.
pub fn write_u32<S>(stream: &mut S, data: &[u32]) -> Result<(), io::Error>
where
    S: io::Write,
{
    write_general(stream, data,
                  |stream, count| {
                      let count = count as u32;
                      stream.write_all(raw::as_bytes(&count))
                  })
}

/// Write a generic slice of values with run-length encoding.
///
/// Write `data` to `stream`, representing contiguous runs of equal elements of
/// `T` as a repetition count followed by the repeated value.
///
/// All counts are written using `write_count`, so that determines their format.
/// This function inserts no padding before counts or `T` values, so
/// `write_count` can control both. (Of course, since `stream` is just an
/// `io::Write` implementor, the best this function can do is preserve
/// alignment, not establish it.)
pub fn write_general<T, S, W, R>(
    stream: &mut S,
    mut data: &[T],
    mut write_count: W,
) -> Result<(), io::Error>
where
    T: raw::Simple + PartialEq,
    S: io::Write,
    W: FnMut(&mut S, usize) -> Result<R, io::Error>,
{
    // If `data` is non-empty, start with a run.
    let mut lead = match data.split_first() {
        None => return Ok(()),
        Some((head, tail)) => {
            data = tail;
            *head
        }
    };
    let mut run_length = 1;

    loop {
        // invariant: `data` is the portion of the input immediately following
        // `run_length` consecutive copies of `lead`.

        // Extend the run as far as we can.
        let extension_length = data.iter().take_while(|&&v| v == lead).count();

        write_count(stream, run_length + extension_length)?;
        stream.write_all(raw::as_bytes(&lead))?;
        data = &data[extension_length..];

        // Write a literal. Figuring out the optimal place to end a literal and
        // switch to a run is not straightforward. Don't bother trying to be
        // optimal; just require at least four repetitions to switch to a run.
        let literal_tail = match data.split_first() {
            None => return Ok(()),
            Some((head, tail)) => {
                lead = *head;
                tail
            }
        };
        run_length = 1;

        let mut literal_length = 1;
        for elt in literal_tail {
            literal_length += 1;
            if *elt == lead {
                run_length += 1;
                if run_length >= 4 {
                    break;
                }
            } else {
                lead = *elt;
                run_length = 1;
            }
        }

        // If we didn't find a long enough run, this literal goes to the end.
        if run_length < 4 {
            assert_eq!(literal_length, data.len());
            write_count(stream, literal_length)?;
            stream.write_all(raw::slice_as_bytes(data))?;
            return Ok(());
        }

        // Write out this literal, and begin the next run.
        literal_length -= run_length;
        write_count(stream, literal_length)?;
        stream.write_all(raw::slice_as_bytes(&data[..literal_length]))?;
        data = &data[literal_length + run_length..];
    }
}

#[test]
fn test_write_u8() {
    fn check(data: &[u8], rle: &[u8]) {
        let mut buf = vec![];
        assert!(write_u8(&mut buf, &data).is_ok());
        assert_eq!(buf, rle);
    }

    check(&[], &[]);
    check(&[1], &[1, 1]);
    check(&[1, 1], &[2, 1]);
    check(&[1, 1, 1, 2, 2, 2, 2], &[3, 1, 0, 4, 2]);
    check(&[1, 2, 3, 4, 5, 6], &[1, 1, 5, 2, 3, 4, 5, 6]);
    check(&[1, 2, 3, 3, 3], &[1, 1, 4, 2, 3, 3, 3]);
    check(&[1, 2, 3, 3, 3, 3], &[1, 1, 1, 2, 4, 3]);
    check(&[1, 2, 3, 3, 3, 3, 3], &[1, 1, 1, 2, 5, 3]);

    check(&[1, 2, 3, 3, 3, 3, 3, 4, 5], &[1, 1, 1, 2, 5, 3, 2, 4, 5]);
    check(
        &[1, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5],
        &[1, 1, 1, 2, 4, 3, 0, 4, 4, 0, 4, 5],
    );
}

#[test]
fn test_write_u32() {
    fn bytes_to_u32(bytes: &[u8]) -> &[u32] {
        if bytes.is_empty() {
            return &[];
        }
        assert!(bytes.len() & (std::mem::size_of::<u32>() - 1) == 0);
        assert!(bytes.as_ptr() as usize & (std::mem::align_of::<u32>() - 1) == 0);
        unsafe {
            std::slice::from_raw_parts(bytes.as_ptr() as *const u32,
                                       bytes.len() / 4)
        }
    }

    fn check(data: &[u32], rle: &[u32]) {
        let mut buf = vec![];
        assert!(write_u32(&mut buf, &data).is_ok());
        dbg!(&buf);
        assert_eq!(bytes_to_u32(&buf), rle);
    }

    check(&[], &[]);
    check(&[90], &[1, 90]);
    check(&[1, 1], &[2, 1]);
    check(&[1, 1, 1, 2, 2, 2, 2], &[3, 1, 0, 4, 2]);
    check(&[1, 2, 3, 4, 5, 6], &[1, 1, 5, 2, 3, 4, 5, 6]);
    check(&[1, 2, 3, 3, 3], &[1, 1, 4, 2, 3, 3, 3]);
    check(&[1, 2, 3, 3, 3, 3], &[1, 1, 1, 2, 4, 3]);
    check(&[1, 2, 3, 3, 3, 3, 3], &[1, 1, 1, 2, 5, 3]);

    check(&[1, 2, 3, 3, 3, 3, 3, 4, 5], &[1, 1, 1, 2, 5, 3, 2, 4, 5]);
    check(
        &[1, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5],
        &[1, 1, 1, 2, 4, 3, 0, 4, 4, 0, 4, 5],
    );
}

/// A type which can store expanded run-length-encoded data.
pub trait RleSink<T: Copy> {
    type Error: From<DeserializeError>;
    fn write_run(&mut self, value: T, count: usize) -> Result<(), Self::Error>;
    fn write_literal(&mut self, values: &[T]) -> Result<(), Self::Error>;
}

impl RleSink<u8> for Vec<u8> {
    type Error = DeserializeError;
    fn write_run(&mut self, value: u8, count: usize) -> Result<(), Self::Error> {
        unsafe {
            raw::extend_vec_uninit(self, count, |slice| {
                std::ptr::write_bytes(slice.as_mut_ptr(), value, slice.len());
            });
        }
        Ok(())
    }
    fn write_literal(&mut self, values: &[u8]) -> Result<(), Self::Error> {
        self.extend_from_slice(values);
        Ok(())
    }
}

impl RleSink<u32> for Vec<u8> {
    type Error = DeserializeError;
    fn write_run(&mut self, value: u32, count: usize) -> Result<(), Self::Error> {
        unsafe {
            raw::extend_vec_uninit(self, count * mem::size_of::<u32>(), move |slice| {
                let slice = std::slice::from_raw_parts_mut(
                    slice.as_mut_ptr() as *mut u32,
                    count
                );
                slice.iter_mut().for_each(|elt| *elt = value);
            });
        }
        Ok(())
    }
    fn write_literal(&mut self, values: &[u32]) -> Result<(), Self::Error> {
        let count = values.len();
        unsafe {
            raw::extend_vec_uninit(self, count * mem::size_of::<u32>(), move |slice| {
                let slice = std::slice::from_raw_parts_mut(
                    slice.as_mut_ptr() as *mut u32,
                    count
                );
                slice.copy_from_slice(values);
            });
        }
        Ok(())
    }
}

/// Read run-length encoded `u8` values from `buf`, returning a `Vec<u8>`.
pub fn read_u8(buf: &mut &[u8]) -> Result<Vec<u8>, DeserializeError> {
    let mut expanded = Vec::new();
    read_general(buf, &mut expanded, |buf| {
        Ok(leb128::read::unsigned(buf)? as usize)
    })?;
    Ok(expanded)
}

/// Read run-length encoded `u32` values from `buf`, returning a `Vec<u8>`.
pub fn read_u32(buf: &mut &[u32]) -> Result<Vec<u8>, DeserializeError> {
    let mut expanded = Vec::new();
    read_general(buf, &mut expanded, |buf| {
        match buf.split_first() {
            Some((head, tail)) => {
                *buf = tail;
                Ok(*head as usize)
            }
            None => {
                Err(DeserializeError::UnexpectedEof)
            }
        }
    })?;
    Ok(expanded)
}

/// Read run-length encoded data from `buf`, and write it to `sink`.
///
/// Use `read_count` to parse read counts from `buf`.
pub fn read_general<T, R, S>(buf: &mut &[T], sink: &mut S, mut read_count: R) -> Result<(), S::Error>
where
    T: raw::Simple + PartialEq,
    R: FnMut(&mut &[T]) -> Result<usize, DeserializeError>,
    S: RleSink<T>
{
    loop {
        if buf.is_empty() {
            break;
        }

        // Expand a run.
        let count = read_count(buf)?;
        let value = match buf.split_first() {
            Some((head, tail)) => {
                *buf = tail;
                *head
            }
            None => {
                return Err(S::Error::from(DeserializeError::UnexpectedEof));
            }
        };
        sink.write_run(value, count)?;

        if buf.is_empty() {
            break;
        }

        // Expand a literal.
        let count = read_count(buf)?;
        let slice = match buf.get(..count) {
            Some(slice) => slice,
            None => {
                return Err(S::Error::from(DeserializeError::UnexpectedEof));
            }
        };
        sink.write_literal(slice)?;
        *buf = &buf[count..];
    }

    Ok(())
}

#[test]
fn test_read_u8() {
    fn check(mut rle: &[u8], expected: &[u8]) {
        let result = read_u8(&mut rle);

        assert_eq!(result.unwrap(), expected);
    }

    check(&[], &[]);
    check(&[1, 1], &[1]);
    check(&[2, 1], &[1, 1]);
    check(&[3, 1, 0, 4, 2], &[1, 1, 1, 2, 2, 2, 2]);
    check(&[1, 1, 5, 2, 3, 4, 5, 6], &[1, 2, 3, 4, 5, 6]);
    check(&[1, 1, 4, 2, 3, 3, 3], &[1, 2, 3, 3, 3]);
    check(&[1, 1, 1, 2, 4, 3], &[1, 2, 3, 3, 3, 3]);
    check(&[1, 1, 1, 2, 5, 3], &[1, 2, 3, 3, 3, 3, 3]);

    check(&[1, 1, 1, 2, 5, 3, 2, 4, 5], &[1, 2, 3, 3, 3, 3, 3, 4, 5]);
    check(
        &[1, 1, 1, 2, 4, 3, 0, 4, 4, 0, 4, 5],
        &[1, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5],
    );
}
