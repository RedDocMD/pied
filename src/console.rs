mod interface {
    use core::fmt;

    pub trait Write {
        fn write_char(&self, c: char) -> fmt::Result;

        fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;

        fn flush(&self) -> fmt::Result;
    }

    pub trait Read {
        fn read_char(&self) -> Result<char, fmt::Error> {
            Ok(' ')
        }

        fn clear_rx(&self) -> fmt::Result;
    }

    pub trait Statistics {
        fn chars_written(&self) -> usize {
            0
        }

        fn chars_read(&self) -> usize {
            0
        }
    }

    pub trait Console = Write + Read + Statistics;
}

pub use interface::*;
