mod translation_table;

#[derive(Debug)]
pub enum MMUEnableError {
    AlreadyEnabled,
    Other(&'static str),
}

mod interface {
    use super::*;

    pub trait MMU {
        // Called by the kernel during early init. Supposed to take the translation tables from the
        // BSP-supplied `virt_mem_layout` and install/activate them for the respective MMU.
        unsafe fn enable_mmu_and_caching(&self) -> Result<(), MMUEnableError>;

        fn is_enabled(&self) -> bool;
    }
}

use core::ops::RangeInclusive;

pub use interface::*;

// Describes the characterisitics of a translation granule
pub struct TranslationGranule<const GRANULE_SIZE: usize>;

// Describes properties of an address space
pub struct AddressSpace<const AS_SIZE: usize>;

// Architecture agnostic translation types
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Translation {
    Identity,
    Offset(usize),
}

// Architecture agnostic memory attributes
#[derive(Clone, Copy)]
pub enum MemAttributes {
    CacheableDRAM,
    Device,
}

// Architecture agnostic access permissions
#[derive(Clone, Copy)]
pub enum AccessPermissions {
    ReadOnly,
    ReadWrite,
}

// Collection of memory attributes
#[derive(Clone, Copy)]
pub struct AttributeFields {
    pub mem_attributes: MemAttributes,
    pub acc_perms: AccessPermissions,
    pub execute_never: bool,
}

// Architecture agnostic descriptor for a memory range
pub struct TranslationDescriptor {
    pub name: &'static str,
    pub virtual_range: fn() -> RangeInclusive<usize>,
    pub physical_range_translation: Translation,
    pub attribute_fields: AttributeFields,
}

// Type representing the kernel's virtual memory layout
pub struct KernelVirtualLayout<const NUM_SPECIAL_RANGES: usize> {
    // The last (inclusive) address of the address space
    max_virt_addr: usize,

    // Array of descriptors for non-standard (normal cacheable DRAM) memory regions
    inner: [TranslationDescriptor; NUM_SPECIAL_RANGES],
}
