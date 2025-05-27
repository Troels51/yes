use std::io::{self, Write};

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


fn main() -> io::Result<()>{
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    loop {
        handle.write_all(&YES_ARRAY)?;
    }
}
