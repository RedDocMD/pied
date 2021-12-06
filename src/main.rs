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
    use driver::DriverManager;

    for i in bsp::driver::driver_manager().all_device_drivers().iter() {
        if let Err(x) = i.init() {
            panic!("Error loading driver: {}: {}", i.compatible(), x);
        }
    }
    bsp::driver::driver_manager().post_device_driver_init();
    // kprintln! is usable from here on

    // Transition from unsafe to safe
    kernel_main()
}

fn kernel_main() -> ! {
    use bsp::console::console;
    use console::Console;
    use driver::DriverManager;

    kprintln!(
        "[0] {} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    kprintln!("[1] Booting on: {}", bsp::board_name());

    kprintln!("[2] Drivers loaded:");
    for (i, driver) in bsp::driver::driver_manager()
        .all_device_drivers()
        .iter()
        .enumerate()
    {
        kprintln!("      {}. {}", i + 1, driver.compatible());
    }

    kprintln!(
        "[3] Chars written: {}",
        bsp::console::console().chars_written()
    );
    kprintln!("[4] Echoing input now");

    // Discard any spurious received characters before going into echo mode.
    console().clear_rx().unwrap();
    loop {
        let c = bsp::console::console().read_char().unwrap();
        bsp::console::console().write_char(c).unwrap();
    }
}
