
#![no_main]
use std::io::{self, Write};
use std::fs::File;
use std::os::unix::io::FromRawFd;


const N: usize = 1024;
const YES: [u8; 2] = *b"y\n";
const fn create_yes_array() -> [u8; N] {
    let mut res = [0; N];
    let mut i = 0;
    while i < res.len() {
        res[i] = YES[0];
        res[i + 1] = YES[1];
        i += 2;
    }
    res
}
const YES_ARRAY: [u8; N] = create_yes_array();

fn stdout() -> File {
    unsafe { File::from_raw_fd(1) }
}

#[unsafe(no_mangle)]
pub fn main(_argc: i32, _argv: *const *const u8) {
    let mut stdout = stdout();
    loop {
        let _ = stdout.write(&YES_ARRAY);
    }
}
