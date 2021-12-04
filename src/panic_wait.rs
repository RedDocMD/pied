use crate::{cpu, kprintln};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(args) = info.message() {
        kprintln!("\nKernel panic: {}", args);
    } else {
        kprintln!("\nKernel panic!");
    }
    cpu::wait_forever()
}
