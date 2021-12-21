use core::ops::RangeInclusive;

use super::map as memory_map;
use crate::memory::mmu::*;

pub type KernelAddrSpace = AddressSpace<{ memory_map::END_INCLUSIVE + 1 }>;

const NUM_MEM_RANGES: usize = 2;

// The virtual memory layout
// The layout must contain only special ranges, ie, anything that is _not_ noermal cacheable DRAM
// It is agnostic of the paging granularity that the architecture's MMU will use.
pub static LAYOUT: KernelVirtualLayout<NUM_MEM_RANGES> = KernelVirtualLayout::new(
    memory_map::END_INCLUSIVE,
    [
        TranslationDescriptor {
            name: "Kernel code and RO data",
            virtual_range: code_range_inclusive,
            physical_range_translation: Translation::Identity,
            attribute_fields: AttributeFields {
                mem_attributes: MemAttributes::CacheableDRAM,
                acc_perms: AccessPermissions::ReadOnly,
                execute_never: false,
            },
        },
        TranslationDescriptor {
            name: "Device MMIO",
            virtual_range: mmio_range_inclusive,
            physical_range_translation: Translation::Identity,
            attribute_fields: AttributeFields {
                mem_attributes: MemAttributes::Device,
                acc_perms: AccessPermissions::ReadWrite,
                execute_never: true,
            },
        },
    ],
);

fn code_range_inclusive() -> RangeInclusive<usize> {
    RangeInclusive::new(super::code_start(), super::code_end_exclusive() - 1)
}

fn mmio_range_inclusive() -> RangeInclusive<usize> {
    RangeInclusive::new(memory_map::mmio::START, memory_map::mmio::END_INCLUSIVE)
}

pub fn virt_mem_layout() -> &'static KernelVirtualLayout<NUM_MEM_RANGES> {
    &LAYOUT
}
