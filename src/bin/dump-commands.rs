use docopt::Docopt;
use serde::Deserialize;
use std::io;

use gl_replay::FileRecording;
use gl_replay::Call;

type Recording = FileRecording<Call>;

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
        let recording = match Recording::open(dir, gl_replay::GL_MAGIC) {
            Err(err) => {
                eprintln!("{}: {}", err, dir);
                continue;
            }
            Ok(recording) => recording,
        };

        for (i, call) in recording.calls.iter().enumerate() {
            println!("{:4} {:?}", i, call);
        }
    }

    Ok(())
}
