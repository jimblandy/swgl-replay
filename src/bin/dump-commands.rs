use docopt::Docopt;
use serde::Deserialize;
use std::io::prelude::*;
use std::{fs, io, mem, path};

use gl_replay::{Call, Recording};

const USAGE: &'static str = "
Dump gl-replay command log.

Usage:
  gl-replay <dir>...
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_dir: Vec<String>,
}

fn main() -> io::Result<()> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    for dir in &args.arg_dir {
        let mut file = io::BufReader::new(fs::File::open(path::Path::new(dir).join("calls"))?);

        let mut header = [0_u8; 8];
        file.read_exact(&mut header)?;
        if let Err(err) = Recording::check_header(&header) {
            eprintln!("{}: {}", err, dir);
            continue;
        }

        union CallBuffer {
            bytes: [u8; mem::size_of::<Call>()],
            call: Call,
        };

        let mut buf = CallBuffer {
            bytes: [0; mem::size_of::<Call>()],
        };
        for i in 0.. {
            match file.read_exact(unsafe { &mut buf.bytes }) {
                Err(e) => {
                    if e.kind() == io::ErrorKind::UnexpectedEof {
                        break;
                    }
                    return Err(e);
                }
                Ok(()) => (),
            };

            let call = unsafe { buf.call };
            println!("{:4} {:?}", i, call);
        }
    }

    Ok(())
}
