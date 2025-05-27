use std::io::{self, Write};

fn main() -> io::Result<()>{
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let buffer = b"y\n".repeat(1024/2);
    loop {
        handle.write_all(buffer.as_slice())?;
    }
    Ok(())
}
