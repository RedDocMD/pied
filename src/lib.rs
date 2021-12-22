#![feature(global_asm)]
#![feature(asm)]
#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![feature(trait_alias)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(core_intrinsics)]
#![feature(stmt_expr_attributes)]
#![no_std]
// Testing
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(crate::test_runner)]

pub mod bsp;
pub mod console;
pub mod cpu;
pub mod driver;
pub mod exception;
pub mod memory;
pub mod panic_wait;
pub mod print;
pub mod time;

#[macro_use]
extern crate tock_registers;

#[cfg(not(test))]
extern "Rust" {
    fn kernel_init() -> !;
}

#[cfg(test)]
#[no_mangle]
unsafe fn kernel_init() -> ! {
    exception::handling_init();

    panic!("Testing not supported yet!");
}
