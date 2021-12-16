#![feature(global_asm)]
#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![feature(trait_alias)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(core_intrinsics)]
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
    use memory::mmu::MMU;

    if let Err(string) = memory::mmu::mmu().enable_mmu_and_caching() {
        panic!("MMU: {}", string);
    }

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

    kinfo!("MMU online. Special regions:");
    bsp::memory::mmu::virt_mem_layout().print_layout();

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

    let remapped_uart = unsafe { bsp::device_driver::PL011Uart::new(0x1FFF_1000) };
    writeln!(
        remapped_uart,
        "[     !!!    ] Writing through the remapped UART at 0x1FFF_1000"
    )
    .unwrap();

    kinfo!("Echoing input now");
    console().clear_rx().unwrap();

    loop {
        let c = console().read_char().unwrap();
        console().write_char(c).unwrap();
    }
}
