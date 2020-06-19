#![no_main]
use libfuzzer_sys::fuzz_target;

use gl_replay::var::DeserializeAs;
use gl_replay::form::Rle;

fuzz_target!(|data: &[u8]| {
    let mut buf = vec![];
    gl_replay::var::write_rle(data, &mut buf).expect("encoding failed");
    let mut var = &buf[..];
    match <Rle<u8>>::deserialize(&mut var) {
        Ok(vec) => assert_eq!(vec, data),
        Err(e) => panic!("Error: {}", e),
    }
});
