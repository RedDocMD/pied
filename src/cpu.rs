#[cfg(target_arch = "aarch64")]
#[path = "_arch/aarch64/cpu/cpu.rs"]
mod arch_cpu;

mod boot;

#[cfg(feature = "bsp_rpi3")]
pub use arch_cpu::spin_for_cycles;

pub use arch_cpu::{nop, wait_forever};
