use docopt::Docopt;
use serde::Deserialize;
use gl_replay::Call as GlCall;
use gl_replay::replay::Parameter;
use swgl_replay::FileRecording;
use swgl_replay::Call as SwglCall;

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
            SwglCall::gl(GlCall::read_pixels_into_buffer {
                x: _, y: _, width, height, format, pixel_type, dst_buffer
            }) => {
                let dst_buffer = <&[u8]>::from_call(dst_buffer, &recording.variable);
                let filename = format!("read_pixels_into_buffer-{}.png", i);
                gl_replay::write_image(&filename, &dst_buffer,
                                       width as usize, height as usize,
                                       format, pixel_type);
                count += 1;
            }
            _ => (),
        }
    }
    println!("wrote {} read_pixels_into_buffer images", count);

    Ok(())
}
