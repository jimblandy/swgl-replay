//! A `var::CallStream` implementation that saves data to the filesystem.

use std::io::prelude::*;
use std::io::Write;
use std::path::Path;
use std::{fs, io, mem};

use crate::raw::{self, Simple};
use crate::var::{CallStream, Stream};

/// A `CallStream` that writes the OpenGL calls to files on disk.
pub struct FileStream<Call> {
    calls: io::BufWriter<fs::File>,
    variable: io::BufWriter<fs::File>,
    bytes_written: usize,
    call_serial: usize,
    _phantom: std::marker::PhantomData<Call>
}

impl<Call: Simple> FileStream<Call> {
    pub fn create<P: AsRef<Path>>(dir: P) -> io::Result<FileStream<Call>> {
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
        calls.write_all(raw::as_bytes(&Header::for_call::<Call>()))?;

        Ok(FileStream {
            calls,
            variable,
            bytes_written: 0,
            call_serial: 0,
            _phantom: Default::default(),
        })
    }
}

impl<Call> Stream for FileStream<Call> {
    type Error = io::Error;

    fn write_unaligned(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let pos = self.bytes_written;
        self.variable.write_all(buf)?;
        self.bytes_written += buf.len();
        Ok(pos)
    }

    fn mark(&self) -> usize {
        self.bytes_written
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.calls.flush()?;
        self.variable.flush()?;
        Ok(())
    }
}

impl<Call: Simple> CallStream<Call> for FileStream<Call> {
    fn write_call(&mut self, call: &Call) -> Result<usize, Self::Error> {
        let n = self.call_serial;
        self.calls.write_all(raw::as_bytes(call))?;
        self.call_serial += 1;
        Ok(n)
    }

    fn serial(&self) -> usize {
        self.call_serial
    }
}

/// A recording of `Call` calls, created with `CallStream<Call>`.
pub struct FileRecording<Call> {
    pub calls: Vec<Call>,
    pub variable: Vec<u8>,
}

/// Read the remaining contents of `file` directly into memory as a `Vec<T>`.
/// Assume that `skipped` bytes have already been read.
fn read_vector<T: Simple>(
    mut file: fs::File,
    skipped: usize,
    alignment: usize,
    file_name: &str,
    type_name: &str,
) -> io::Result<Vec<T>> {
    // Make sure the remaining data has the size of a whole number of `T` values.
    let bytes = file.metadata()?.len() as usize - skipped;
    if bytes % mem::size_of::<T>() != 0 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "gl-record {} file size is not an even number of {} structures",
                file_name, type_name
            ),
        ));
    }

    // Allocate a vector of the appropriate capacity. We just checked that this
    // division will have no remainder.
    let len = bytes / mem::size_of::<T>();
    let mut vec = Vec::with_capacity(len);

    // Make sure that this vector's buffer is properly aligned. All vector
    // buffers should be...
    if bytes != 0 {
        assert!(vec.as_ptr() as *const () as usize & alignment - 1 == 0);
    }

    // Get a slice referring to the portion of vector's unused capacity that
    // we'll populate.
    let elt_slice = unsafe { std::slice::from_raw_parts_mut(vec.as_mut_ptr(), len) };
    // But as bytes, as read_exact expects.
    let byte_slice = unsafe { raw::slice_as_bytes_mut(elt_slice) };

    file.read_exact(byte_slice)?;

    // Set the vector's length to enclose the part we've now initialized.
    unsafe {
        vec.set_len(len);
    }

    Ok(vec)
}

impl<Call: Simple> FileRecording<Call> {
    pub fn open<P: AsRef<Path>>(dir: P) -> io::Result<FileRecording<Call>> {
        let dir = dir.as_ref();
        let mut calls_file = fs::File::open(dir.join("calls"))?;
        let variable_file = fs::File::open(dir.join("variable"))?;

        if calls_file.metadata()?.len() == 0 {
            return Err(io::Error::new(io::ErrorKind::Other,
                                      "gl-replay calls file is zero-length.\n\
                                       Are you recording to the same file you're trying to replay from?"))
        }

        let mut header = Header::zeros();
        calls_file.read_exact(unsafe {
            // This use of unsafe is totally bogus. Bad data in the file could
            // produce Calls with invalid discriminants, which is undefined
            // behavior.
            raw::slice_as_bytes_mut(std::slice::from_mut(&mut header))
        })?;
        header.check::<Call>()?;

        let alignment = max_alignment::<Call>();
        Ok(FileRecording {
            calls: read_vector(calls_file, mem::size_of_val(&header), alignment, "calls", "Call")?,
            variable: read_vector(variable_file, 0, alignment, "variable", "byte")?,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(packed)]
struct Header {
    magic: u32,
    size_of_usize: u8,
    size_of_call: u8,
    max_alignment: u8,
    padding: u8,
}

unsafe impl Simple for Header { }

pub const MAGIC: u32 = (((b'G' as u32) << 8 | (b'L' as u32)) << 8 | (b'R' as u32)) << 8 | (b'R' as u32);

fn max_alignment<Call: Copy>() -> usize {
    // A type whose alignment is as strict as we need. Add more types to
    // this as needed.
    #[allow(dead_code)]
    union Alignment<Call: Copy> {
        call: Call,
        gl_float: gleam::gl::GLfloat,
    }

    mem::align_of::<Alignment<Call>>()
}

impl Header {
    fn for_call<Call: Simple>() -> Header {
        assert_eq!(mem::size_of::<Header>(), 8);
        assert!(mem::size_of::<Call>() <= 255);
        Header {
            magic: MAGIC,
            size_of_usize: mem::size_of::<usize>() as u8,
            size_of_call: mem::size_of::<Call>() as u8,
            max_alignment: max_alignment::<Call>() as u8,
            padding: b'P',
        }
    }

    fn zeros() -> Header {
        Header {
            magic: 0,
            size_of_usize: 0,
            size_of_call: 0,
            max_alignment: 0,
            padding: 0,
        }
    }

    fn check<Call: Simple>(&self) -> io::Result<()> {
        let mut expected = Header::for_call::<Call>();
        expected.padding = self.padding;
        if expected != *self {
            let msg = format!("gl-replay header does not match:\n\
                               expected: {:?}\n\
                               actual:   {:?}\n",
                              expected, self);
            return Err(io::Error::new(io::ErrorKind::Other, msg));
        }
        Ok(())
    }
}
