use std::io::{self, Write};

fn main() -> io::Result<()>{
    const BUFFER_SIZE: usize =  4*1024;
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let buffer = b"y\n".repeat(BUFFER_SIZE/2);
    loop {
        handle.write_all(buffer.as_slice())?;
    }
}