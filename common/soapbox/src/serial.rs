use std::{
    io::{Read, Write},
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
            .timeout(Duration::from_secs(5));
        let tty = port_builder.open()?;
        Ok(Self { tty })
    }

    pub fn putc(&mut self, b: u8) -> SoapboxResult<()> {
        let buf = [b];
        self.tty.write(&buf)?;
        self.tty.flush()?;
        Ok(())
    }

    pub fn getc(&mut self) -> SoapboxResult<u8> {
        let mut buf = [0_u8; 1];
        let bytes_read = self.tty.read(&mut buf)?;
        assert!(bytes_read == 1);
        Ok(buf[0])
    }

    pub fn read(&mut self, buf: &mut [u8]) -> SoapboxResult<usize> {
        let size = self.tty.read(buf)?;
        Ok(size)
    }

    pub fn write(&mut self, buf: &[u8]) -> SoapboxResult<()> {
        self.tty.write_all(buf)?;
        Ok(())
    }
}

pub fn is_serial_connected<P: AsRef<Path>>(tty_name: P) -> bool {
    tty_name.as_ref().exists()
}
