use std::io::{self, Read, Write};

pub fn putc(c: u8) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let buf = [c];
    stdout.write(&buf)?;
    stdout.flush()?;
    Ok(())
}

pub fn getc() -> anyhow::Result<u8> {
    let mut buf = [0_u8; 1];
    let mut stdin = io::stdin();
    let bytes_read = stdin.read(&mut buf)?;
    assert!(bytes_read == 1);
    Ok(buf[0])
}
