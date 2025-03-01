use crate::commun::{ConstDefault, ConstFrom, ConstInto};
use bitflags::bitflags;
use core::mem;

use crate::memory::addr::PhysAddr;

use usize as EntryType;

pub const PAGES_TABLES_SIZE: usize = 0x40_0000;

/// Page Directory Table Physical Address
pub const PDT_PHYS: PhysAddr = PhysAddr::from_const(0x1000);
/// Page Directory Table Size
pub const PDT_SIZE: usize = mem::size_of::<PDT>();

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PDE(EntryType);

impl PDE
{
    const FLAG_MASK_4MB: EntryType = 0x20_1fff;
    const FLAG_MASK_4KB: EntryType = 0x1fff;
    const ADDR_MASK_4MB: EntryType = 0xffc0_0000;
    const ADDR_MASK_4KB: EntryType = 0xffff_f000;

    pub const fn new(
        p: PhysAddr,
        f: PDEFlags,
    ) -> Self
    {
        let mut e: EntryType = 0;
        if (f.bits() & PDEFlags::PAGE_SIZE.bits()) != 0 {
            e = p.inner() & Self::ADDR_MASK_4KB;
        } else {
            e = p.inner() & Self::ADDR_MASK_4MB;
        }
        Self(e | f.bits())
    }

    pub const fn flags(&self) -> PDEFlags
    {
        PDEFlags::from_bits_truncate(
            self.0
                & if (self.0 & PDEFlags::PAGE_SIZE.bits()) != 0 {
                    Self::FLAG_MASK_4MB
                } else {
                    Self::FLAG_MASK_4KB
                },
        )
    }

    pub const fn address(&self) -> PhysAddr
    {
        PhysAddr::from_const(if (self.0 & PDEFlags::PAGE_SIZE.bits()) != 0 {
            self.0 & Self::ADDR_MASK_4MB
        } else {
            self.0 & Self::ADDR_MASK_4KB
        })
    }
}

impl const ConstDefault for PDE
{
    fn default_const() -> Self { Self(0) }
}

#[repr(C)]
pub struct PDT
{
    pub user_space:   [PDE; 768],
    pub kernel_space: [PDE; 1024 - 768],
}

impl const ConstDefault for PDT
{
    fn default_const() -> Self
    {
        Self {
            user_space:   [PDE::default_const(); 768],
            kernel_space: [PDE::default_const(); 1024 - 768],
        }
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(PartialEq, Eq, Debug, Clone, Copy)]
    pub struct PDEFlags: usize {
        const PRESENT = 1 << 0;
        const READ_WRITE = 1 << 1;
        const USER = 1 << 2;
        const WRITE_THROUGH = 1 << 3;
        const CACHE_DISABLE = 1 << 4;
        const ACCESSED = 1 << 5;
        const DIRTY = 1 << 6;
        const AVAILABLE = 1 << 6;
        const PAGE_SIZE = 1 << 7;
        const GLOBAL = 1 << 8;
        const BIT_8 = 1 << 8;
        const BIT_9 = 1 << 9;
        const BIT_10 = 1 << 10;
        const BIT_11 = 1 << 11;
        const PAGE_ATTRIBUTE_TABLE = 1 << 12;
        const BIT_21 = 1 << 21;
    }
}
