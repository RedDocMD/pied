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

use core::{fmt, ops::RangeInclusive};

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
    max_virt_addr_inclusive: usize,

    // Array of descriptors for non-standard (normal cacheable DRAM) memory regions
    inner: [TranslationDescriptor; NUM_SPECIAL_RANGES],
}

impl fmt::Display for MMUEnableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MMUEnableError::AlreadyEnabled => write!(f, "MMU is already enabled"),
            MMUEnableError::Other(x) => write!(f, "{}", x),
        }
    }
}

impl<const GRANULE_SIZE: usize> TranslationGranule<GRANULE_SIZE> {
    pub const SIZE: usize = Self::size_checked();

    pub const SHIFT: usize = Self::SIZE.trailing_zeros() as usize;

    const fn size_checked() -> usize {
        assert!(GRANULE_SIZE.is_power_of_two());
        GRANULE_SIZE
    }
}

impl<const AS_SIZE: usize> AddressSpace<AS_SIZE> {
    pub const SIZE: usize = Self::size_checked();

    pub const SHIFT: usize = Self::SIZE.trailing_zeros() as usize;

    const fn size_checked() -> usize {
        assert!(AS_SIZE.is_power_of_two());
        Self::arch_address_space_size_sanity_check();

        AS_SIZE
    }
}

impl Default for AttributeFields {
    fn default() -> Self {
        Self {
            mem_attributes: MemAttributes::CacheableDRAM,
            acc_perms: AccessPermissions::ReadWrite,
            execute_never: true,
        }
    }
}

impl fmt::Display for TranslationDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let start = *(self.virtual_range)().start();
        let end = *(self.virtual_range)().end();
        let size = end - start + 1;

        const KIB_SHIFT: u32 = 10;
        const MIB_SHIFT: u32 = 20;

        let (size, unit) = if (size >> MIB_SHIFT) > 0 {
            (size >> MIB_SHIFT, "MiB")
        } else if (size >> KIB_SHIFT) > 0 {
            (size >> KIB_SHIFT, "KiB")
        } else {
            (size, "Byte")
        };

        let attr = match self.attribute_fields.mem_attributes {
            MemAttributes::CacheableDRAM => "C",
            MemAttributes::Device => "Dev",
        };

        let acc_p = match self.attribute_fields.acc_perms {
            AccessPermissions::ReadOnly => "RO",
            AccessPermissions::ReadWrite => "RW",
        };

        let xn = if self.attribute_fields.execute_never {
            "PXN"
        } else {
            "PX"
        };

        write!(
            f,
            "    {:#010x} - {:#010x} | {: >3} {} | {: <3} {} {: <3} | {}",
            start, end, size, unit, attr, acc_p, xn, self.name
        )
    }
}

impl<const NUM_SPECIAL_RANGES: usize> KernelVirtualLayout<NUM_SPECIAL_RANGES> {
    pub const fn new(max: usize, layout: [TranslationDescriptor; NUM_SPECIAL_RANGES]) -> Self {
        Self {
            max_virt_addr_inclusive: max,
            inner: layout,
        }
    }

    // For a virtual address, find and return the physical output address and corresponding
    // attributes.
    //
    // If the address is not found in `inner`, return an identity mapped default with normal
    // cacheable DRAM attributes.
    pub fn virt_addr_properties(
        &self,
        virt_addr: usize,
    ) -> Result<(usize, AttributeFields), &'static str> {
        if virt_addr > self.max_virt_addr_inclusive {
            return Err("Address out of range");
        }

        for i in self.inner.iter() {
            if (i.virtual_range)().contains(&virt_addr) {
                let output_addr = match i.physical_range_translation {
                    Translation::Identity => virt_addr,
                    Translation::Offset(a) => a + (virt_addr - (i.virtual_range)().start()),
                };

                return Ok((output_addr, i.attribute_fields));
            }
        }

        Ok((virt_addr, AttributeFields::default()))
    }

    // Print the memory layout.
    pub fn print_layout(&self) {
        use crate::kinfo;

        for i in self.inner.iter() {
            kinfo!("{}", i);
        }
    }
}
