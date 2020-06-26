#![no_main]
use libfuzzer_sys::fuzz_target;

fn bytes_to_u32(bytes: &[u8]) -> &[u32] {
    if bytes.is_empty() {
        return &[];
    }
    assert!(bytes.len() & (std::mem::size_of::<u32>() - 1) == 0);
    assert!(bytes.as_ptr() as usize & (std::mem::align_of::<u32>() - 1) == 0);
    unsafe {
        std::slice::from_raw_parts(bytes.as_ptr() as *const u32,
                                   bytes.len() / 4)
    }
}

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len() & !3];
    let data_u32 = bytes_to_u32(data);

    let mut buf = vec![];
    gl_replay::rle::write_u32(&mut buf, data_u32).expect("encoding failed");

    let mut var = bytes_to_u32(&buf);
    match gl_replay::rle::read_u32(&mut var) {
        Ok(vec) => assert_eq!(vec, data),
        Err(e) => panic!("Error: {}", e),
    }
});
