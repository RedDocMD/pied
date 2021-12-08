use std::io::{self, Read, Write};

use crate::error::SoapboxResult;

pub fn putc(c: u8) -> SoapboxResult<()> {
    let mut stdout = io::stdout();
    let buf = [c];
    stdout.write(&buf)?;
    stdout.flush()?;
    Ok(())
}

pub fn getc() -> SoapboxResult<u8> {
    let mut buf = [0_u8; 1];
    let mut stdin = io::stdin();
    let bytes_read = stdin.read(&mut buf)?;
    assert!(bytes_read == 1);
    Ok(buf[0])
}
