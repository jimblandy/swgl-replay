use docopt::Docopt;
use serde::Deserialize;
use std::io;

use swgl_replay::{Call, FileRecording};
use gl_replay::replay;

const USAGE: &'static str = "
Dump swgl-replay command log.

Usage:
  dump-commands <dir>...
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
        let recording = match FileRecording::open(dir, swgl_replay::SWGR_MAGIC) {
            Err(err) => {
                eprintln!("{}: {}", err, dir);
                continue;
            }
            Ok(recording) => recording,
        };

        for (i, call) in recording.calls.iter().enumerate() {
            match call {
                Call::note(note) => {
                    let note: &str = replay::get_parameter(*note, &recording.variable);
                    println!("{:4} note: {:?}", i, note);
                }
                other => {
                    println!("{:4} {:?}", i, other);
                }
            }
        }
    }

    Ok(())
}
