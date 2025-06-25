use std::io::{self, Write};

fn main() -> io::Result<()>{
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    loop {
        handle.write_all(b"y\n")?;
    }
}