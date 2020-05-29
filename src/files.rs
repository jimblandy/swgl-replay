//! A `Serializer` that saves method calls in files.

use std::io::Write;
use std::path::Path;
use std::{fs, io, mem};

use super::Serializer;
use crate::{call, raw};

/// A stream of Gl method calls being written to files.
pub struct Files {
    calls: io::BufWriter<fs::File>,
    variable: io::BufWriter<fs::File>,
    next_variable_offset: usize,
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
        assert!(mem::size_of::<call::Call>() <= 255);
        calls.write_all(&[
            b'G',
            b'L',
            b'R',
            b'R',
            mem::size_of::<usize>() as u8,
            mem::size_of::<call::Call>() as u8,
            0,
            0,
        ])?;

        Ok(Files {
            calls,
            variable,
            next_variable_offset: 0,
        })
    }
}

impl Serializer for Files {
    type Error = io::Error;

    fn write_call(&mut self, call: &call::Call) -> Result<(), Self::Error> {
        self.calls.write_all(raw::as_bytes(call))?;
        Ok(())
    }

    fn write_variable(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.variable.write_all(buf)?;
        self.next_variable_offset += buf.len();
        Ok(())
    }

    /// Return an identifier for the next value written with `write_variable`.
    fn next_variable_id(&self) -> usize {
        self.next_variable_offset
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.calls.flush()?;
        self.variable.flush()?;
        Ok(())
    }
}
