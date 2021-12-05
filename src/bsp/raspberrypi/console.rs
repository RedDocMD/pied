use spin::Mutex;

use crate::console;
use core::{
    fmt::{self, Write},
    ptr,
};

struct QEMUOutput {
    inner: Mutex<QEMUOutputInner>,
}

static QEMU_OUTPUT: QEMUOutput = QEMUOutput::new();

pub fn console() -> &'static impl console::Console {
    &QEMU_OUTPUT
}

impl QEMUOutput {
    const fn new() -> Self {
        Self {
            inner: Mutex::new(QEMUOutputInner { cnt: 0 }),
        }
    }
}

impl console::Write for QEMUOutput {
    fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result {
        self.inner.lock().write_fmt(args)
    }

    fn write_char(&self, c: char) -> fmt::Result {
        self.inner.lock().write_char(c);
        Ok(())
    }

    fn flush(&self) -> fmt::Result {
        Ok(())
    }
}

impl console::Read for QEMUOutput {
    fn read_char(&self) -> Result<char, fmt::Error> {
        unimplemented!("Cannot read from QEMU hack")
    }

    fn clear_rx(&self) -> fmt::Result {
        unimplemented!("Cannot clear rx of QEMU hack")
    }
}

impl console::Statistics for QEMUOutput {
    fn chars_written(&self) -> usize {
        self.inner.lock().cnt
    }
}

struct QEMUOutputInner {
    cnt: usize,
}

impl QEMUOutputInner {
    fn write_char(&mut self, ch: char) {
        unsafe { ptr::write_volatile(0x3F20_1000 as *mut u8, ch as u8) };
        self.cnt += 1;
    }
}

impl fmt::Write for QEMUOutputInner {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            if c == '\n' {
                self.write_char('\r');
            }
            self.write_char(c);
        }
        Ok(())
    }
}
