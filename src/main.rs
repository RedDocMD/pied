#![feature(format_args_nl)]
#![no_main]
#![no_std]

use libkernel::*;

#[no_mangle]
unsafe fn kernel_init() -> ! {
    use driver::DriverManager;
    use memory::mmu::MMU;

    exception::handling_init();

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

    // Cause an exception by accessing a virtual address for which no translation was set up. This
    // code accesses the address 8 GiB, which is outside the mapped address space.
    //
    // For demo purposes, the exception handler will catch the faulting 8 GiB address and allow
    // execution to continue.
    kinfo!("");
    kinfo!("Trying to read from address 8 GiB...");
    let mut big_addr: u64 = 8 * 1024 * 1024 * 1024;
    unsafe { core::ptr::read_volatile(big_addr as *mut u64) };

    kinfo!("************************************************");
    kinfo!("Whoa! We recovered from a synchronous exception!");
    kinfo!("************************************************");
    kinfo!("");
    kinfo!("Let's try again");

    // Now use address 9 GiB. The exception handler won't forgive us this time.
    kinfo!("Trying to read from address 9 GiB...");
    big_addr = 9 * 1024 * 1024 * 1024;
    unsafe { core::ptr::read_volatile(big_addr as *mut u64) };

    kinfo!("Echoing input now");
    console().clear_rx().unwrap();

    loop {
        let c = console().read_char().unwrap();
        console().write_char(c).unwrap();
    }
}
