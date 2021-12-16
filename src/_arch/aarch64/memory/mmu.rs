use core::intrinsics::unlikely;

use cortex_a::{
    asm::barrier,
    registers::{ID_AA64MMFR0_EL1, MAIR_EL1, SCTLR_EL1, TCR_EL1, TTBR0_EL1},
};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use super::{translation_table::KernelTranslationTable, TranslationGranule};
use crate::{bsp, memory};

struct MemoryManagementUnit;

pub type GranuleSize512MiB = TranslationGranule<{ 512 * 1 << 20 }>;
pub type GranuleSize64KiB = TranslationGranule<{ 64 * 1 << 10 }>;

#[allow(dead_code)]
pub mod mair {
    pub const DEVICE: u64 = 0;
    pub const NORMAL: u64 = 1;
}

// The kernel translation tables
// Safety - supposed to land in ".bss". Therefore, ensure that all initial members are zeored out
static mut KERNEL_TABLES: KernelTranslationTable = KernelTranslationTable::new();

static MMU: MemoryManagementUnit = MemoryManagementUnit;

impl<const AS_SIZE: usize> memory::mmu::AddressSpace<AS_SIZE> {
    // Check the architectural restrictions
    pub const fn arch_address_space_size_sanity_check() {
        // Size must be at least one full 512 MiB table
        assert!((AS_SIZE % GranuleSize512MiB::SIZE) == 0);

        // Check for 48 bit virtual address size as maximum, which is supported by any ARMv8 version
        assert!(AS_SIZE <= (1 << 48));
    }
}

impl MemoryManagementUnit {
    // Setup function for the MAIR_EL1 register
    fn set_up_mair(&self) {
        // Define the memory types being mapped
        MAIR_EL1.write(
            // Attribute 1 - Cacheable normal DRAM
            MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc +
            MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc +
            // Atribute 0 - Device
            MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck,
        );
    }

    // Configure various settings of stage 1 of the EL1 translation regime
    fn configure_translation_control(&self) {
        let t0sz = (64 - bsp::memory::mmu::KernelAddrSpace::SHIFT) as u64;

        TCR_EL1.write(
            TCR_EL1::TBI0::Used
                + TCR_EL1::IPS::Bits_40
                + TCR_EL1::SH0::Inner
                + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
                + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
                + TCR_EL1::EPD0::EnableTTBR0Walks
                + TCR_EL1::A1::TTBR0
                + TCR_EL1::T0SZ.val(t0sz)
                + TCR_EL1::EPD0::DisableTTBR0Walks,
        );
    }
}

pub fn mmu() -> &'static impl memory::mmu::MMU {
    &MMU
}

use memory::mmu::MMUEnableError;

impl memory::mmu::MMU for MemoryManagementUnit {
    unsafe fn enable_mmu_and_caching(&self) -> Result<(), MMUEnableError> {
        if unlikely(self.is_enabled()) {
            return Err(MMUEnableError::AlreadyEnabled);
        }

        // Fail early if translation-granule is not supported
        if unlikely(!ID_AA64MMFR0_EL1.matches_all(ID_AA64MMFR0_EL1::TGran64::Supported)) {
            return Err(MMUEnableError::Other(
                "Translation granule not supported in HW",
            ));
        }

        // Prepare the memory attribute indirection register (MAIR)
        self.set_up_mair();

        // Populate translation tables
        KERNEL_TABLES
            .populate_tt_entries()
            .map_err(MMUEnableError::Other)?;

        // Set the "Translation Table Base Register"
        TTBR0_EL1.set_baddr(KERNEL_TABLES.phys_base_address());

        self.configure_translation_control();

        // Switch the MMU on
        // First, force all previous changes to be seen before the MMU is enabled
        barrier::isb(barrier::SY);

        // Enable the MMU and turn on data and instruction caching
        SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);

        // Force the MMU init to complete before next instruction
        barrier::isb(barrier::SY);

        Ok(())
    }

    #[inline(always)]
    fn is_enabled(&self) -> bool {
        SCTLR_EL1.matches_all(SCTLR_EL1::M::Enable)
    }
}
