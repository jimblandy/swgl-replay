use docopt::Docopt;
use serde::Deserialize;
use swgl::Context;
use swgl_replay::{FileRecording, ReplayState};

use std::io;

static USAGE: &str = "
Replay swgl-replay command log.

Usage:
  swgl-replay <dir>
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_dir: String,
}

fn main() -> io::Result<()> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let recording = FileRecording::open(args.arg_dir, swgl_replay::SWGR_MAGIC)?;

    let swgl = Context::create();
    swgl.make_current();

    ReplayState::from_swgl(swgl).replay(&recording.calls, &recording.variable);
    Ok(())
}
