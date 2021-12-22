#[cfg(target_arch = "aarch64")]
#[path = "../_arch/aarch64/exception/asynchronous.rs"]
mod arch_asynchronous;

use core::{
    fmt::{self, Display, Formatter},
    marker::PhantomData,
};

pub use arch_asynchronous::print_state;

#[derive(Clone, Copy)]
pub struct IRQDescriptor {
    pub name: &'static str,
    pub handler: &'static (dyn interface::IRQHandler + Sync),
}

// IRQContext token.
//
// An instance of this type indicates that the local core is currently executing in IRQ
// context, aka executing an interrupt vector or subcalls of it.
//
// Concept and implementation derived from the `CriticalSection` introduced in
// <https://github.com/rust-embedded/bare-metal>
#[derive(Clone, Copy)]
pub struct IRQContext<'ctxt> {
    _0: PhantomData<&'ctxt ()>,
}

mod interface {
    pub trait IRQHandler {
        fn handle(&self) -> Result<(), &'static str>;
    }

    pub trait IRQManager {
        type IRQNumberType;

        fn register_handler(
            &self,
            irq_number: Self::IRQNumberType,
            descriptor: super::IRQDescriptor,
        ) -> Result<(), &'static str>;

        fn enable(&self, irq_number: Self::IRQNumberType);

        // Handle pending interrupts.
        //
        // This function is called directly from the CPU's IRQ exception vector. On AArch64,
        // this means that the respective CPU core has disabled exception handling.
        // This function can therefore not be preempted and runs start to finish.
        //
        // Takes an IRQContext token to ensure it can only be called from IRQ context.
        #[allow(clippy::trivially_copy_pass_by_ref)]
        fn handle_pending_irqs<'irq_context>(
            &'irq_context self,
            ic: &super::IRQContext<'irq_context>,
        );

        fn print_handlers(&self);
    }
}

pub use interface::*;

use self::arch_asynchronous::{local_irq_mask_save, local_irq_restore};

#[derive(Clone, Copy)]
pub struct IRQNumber<const MAX_INCLUSIVE: usize>(usize);

impl<'ctxt> IRQContext<'ctxt> {
    // Creates an IRQContext token.
    //
    // # Safety
    //
    // - This must only be called when the current core is in an interrupt context and will not
    //   live beyond the end of it. That is, creation is allowed in interrupt vector functions. For
    //   example, in the ARMv8-A case, in `extern "C" fn current_elx_irq()`.
    // - Note that the lifetime `'irq_context` of the returned instance is unconstrained. User code
    //   must not be able to influence the lifetime picked for this type, since that might cause it
    //   to be inferred to `'static`.
    #[inline(always)]
    pub unsafe fn new() -> Self {
        IRQContext { _0: PhantomData }
    }
}

impl<const MAX_INCLUSIVE: usize> IRQNumber<{ MAX_INCLUSIVE }> {
    pub const fn new(number: usize) -> Self {
        assert!(number <= MAX_INCLUSIVE);

        Self { 0: number }
    }

    pub const fn get(self) -> usize {
        self.0
    }
}

impl<const MAX_INCLUSIVE: usize> Display for IRQNumber<{ MAX_INCLUSIVE }> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Executes the provided closure while IRQs are masked on the executing core
// - Safety:
//   Restores the IRQ state after executing the closure
#[inline(always)]
pub fn exec_with_irq_masked<T>(f: impl FnOnce() -> T) -> T {
    let ret: T;

    unsafe {
        let saved = local_irq_mask_save();
        ret = f();
        local_irq_restore(saved);
    }

    ret
}
