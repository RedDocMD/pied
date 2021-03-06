use spin::Mutex;
use tock_registers::{
    interfaces::{ReadWriteable, Writeable},
    registers::ReadWrite,
};

use crate::{bsp::device_driver::common::MMIODerefWrapper, driver};

register_bitfields![
    u32,
    GPFSEL1 [
        // Pin 15
        FSEL15 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100, // PL011 UART RX
        ],

        // Pin 14
        FSEL14 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100, // PL011 UART TX
        ],
    ],
    // GPIO Pull-up/down Register
        // BCM2837 only
    GPPUD [
        PUD OFFSET(0) NUMBITS(2) [
            Off = 0b00,
            PullDown = 0b01,
            PullUp = 0b10,
        ]
    ],

    // GPIO Pull-up/down Clock Register 0
    // BCM2837 Only
    GPPUDCLK0 [
        // GPIO 15
        PUDCLK15 OFFSET(15) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1,
        ],
        // GPIO 14
        PUDCLK14 OFFSET(14) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1,
        ]
    ],

    // GPOP Pull-up/down Register 0
    // BCM2711 Only
    GPIO_PUP_PDN_CNTRL_REG0 [
        // Pin 15
        GPIO_PUP_PDN_CNTRL15 OFFSET(30) NUMBITS(2) [
            NoResistor = 0b00,
            PullUp = 0b01,
        ],

        /// Pin 14
        GPIO_PUP_PDN_CNTRL14 OFFSET(28) NUMBITS(2) [
            NoResistor = 0b00,
            PullUp = 0b01
        ]
    ]
];

register_structs! {
    #[allow(non_snake_case)]
    RegisterBlock {
        (0x00 => _reserved1),
        (0x04 => GPFSEL1: ReadWrite<u32, GPFSEL1::Register>),
        (0x08 => _reserved2),
        (0x94 => GPPUD: ReadWrite<u32, GPPUD::Register>),
        (0x98 => GPPUDCLK0: ReadWrite<u32, GPPUDCLK0::Register>),
        (0x9C => _reserved3),
        (0xE4 => GPIO_PUP_PDN_CNTRL_REG0: ReadWrite<u32, GPIO_PUP_PDN_CNTRL_REG0::Register>),
        (0xE8 => @END),
    }
}

type Registers = MMIODerefWrapper<RegisterBlock>;

pub struct GPIOInner {
    registers: Registers,
}

pub use GPIOInner as PanicGPIO;

pub struct GPIO {
    inner: Mutex<GPIOInner>,
}

impl GPIOInner {
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            registers: Registers::new(mmio_start_addr),
        }
    }

    // Disable pull-up/down on pins 14 and 15.
    #[cfg(feature = "bsp_rpi3")]
    fn disable_pud_14_15_bcm2837(&mut self) {
        use crate::{time, time::TimeManager};
        use core::time::Duration;

        // The Linux 2837 GPIO driver waits for 1??s between the steps
        const DELAY: Duration = Duration::from_micros(1);

        self.registers.GPPUD.write(GPPUD::PUD::Off);
        time::time_manager().spin_for(DELAY);

        self.registers
            .GPPUDCLK0
            .write(GPPUDCLK0::PUDCLK15::AssertClock + GPPUDCLK0::PUDCLK14::AssertClock);
        time::time_manager().spin_for(DELAY);

        self.registers.GPPUD.write(GPPUD::PUD::Off);
        self.registers.GPPUDCLK0.set(0);
    }

    // Disable pull-up/down on pins 14 and 15.
    #[cfg(feature = "bsp_rpi4")]
    fn disable_pud_14_15_bcm2711(&mut self) {
        self.registers.GPIO_PUP_PDN_CNTRL_REG0.write(
            GPIO_PUP_PDN_CNTRL_REG0::GPIO_PUP_PDN_CNTRL15::PullUp
                + GPIO_PUP_PDN_CNTRL_REG0::GPIO_PUP_PDN_CNTRL14::PullUp,
        );
    }

    pub fn map_pl011_uart(&mut self) {
        // TX to pin 14
        // RX to pin 15
        self.registers
            .GPFSEL1
            .modify(GPFSEL1::FSEL15::AltFunc0 + GPFSEL1::FSEL14::AltFunc0);

        // Disable pull-up/down on pins 14 and 15.
        #[cfg(feature = "bsp_rpi3")]
        self.disable_pud_14_15_bcm2837();

        #[cfg(feature = "bsp_rpi4")]
        self.disable_pud_14_15_bcm2711();
    }
}

impl GPIO {
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            inner: Mutex::new(GPIOInner::new(mmio_start_addr)),
        }
    }

    pub fn map_pl011_uart(&self) {
        self.inner.lock().map_pl011_uart();
    }
}

impl driver::DeviceDriver for GPIO {
    fn compatible(&self) -> &'static str {
        "BCM GPIO"
    }
}
