use tock_registers::{
    interfaces::{Readable, Writeable},
    registers::InMemoryRegister,
};

use crate::{
    bsp,
    memory::mmu::{
        arch_mmu::{GranuleSize512MiB, GranuleSize64KiB},
        AttributeFields, MemAttributes,
    },
    memory::{self, mmu::AccessPermissions},
};

// A table descriptor, as per ARMv8-A Architecture reference manual Figure D5-15.
register_bitfields! {u64,
    STAGE1_TABLE_DESCRIPTOR [
        // Physical address of the next descriptor
        NEXT_LEVEL_TABLE_ADDR_64KiB OFFSET(16) NUMBITS(32) [],

        TYPE OFFSET(1) NUMBITS(1) [
            Block = 0,
            Table = 1,
        ],

        VALID OFFSET(0) NUMBITS(1) [
            False = 0,
            True = 1,
        ]
    ]
}

// A page descriptor, as per ARMv8-A Architecture reference manual Figure D5-17
register_bitfields! {u64,
    STAGE1_PAGE_DESCRIPTOR [
        // Unprivilleged execute-never
        UXN OFFSET(54) NUMBITS(1) [
            False = 0,
            True = 1,
        ],

        // Privilleged execute-never
        PXN OFFSET(53) NUMBITS(1) [
            False = 0,
            True = 1,
        ],

        // Physical address of the next table descriptor (lvl2) or the page descriptor (lvl3)
        OUTPUT_ADDR_64KiB OFFSET(16) NUMBITS(32) [],

        // Access flag
        AF OFFSET(10) NUMBITS(1) [
            False = 0,
            True = 1,
        ],

        // Shareablitiy flag
        SH OFFSET(8) NUMBITS(2) [
            OuterShareable = 0b10,
            InnerShareable = 0b11,
        ],

        // Access permissions
        AP OFFSET(6) NUMBITS(2) [
            RW_EL1 = 0b00,
            RW_EL1_EL0 = 0b01,
            RO_EL1 = 0b10,
            RO_EL1_EL0 = 0b11,
        ],

        // Memory attributes index into the MAIR_EL1 register.
        AttrIdx OFFSET(2) NUMBITS(3) [],

        TYPE OFFSET(1) NUMBITS(1) [
            Reserved_Invalid = 0,
            Page = 1,
        ],

        VALID OFFSET(0) NUMBITS(1) [
            False = 0,
            True = 1,
        ]
    ]
}

// A table descriptor for 64KiB aperture
#[derive(Clone, Copy)]
#[repr(C)]
struct TableDescriptor {
    value: u64,
}

// A table descriptor for 64KiB aperture
#[derive(Clone, Copy)]
#[repr(C)]
struct PageDescriptor {
    value: u64,
}

trait StartAddr {
    fn phys_start_addr_u64(&self) -> u64;
    fn phys_start_addr_usize(&self) -> usize;
}

const NUM_LVL2_TABLES: usize = bsp::memory::mmu::KernelAddrSpace::SIZE >> GranuleSize512MiB::SHIFT;

// Single big struct to hold all the translation tables. Individual levels must be 64KiB aligned
// so the lvl3 is put first
#[repr(C)]
#[repr(align(65536))]
pub struct FixedSizeTranslationTable<const NUM_TABLES: usize> {
    // Page descriptors, covering 64 KiB windows per entry
    lvl3: [[PageDescriptor; 8192]; NUM_TABLES],

    // Table descriptors, covering 512 MiB windows
    lvl2: [TableDescriptor; NUM_TABLES],
}

// A translation table for the kernel space
pub type KernelTranslationTable = FixedSizeTranslationTable<NUM_LVL2_TABLES>;

// The binary is sill identity mapped, so we don't need to convert here
impl<T, const N: usize> StartAddr for [T; N] {
    fn phys_start_addr_u64(&self) -> u64 {
        self as *const _ as u64
    }

    fn phys_start_addr_usize(&self) -> usize {
        self as *const _ as usize
    }
}

impl TableDescriptor {
    // Creates a new invalid descriptor
    pub const fn new_zeroed() -> Self {
        Self { value: 0 }
    }

    // Create an instance pointing ot the supplied address
    pub fn from_next_lvl_table_addr(phys_next_lvl_table_addr: usize) -> Self {
        let val = InMemoryRegister::<u64, STAGE1_TABLE_DESCRIPTOR::Register>::new(0);

        let shifted = phys_next_lvl_table_addr >> GranuleSize64KiB::SHIFT;
        val.write(
            STAGE1_TABLE_DESCRIPTOR::NEXT_LEVEL_TABLE_ADDR_64KiB.val(shifted as u64)
                + STAGE1_TABLE_DESCRIPTOR::TYPE::Table
                + STAGE1_TABLE_DESCRIPTOR::VALID::True,
        );

        TableDescriptor { value: val.get() }
    }
}

// Convert the kernel's generic memory attributes to HW-specific attributes of the MMU
impl From<AttributeFields>
    for tock_registers::fields::FieldValue<u64, STAGE1_PAGE_DESCRIPTOR::Register>
{
    fn from(attribute_fields: AttributeFields) -> Self {
        // Memory attributes
        let mut desc = match attribute_fields.mem_attributes {
            MemAttributes::CacheableDRAM => {
                STAGE1_PAGE_DESCRIPTOR::SH::InnerShareable
                    + STAGE1_PAGE_DESCRIPTOR::AttrIdx.val(memory::mmu::arch_mmu::mair::NORMAL)
            }
            MemAttributes::Device => {
                STAGE1_PAGE_DESCRIPTOR::SH::OuterShareable
                    + STAGE1_PAGE_DESCRIPTOR::AttrIdx.val(memory::mmu::arch_mmu::mair::DEVICE)
            }
        };

        // Access permissions
        desc += match attribute_fields.acc_perms {
            AccessPermissions::ReadOnly => STAGE1_PAGE_DESCRIPTOR::AP::RO_EL1,
            AccessPermissions::ReadWrite => STAGE1_PAGE_DESCRIPTOR::AP::RW_EL1,
        };

        // The execute-never attribute is mapped to PXN in AArch64
        desc += if attribute_fields.execute_never {
            STAGE1_PAGE_DESCRIPTOR::PXN::True
        } else {
            STAGE1_PAGE_DESCRIPTOR::PXN::False
        };

        // Always set the un-privileged execute-never as long as userspace is not implemented
        desc += STAGE1_PAGE_DESCRIPTOR::UXN::True;

        desc
    }
}

impl PageDescriptor {
    // Create a new invalid descriptor
    pub const fn new_zeroed() -> Self {
        Self { value: 0 }
    }

    // Create an instance
    pub fn from_output_addr(phys_output_addr: usize, attribute_fields: &AttributeFields) -> Self {
        let val = InMemoryRegister::<u64, STAGE1_PAGE_DESCRIPTOR::Register>::new(0);

        let shifted = phys_output_addr as u64 >> GranuleSize64KiB::SHIFT;
        val.write(
            STAGE1_PAGE_DESCRIPTOR::OUTPUT_ADDR_64KiB.val(shifted)
                + STAGE1_PAGE_DESCRIPTOR::AF::True
                + STAGE1_PAGE_DESCRIPTOR::TYPE::Page
                + STAGE1_PAGE_DESCRIPTOR::VALID::True
                + (*attribute_fields).into(),
        );

        Self { value: val.get() }
    }
}

impl<const NUM_TABLES: usize> FixedSizeTranslationTable<NUM_TABLES> {
    pub const fn new() -> Self {
        assert!(NUM_TABLES > 0);

        Self {
            lvl3: [[PageDescriptor::new_zeroed(); 8192]; NUM_TABLES],
            lvl2: [TableDescriptor::new_zeroed(); NUM_TABLES],
        }
    }

    // Iterates over all static translation table entries and fills them at once
    // Safety: Modifies a static mut (hence unsafe) - Ensure this only happens from here
    pub unsafe fn populate_tt_entries(&mut self) -> Result<(), &'static str> {
        for (l2_nr, l2_entry) in self.lvl2.iter_mut().enumerate() {
            *l2_entry =
                TableDescriptor::from_next_lvl_table_addr(self.lvl3[l2_nr].phys_start_addr_usize());

            for (l3_nr, l3_entry) in self.lvl3[l2_nr].iter_mut().enumerate() {
                let virt_addr =
                    (l2_nr << GranuleSize512MiB::SHIFT) + (l3_nr << GranuleSize64KiB::SHIFT);

                let (phys_output_addr, attribute_fields) =
                    bsp::memory::mmu::virt_mem_layout().virt_addr_properties(virt_addr)?;

                *l3_entry = PageDescriptor::from_output_addr(phys_output_addr, &attribute_fields);
            }
        }

        Ok(())
    }

    // The translation table's base address to be used for programming the MMU
    pub fn phys_base_address(&self) -> u64 {
        self.lvl2.phys_start_addr_u64()
    }
}
