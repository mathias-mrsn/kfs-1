use crate::memory::addr::PhysAddr;

use super::RegisterAccessor;
use bitflags::bitflags;
use core::arch::asm;

pub struct CR3;

bitflags! {
    #[repr(transparent)]
    #[derive(PartialEq, Eq, Debug, Clone, Copy)]
    pub struct CR3Flags: u32 {
        const PWT = 1 << 3;
        const PCD = 1 << 4;
    }
}

impl core::fmt::Debug for CR3
{
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result
    {
        f.debug_struct("CR0")
            .field("WP", &CR3::read_bit(CR3Flags::PWT))
            .field("PG", &CR3::read_bit(CR3Flags::PCD))
            .finish()
    }
}

impl RegisterAccessor<u32> for CR3
{
    type Flags = CR3Flags;

    #[inline]
    fn read() -> Self::Flags { Self::Flags::from_bits_truncate(Self::read_raw()) }

    fn read_raw() -> u32
    {
        let out: u32;
        unsafe {
            asm!("mov {:e}, cr3",
                out(reg) out,
                options(readonly, nostack, preserves_flags)
            );
        }
        out
    }

    fn read_bit(f: Self::Flags) -> bool
    {
        let r = Self::read_raw();
        r & f.bits() != 0
    }

    #[inline]
    unsafe fn write(f: Self::Flags) { Self::write_raw(f.bits()); }

    unsafe fn write_raw(v: u32)
    {
        asm!("mov cr3, {:e}",
            in(reg) v,
            options(nostack, preserves_flags)
        );
    }

    unsafe fn write_bit(
        f: Self::Flags,
        b: bool,
    )
    {
        let r = Self::read() ^ f;
        Self::write(
            r & if b == true {
                f
            } else {
                Self::Flags::from_bits_truncate(0)
            },
        );
    }
}

impl CR3
{
    pub fn read_pdt() -> PhysAddr
    {
        let p = Self::read_raw();
        PhysAddr(p >> 12)
    }

    pub unsafe fn write_pdt(p: PhysAddr)
    {
        let cr3 = Self::read_raw() & 0xFFF;
        let p = p.0 << 12;
        Self::write_raw(cr3 | p);
    }
}
