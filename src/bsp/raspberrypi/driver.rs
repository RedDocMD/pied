use crate::driver::{self, DeviceDriver};

struct BSPDriverManager {
    device_drivers: [&'static (dyn DeviceDriver + Sync); 2],
}

static BSP_DRIVER_MANAGER: BSPDriverManager = BSPDriverManager {
    device_drivers: [&super::GPIO, &super::PL011_UART],
};

pub fn driver_manager() -> &'static impl driver::DriverManager {
    &BSP_DRIVER_MANAGER
}

impl driver::DriverManager for BSPDriverManager {
    fn all_device_drivers(&self) -> &[&'static (dyn DeviceDriver + Sync)] {
        &self.device_drivers[..]
    }

    fn post_device_driver_init(&self) {
        super::GPIO.map_pl011_uart();
    }
}
