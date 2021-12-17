use std::{
    io::{self, Read, Write},
    path::Path,
    time::Duration,
};

use serialport::SerialPort;

use crate::error::SoapboxResult;

pub struct Serial {
    tty: Box<dyn SerialPort>,
}

const BAUD_RATE: u32 = 921_600;

impl Serial {
    pub fn new(dev_name: &str) -> SoapboxResult<Self> {
        use serialport::*;

        let port_builder = serialport::new(dev_name, BAUD_RATE)
            .data_bits(DataBits::Eight)
            .flow_control(FlowControl::None)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .timeout(Duration::from_secs(0));
        let tty = port_builder.open()?;
        Ok(Self { tty })
    }

    pub fn putc(&mut self, b: u8) -> SoapboxResult<()> {
        let buf = [b];
        self.write(&buf)
    }

    pub fn getc(&mut self) -> SoapboxResult<u8> {
        let mut buf = [0_u8; 1];
        let bytes_read = self.read(&mut buf)?;
        assert!(bytes_read == 1);
        Ok(buf[0])
    }

    pub fn read_partial(&mut self, buf: &mut [u8]) -> SoapboxResult<usize> {
        self.tty.set_timeout(Duration::from_secs(0))?;
        let size = match self.tty.read(buf) {
            Ok(len) => len,
            Err(err) => {
                let io_error: serialport::Error = err.into();
                if let serialport::ErrorKind::Io(kind) = io_error.kind {
                    if kind == io::ErrorKind::TimedOut {
                        return Ok(0);
                    }
                }
                return Err(io_error)?;
            }
        };
        Ok(size)
    }

    pub fn read(&mut self, buf: &mut [u8]) -> SoapboxResult<usize> {
        self.tty.set_timeout(Duration::from_secs(1000_000))?;
        Ok(self.tty.read(buf)?)
    }

    pub fn write(&mut self, buf: &[u8]) -> SoapboxResult<()> {
        self.tty.write_all(buf)?;
        Ok(self.tty.flush()?)
    }
}

pub fn is_serial_connected<P: AsRef<Path>>(tty_name: P) -> bool {
    tty_name.as_ref().exists()
}
