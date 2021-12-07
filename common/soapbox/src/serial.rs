use std::{
    io::{Read, Write},
    time::Duration,
};

use serialport::TTYPort;

struct SerialConsole {
    tty: TTYPort,
}

const BAUD_RATE: u32 = 921_600;

impl SerialConsole {
    pub fn new(dev_name: &str) -> anyhow::Result<Self> {
        use serialport::*;

        let port_builder = serialport::new(dev_name, BAUD_RATE)
            .data_bits(DataBits::Eight)
            .flow_control(FlowControl::None)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .timeout(Duration::from_secs(100));
        let tty = TTYPort::open(&port_builder)?;
        Ok(Self { tty })
    }

    pub fn putc(&mut self, b: u8) -> anyhow::Result<()> {
        let buf = [b];
        self.tty.write(&buf)?;
        self.tty.flush()?;
        Ok(())
    }

    pub fn getc(&mut self) -> anyhow::Result<u8> {
        let mut buf = [0_u8; 1];
        let bytes_read = self.tty.read(&mut buf)?;
        assert!(bytes_read == 1);
        Ok(buf[0])
    }
}
