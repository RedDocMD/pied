pub mod interface {
    use core::fmt;

    pub trait Write {
        fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;
    }
}
