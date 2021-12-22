use cortex_a::registers::*;
use tock_registers::interfaces::{Readable, Writeable};

trait DaifField {
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register>;
}

struct Debug;
struct SError;
struct IRQ;
struct FIQ;

impl DaifField for Debug {
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
        DAIF::D
    }
}

impl DaifField for SError {
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
        DAIF::A
    }
}

impl DaifField for IRQ {
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
        DAIF::I
    }
}

impl DaifField for FIQ {
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
        DAIF::F
    }
}

fn is_masked<T: DaifField>() -> bool {
    DAIF.is_set(T::daif_field())
}

/// Print the AArch64 exceptions status.
#[rustfmt::skip]
pub fn print_state() {
    use crate::kinfo;

    let to_mask_str = |x| -> _ {
        if x { "Masked" } else { "Unmasked" }
    };

    kinfo!("      Debug:  {}", to_mask_str(is_masked::<Debug>()));
    kinfo!("      SError: {}", to_mask_str(is_masked::<SError>()));
    kinfo!("      IRQ:    {}", to_mask_str(is_masked::<IRQ>()));
    kinfo!("      FIQ:    {}", to_mask_str(is_masked::<FIQ>()));
}

// Returns whether IRQs are masked on the executing core
pub fn is_local_irq_masked() -> bool {
    !is_masked::<IRQ>()
}

mod daif_bits {
    pub const IRQ: u8 = 0b0010;
}

// Unmask IRQs on the executing core.
// It is not required to put a synchronization barrier after this.
#[inline(always)]
pub unsafe fn local_irq_unmask() {
    #[rustfmt::skip]
    asm!(
        "msr DAIFClr, {arg}",
        arg = const daif_bits::IRQ,
        options(nomem, nostack, preserves_flags)
    )
}

// Mask IRQs on executing core
#[inline(always)]
pub unsafe fn local_irq_mask() {
    #[rustfmt::skip]
    asm!(
        "msr DAIFSet, {arg}",
        arg = const daif_bits::IRQ,
        options(nomem, nostack, preserves_flags)
    )
}

// Mask IRQs on the executing core and return the previously saved interrupt mask bits (DAIF)
#[inline(always)]
pub unsafe fn local_irq_mask_save() -> u64 {
    let saved = DAIF.get();
    local_irq_mask();
    saved
}

// Restore the interrupt mask bits (DAIF) using the calee's argument
// Precondition: Inputs must be sane
#[inline(always)]
pub unsafe fn local_irq_restore(saved: u64) {
    DAIF.set(saved);
}
