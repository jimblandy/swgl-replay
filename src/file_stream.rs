//! A `var::CallStream` implementation that saves data to the filesystem.

use std::io::prelude::*;
use std::io::Write;
use std::path::Path;
use std::{fs, io, mem};

use crate::raw::{self, Simple};
use crate::var::{CallStream, MarkedWrite};

/// A `CallStream` implementation that writes the OpenGL calls to files on disk.
pub struct FileStream<Call> {
    calls: io::BufWriter<fs::File>,
    variable: io::BufWriter<fs::File>,
    bytes_written: usize,
    call_serial: usize,
    size_limit: usize,
    _phantom: std::marker::PhantomData<Call>
}

impl<Call: Simple> FileStream<Call> {
    pub fn create<P: AsRef<Path>>(dir: P, magic: u32) -> io::Result<FileStream<Call>> {
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
        calls.write_all(raw::as_bytes(&Header::for_call::<Call>(magic)))?;

        Ok(FileStream {
            calls,
            variable,
            bytes_written: 0,
            call_serial: 0,
            size_limit: 4 * 1024 * 1024 * 1024,
            _phantom: Default::default(),
        })
    }

    pub fn set_size_limit(&mut self, limit: usize) {
        self.size_limit = limit;
    }
}

impl<Call> io::Write for FileStream<Call> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written = self.variable.write(buf)?;
        self.bytes_written += written;

        // This lets us go over by one `write`, but the intent of the size limit
        // is just to avoid accidentally owning your own machine, so this will
        // hopefully be good enough.
        if self.bytes_written > self.size_limit {
            panic!("gl-replay: file stream size limit reached");
        }

        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.calls.flush()?;
        self.variable.flush()?;
        Ok(())
    }
}

impl<Call> MarkedWrite for FileStream<Call> {
    fn mark(&self) -> usize {
        self.bytes_written
    }
}

impl<Stored, Passed> CallStream<Passed> for FileStream<Stored>
    where Stored: Simple,
          Passed: Into<Stored>
{
    fn write_call(&mut self, call: Passed) -> io::Result<usize> {
        let call = call.into();
        let n = self.call_serial;
        self.calls.write_all(raw::as_bytes(&call))?;
        self.call_serial += 1;
        Ok(n)
    }

    fn call_serial(&self) -> usize {
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
    unsafe {
        raw::try_extend_vec_uninit(&mut vec, len, |elt_slice| -> io::Result<()> {
            // Make sure that this vector's buffer is properly aligned. All vector
            // buffers should be...
            if bytes != 0 {
                assert!(elt_slice.as_ptr() as *const () as usize & alignment - 1 == 0);
            }

            let byte_slice = raw::slice_as_bytes_mut(elt_slice);
            // unstable: file.initializer().initialize(byte_slice);
            file.read_exact(byte_slice)
        })?;
    }

    Ok(vec)
}

impl<Call: Simple> FileRecording<Call> {
    pub fn open<P: AsRef<Path>>(dir: P, magic: u32) -> io::Result<FileRecording<Call>> {
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
        header.check::<Call>(magic)?;

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
    // Using a `u32` here ensures we get different magic numbers on big-endian
    // and little-endian machines.
    magic: u32,
    size_of_usize: u8,
    size_of_call: u8,
    max_alignment: u8,
    padding: u8,
}

unsafe impl Simple for Header { }

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
    fn for_call<Call: Simple>(magic: u32) -> Header {
        // The header had better not cause the content to be misaligned.
        assert!(mem::size_of::<Header>() % max_alignment::<Call>() == 0);

        // The properties we want to stick into the header had better actually
        // fit in a single byte.
        assert!(mem::size_of::<Call>() <= 255);

        Header {
            magic,
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

    fn check<Call: Simple>(&self, magic: u32) -> io::Result<()> {
        let mut expected = Header::for_call::<Call>(magic);
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
