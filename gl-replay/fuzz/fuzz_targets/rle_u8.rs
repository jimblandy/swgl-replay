#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut buf = vec![];
    gl_replay::rle::write_u8(&mut buf, data).expect("encoding failed");
    let mut var = &buf[..];
    match gl_replay::rle::read_u8(&mut var) {
        Ok(vec) => assert_eq!(vec, data),
        Err(e) => panic!("Error: {}", e),
    }
});
