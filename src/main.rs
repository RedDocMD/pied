#![feature(global_asm)]
#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![no_main]
#![no_std]

mod bsp;
mod console;
mod cpu;
mod panic_wait;
mod print;

unsafe fn kernel_init() -> ! {
    kprintln!("[0] Hello from Rust!");

    panic!("Stopping here.")
}
