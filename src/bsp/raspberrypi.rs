pub mod console;
pub mod cpu;
pub mod driver;
pub mod memory;
pub mod power;

use super::device_driver;

static GPIO: device_driver::GPIO =
    unsafe { device_driver::GPIO::new(memory::map::mmio::GPIO_START) };

static PL011_UART: device_driver::PL011Uart =
    unsafe { device_driver::PL011Uart::new(memory::map::mmio::PL011_UART_START) };

pub fn board_name() -> &'static str {
    #[cfg(feature = "bsp_rpi3")]
    {
        "Raspberry Pi 3"
    }

    #[cfg(feature = "bsp_rpi4")]
    {
        "Raspberry Pi 4"
    }
}
