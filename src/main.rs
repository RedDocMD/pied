#![feature(global_asm)]
#![feature(asm)]
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
mod null_lock;
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

const MINILOAD_LOGO: &str = r#"
  __  __ _       _ _                     _ 
 |  \/  (_)     (_) |                   | |
 | \  / |_ _ __  _| |     ___   __ _  __| |
 | |\/| | | '_ \| | |    / _ \ / _` |/ _` |
 | |  | | | | | | | |___| (_) | (_| | (_| |
 |_|  |_|_|_| |_|_|______\___/ \__,_|\__,_|

"#;

fn kernel_main() -> ! {
    use bsp::console::console;
    use console::Console;

    kprintln!("{}", MINILOAD_LOGO);
    kprintln!("{:^37}", bsp::board_name());
    kprintln!("");
    kprintln!("[ML] Requesting binary");
    console().flush().unwrap();

    // Discard any spurious received characters before starting with the loader protocol
    console().clear_rx().unwrap();

    // Notify Minipush to send the binary
    for _ in 0..3 {
        console().write_char(3 as char).unwrap();
    }

    // Read the binary's size
    let mut buf = [0_u8; 4];
    for i in 0..buf.len() {
        buf[i] = console().read_char().unwrap() as u8;
    }
    // Assume we have a little-endian development system
    let size = u32::from_le_bytes(buf);

    // Trust it's not too big
    console().write_char('O').unwrap();
    console().write_char('K').unwrap();

    let kernel_addr: *mut u8 = bsp::memory::board_default_load_addr() as *mut u8;
    unsafe {
        // Read the kernel byte by byte
        for i in 0..size {
            let byte = console().read_char().unwrap() as u8;
            core::ptr::write_volatile(kernel_addr.offset(i as isize), byte);
        }
    }

    kprintln!("[ML] Loaded! Executing the payload now\n");
    console().flush().unwrap();

    // Create the function pointer like a C programmer
    let kernel: fn() -> ! = unsafe { core::mem::transmute(kernel_addr) };

    // Jump to loaded kernel
    kernel()
}
