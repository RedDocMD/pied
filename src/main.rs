#![feature(global_asm)]
#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![feature(trait_alias)]
#![feature(const_fn_fn_ptr_basics)]
#![no_main]
#![no_std]

mod bsp;
mod console;
mod cpu;
mod driver;
mod panic_wait;
mod print;

#[macro_use]
extern crate tock_registers;

unsafe fn kernel_init() -> ! {
    kprintln!("[0] Hello from Rust!");

    panic!("Stopping here.")
}
