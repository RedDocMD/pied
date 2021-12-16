use core::cell::UnsafeCell;

// BSP Memory Management.
//
// The physical memory layout.
//
// The Raspberry's firmware copies the kernel binary to 0x8_0000. The preceding region will be used
// as the boot core's stack.
//
// +---------------------------------------+
// |                                       | 0x0
// |                                       |                                ^
// | Boot-core Stack                       |                                | stack
// |                                       |                                | growth
// |                                       |                                | direction
// +---------------------------------------+
// |                                       | code_start @ 0x8_0000
// | .text                                 |
// | .rodata                               |
// | .got                                  |
// |                                       |
// +---------------------------------------+
// |                                       | code_end_exclusive
// | .data                                 |
// | .bss                                  |
// |                                       |
// +---------------------------------------+
// |                                       |
// |                                       |

pub mod mmu;

extern "Rust" {
    static __code_start: UnsafeCell<()>;
    static __code_end_exclusive: UnsafeCell<()>;
}

#[rustfmt::skip]
pub(super) mod map {
    pub const GPIO_OFFSET:                usize = 0x0020_0000;
    pub const UART_OFFSET:                usize = 0x0020_1000;
    pub const PM_RSTC_OFFSET:             usize = 0x0010_001c;
    pub const PM_RSTS_OFFSET:             usize = 0x0010_0020;
    pub const PM_WDOG_OFFSET:             usize = 0x0010_0024;

    // END_INCLUSIVE + 1 = 4GiB (although RPi3 has only 1GiB of RAM)
    pub const END_INCLUSIVE:              usize = 0xFFFF_FFFF;

    #[cfg(feature = "bsp_rpi3")]
    pub mod mmio {
        use super::*;

        pub const START:             usize = 0x3F00_0000;
        pub const GPIO_START:        usize = START + GPIO_OFFSET;
        pub const PL011_UART_START:  usize = START + UART_OFFSET;
        pub const PM_RSTC_START:     usize = START + PM_RSTC_OFFSET;
        pub const PM_RSTS_START:     usize = START + PM_RSTS_OFFSET;
        pub const PM_WDOG_START:     usize = START + PM_WDOG_OFFSET;
        // END_INCLUSIVE + 1 = 1GiB
        pub const END_INCLUSIVE:     usize = 0x4000_FFFF;
    }

    #[cfg(feature = "bsp_rpi4")]
    pub mod mmio {
        use super::*;

        pub const START:             usize = 0xFE00_0000;
        pub const GPIO_START:        usize = START + GPIO_OFFSET;
        pub const PL011_UART_START:  usize = START + UART_OFFSET;
        // END_INCLUSIVE + 1 = 4GiB - 8MiB
        pub const END_INCLUSIVE:     usize = 0xFF84_FFFF;
    }
}

#[inline(always)]
#[cfg(feature = "bsp_rpi3")]
pub fn board_pm_rstc() -> *const u32 {
    map::mmio::PM_RSTC_START as _
}

#[inline(always)]
#[cfg(feature = "bsp_rpi3")]
pub fn board_pm_rsts() -> *const u32 {
    map::mmio::PM_RSTS_START as _
}

#[inline(always)]
#[cfg(feature = "bsp_rpi3")]
pub fn board_pm_wdog() -> *const u32 {
    map::mmio::PM_WDOG_START as _
}

// Start page address of the code segment
#[inline(always)]
fn code_start() -> usize {
    unsafe { __code_start.get() as usize }
}

// Exclusive end page address of the code segment
#[inline(always)]
fn code_end_exclusive() -> usize {
    unsafe { __code_end_exclusive.get() as usize }
}
