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

pub fn console() -> &'static impl console::interface::Write {
    &QEMU_OUTPUT
}

impl QEMUOutput {
    const fn new() -> Self {
        Self {
            inner: Mutex::new(QEMUOutputInner {}),
        }
    }
}

impl console::interface::Write for QEMUOutput {
    fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result {
        self.inner.lock().write_fmt(args)
    }
}

struct QEMUOutputInner;

impl QEMUOutputInner {
    fn write_char(&mut self, ch: char) {
        unsafe { ptr::write_volatile(0x3F20_1000 as *mut u8, ch as u8) }
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
