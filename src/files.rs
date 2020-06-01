//! A `Serializer` that saves method calls in files.

use gleam::gl::GLfloat;
use std::io::prelude::*;
use std::io::Write;
use std::path::Path;
use std::{fs, io, mem};

use super::Serializer;
use crate::call::Call;
use crate::raw;

/// A stream of Gl method calls being written to files.
pub struct Files {
    calls: io::BufWriter<fs::File>,
    variable: io::BufWriter<fs::File>,
    bytes_written: usize,
}

// A type whose alignment is as strict as we need. Add more types to
// this as needed.
#[allow(dead_code)]
union Alignment {
    call: Call,
    gl_float: GLfloat,
}

impl Files {
    pub fn create<P: AsRef<Path>>(dir: P) -> io::Result<Files> {
        let dir = dir.as_ref();

        match fs::create_dir(dir) {
            Err(e) if e.kind() != io::ErrorKind::AlreadyExists => {
                return Err(e);
            }
            _ => (),
        }

        let mut calls = io::BufWriter::new(fs::File::create(dir.join("calls"))?);
        let variable = io::BufWriter::new(fs::File::create(dir.join("variable"))?);

        // Write a header to the file.
        assert!(mem::size_of::<Call>() <= 255);
        calls.write_all(&[
            b'G',
            b'L',
            b'R',
            b'R',
            mem::size_of::<usize>() as u8,
            mem::size_of::<Call>() as u8,
            mem::align_of::<Alignment>() as u8,
            0,
        ])?;

        Ok(Files {
            calls,
            variable,
            bytes_written: 0,
        })
    }
}

impl Serializer for Files {
    type Error = io::Error;

    fn write_call(&mut self, call: &Call) -> Result<(), Self::Error> {
        self.calls.write_all(raw::as_bytes(call))?;
        Ok(())
    }

    fn write_variable(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.variable.write_all(buf)?;
        self.bytes_written += buf.len();
        Ok(())
    }

    /// Return an identifier for the next value written with `write_variable`.
    fn variable_size(&self) -> usize {
        self.bytes_written
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.calls.flush()?;
        self.variable.flush()?;
        Ok(())
    }
}

/// A recording of GL calls, created with `Recorder`.
pub struct Recording {
    pub(crate) calls: Vec<Call>,
    pub(crate) variable: Vec<u8>,
}

/// Read the remaining contents of `file` directly into memory as a `Vec<T>`.
/// Assume that `skipped` bytes have already been read.
fn read_vector<T: Copy + 'static>(
    mut file: fs::File,
    skipped: usize,
    file_name: &str,
    type_name: &str)
 -> io::Result<Vec<T>> {
    // Make sure the remaining data has the size of a whole number of `T` values.
    let size = file.metadata()?.len() as usize - skipped;
    if size % mem::size_of::<T>() != 0 {
        return Err(io::Error::new(io::ErrorKind::Other,
                                  format!("gl-record {} file size is not an even number of {} structures",
                                          file_name, type_name)));
    }

    // Allocate a vector of the appropriate capacity. We just checked that this
    // division will have no remainder.
    let len = size / mem::size_of::<T>();
    let mut vec = Vec::with_capacity(len);

    // Make sure that this vector's buffer is properly aligned. All vector
    // buffers should be...
    if size != 0 {
        assert!(vec.as_ptr() as *const () as usize & mem::align_of::<Alignment>() - 1 == 0);
    }

    // Get a slice referring to the portion of vector's unused capacity that
    // we'll populate.
    let elt_slice = unsafe {
        std::slice::from_raw_parts_mut(vec.as_mut_ptr(), len)
    };
    // But as bytes, as read_exact expects.
    let byte_slice = raw::slice_as_bytes_mut(elt_slice);

    file.read_exact(byte_slice)?;

    // Set the vector's length to enclose the part we've now initialized.
    unsafe {
        vec.set_len(len);
    }

    Ok(vec)
}


impl Recording {
    pub fn open<P: AsRef<Path>>(dir: P) -> io::Result<Recording> {
        let dir = dir.as_ref();
        let mut calls_file = fs::File::open(dir.join("calls"))?;
        let variable_file = fs::File::open(dir.join("variable"))?;

        let mut header = [0_u8; 8];
        calls_file.read_exact(&mut header)?;
        Recording::check_header(&header)?;

        Ok(Recording {
            calls: read_vector(calls_file, mem::size_of_val(&header), "calls", "Call")?,
            variable: read_vector(variable_file, 0, "variable", "byte")?,
        })
    }

    pub fn check_header(header: &[u8; 8]) -> io::Result<()> {
        fn make_error(grouse: &str) -> io::Result<()> {
            Err(io::Error::new(io::ErrorKind::Other, grouse))
        }

        if &header[0..4] != b"GLRR" {
            return make_error("gl-replay header: bad magic number");
        }
        if header[4] as usize != mem::size_of::<usize>() {
            return make_error("gl-replay header: size of `usize` doesn't match");
        }
        if header[5] as usize != mem::size_of::<Call>() {
            return make_error("gl-replay header: size of `Call` doesn't match");
        }
        if header[6] as usize != mem::align_of::<f64>() {
            return make_error("gl-replay header: alignment of `f64` doesn't match");
        }

        Ok(())
    }
}
