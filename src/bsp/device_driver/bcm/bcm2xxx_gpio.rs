use spin::Mutex;
use tock_registers::{
    interfaces::{ReadWriteable, Writeable},
    registers::ReadWrite,
};

use crate::bsp::device_driver::common::MMIODerefWrapper;

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
}
