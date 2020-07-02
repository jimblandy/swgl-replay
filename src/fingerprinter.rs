use gl_replay::CallStream;
use super::FileStream;
use crate::call::Call;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Write an entry to the recording fingerprinting the state of the recordee.
/// This is used for debugging record/replay divergence.
pub fn fingerprinter(swgl: &swgl::Context, stream: &mut FileStream) {
    stream.write_call(Call::fingerprint(fingerprint(swgl)))
        .expect("error writing fingerprint to swgl recording");
}

pub fn fingerprint(swgl: &swgl::Context) -> u64 {
    let tex_buffers = swgl.get_all_texture_buffers();
    let mut hasher = DefaultHasher::new();
    tex_buffers.hash(&mut hasher);
    hasher.finish()
}
