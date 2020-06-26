use docopt::Docopt;
use gl_replay::pixels::Pixels;
use gl_replay::replay::Parameter;
use gl_replay::Call as GlCall;
use serde::Deserialize;
use swgl_replay::Call as SwglCall;
use swgl_replay::FileRecording;

use std::io;

static USAGE: &str = "
Extract images from swgl-replay command log.

Scan the log of SWGL and OpenGL calls for uses of the `read_pixels_into_buffer`
method. For each call, write an image named `read_pixels_into_buffer-N.png`,
where N is the method call's serial number in the log.

Usage:
  dump-images <dir>
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

    let mut count = 0;
    for (i, call) in recording.calls.iter().enumerate() {
        match *call {
            SwglCall::gl(GlCall::read_pixels_into_buffer { x: _, y: _, pixels }) => {
                let pixels = Pixels::from_call(pixels, &recording.variable);
                let filename = format!("read_pixels_into_buffer-{}.png", i);
                pixels.write_image(&filename);
                count += 1;
            }
            _ => (),
        }
    }
    println!("wrote {} read_pixels_into_buffer images", count);

    Ok(())
}
