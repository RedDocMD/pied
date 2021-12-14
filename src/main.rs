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
mod exception;
mod memory;
mod null_lock;
mod panic_wait;
mod print;
mod time;

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
    use core::time::Duration;
    use driver::DriverManager;
    use time::TimeManager;

    kinfo!(
        "{} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    kinfo!("Booting on: {}", bsp::board_name());

    let (_, privilege_level) = exception::current_privillege_level();
    kinfo!("Current privilege level: {}", privilege_level);

    kinfo!("Exception handling state:");
    exception::asynchronous::print_state();

    kinfo!(
        "Architectural timer resolution: {} ns",
        time::time_manager().resolution().as_nanos()
    );

    kinfo!("Drivers loaded:");
    for (i, driver) in bsp::driver::driver_manager()
        .all_device_drivers()
        .iter()
        .enumerate()
    {
        kinfo!("      {}. {}", i + 1, driver.compatible());
    }

    kinfo!("Timer test, spinning for 1 second");
    time::time_manager().spin_for(Duration::from_secs(1));

    kinfo!("Echoing input now");
    console().clear_rx().unwrap();

    loop {
        let c = console().read_char().unwrap();
        console().write_char(c).unwrap();
    }
}
