use crate::{bsp, console};
use core::fmt;

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use console::Write;

    bsp::console::console().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ($crate::print::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kprintln {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::print::_print(format_args_nl!($($arg)*));
    })
}
