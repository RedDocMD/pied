use cortex_a::{asm, registers::*};
use tock_registers::interfaces::Writeable;

global_asm!(include_str!("boot.s"));

// # Safety
//
// - The `bss` section is not initialized yet. The code must not use or reference it in any way.
// - The HW state of EL1 must be prepared in a sound way
#[inline(always)]
unsafe fn prepare_el2_to_el1_transition(phys_boot_core_stack_end_exclusive_addr: u64) {
    // Enable timer and counter registers for EL1
    CNTHCTL_EL2.write(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);

    // No offset for reading the counters
    CNTVOFF_EL2.set(0);

    // Set EL1 execution state to AArch64
    HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);

    // Set up the simulated exception return.

    // First, fake a saved prorgram status where all interrupts were masked and SP_EL1 was used as
    // a stack pointer.
    SPSR_EL2.write(
        SPSR_EL2::D::Masked
            + SPSR_EL2::A::Masked
            + SPSR_EL2::I::Masked
            + SPSR_EL2::F::Masked
            + SPSR_EL2::M::EL1h,
    );

    // Second, let the link register point to kernel_init()
    ELR_EL2.set(crate::kernel_init as *const () as u64);

    // Finally, set up SP_EL1 (stack pointer), which will be used by EL1 once we "return" to it.
    // Since there are no plans ot ever return to EL2, just re-use the same stack.
    SP_EL1.set(phys_boot_core_stack_end_exclusive_addr);
}

// # Safety
//
// - Exception return from EL2 must must continue execution in EL1 with `kernel_init()`.
#[no_mangle]
pub unsafe extern "C" fn _start_rust(phys_boot_core_stack_end_exclusive_addr: u64) -> ! {
    prepare_el2_to_el1_transition(phys_boot_core_stack_end_exclusive_addr);

    // Use `eret` to "return" to EL1. This results in execution of kernel_init() in EL1.
    asm::eret()
}
