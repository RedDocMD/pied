mod interface {
    // For a particular driver
    pub trait DeviceDriver {
        // Return a compatibility string
        fn compatible(&self) -> &'static str;

        unsafe fn init(&self) -> Result<(), &'static str> {
            Ok(())
        }
    }

    // Device driver management
    pub trait DriverManager {
        // Return a slice of references to all `BSP`-instantiated drivers.
        // The order of devices is the order in which `DeviceDriver::init()` is called.
        fn all_device_drivers(&self) -> &[&'static (dyn DeviceDriver + Sync)];

        // Initialization code that runs after driver init.
        fn post_device_driver_init(&self);
    }
}

pub use interface::*;
