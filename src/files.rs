//! A `Serializer` that saves method calls in files.

use std::{fs, io};
use std::io::Write;
use std::path::Path;

use super::Serializer;
use crate::{call, raw};

/// A stream of Gl method calls being written to files.
pub struct Files {
    calls: io::BufWriter<fs::File>,
    //large: io::BufWriter<fs::File>,
}

impl Files {
    pub fn create<P: AsRef<Path>>(dir: P) -> io::Result<Files> {
        let dir = dir.as_ref();

        match fs::create_dir(dir) {
            Err(e) if e.kind() != io::ErrorKind::AlreadyExists => {
                return Err(e);
            }
            _ => ()

        }

        let mut calls = io::BufWriter::new(fs::File::create(dir.join("calls"))?);
        assert!(std::mem::size_of::<call::Call>() <= 255);
        calls.write_all(&[b'G', b'L', b'R', b'R',
                          std::mem::size_of::<call::Call>() as u8,
                          0, 0, 0])?;

        Ok(Files {
            calls,
            //large: io::BufWriter::new(fs::File::create(dir.join("large"))?),
        })
    }
}

impl Serializer for Files {
    type Error = io::Error;

    fn write_call(&mut self, call: &call::Call) -> Result<(), Self::Error> {
        self.calls.write_all(raw::as_bytes(call))?;
        Ok(())
    }

    fn write_buffer(&mut self, buf: &[u8], ident: usize) -> Result<(), Self::Error> {
        self.calls.write_all(raw::as_bytes(&ident))?;
        self.calls.write_all(buf)?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.calls.flush()?;
        Ok(())
    }
}
