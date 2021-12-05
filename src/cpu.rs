#[cfg(target_arch = "aarch64")]
#[path = "_arch/aarch64/cpu/cpu.rs"]
mod arch_cpu;

mod boot;

pub use arch_cpu::spin_for_cycles;
pub use arch_cpu::wait_forever;
