use super::RegisterAccessor;
use bitflags::bitflags;
use core::arch::asm;

pub struct CR0;

bitflags! {
    #[repr(transparent)]
    #[derive(PartialEq, Eq, Debug, Clone, Copy)]
    pub struct CR0Flags: u32 {
        const WP = 1 << 16;
        const PG = 1 << 31;
    }
}

impl core::fmt::Debug for CR0
{
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result
    {
        f.debug_struct("CR0")
            .field("WP", &CR0::read_bit(CR0Flags::WP))
            .field("PG", &CR0::read_bit(CR0Flags::PG))
            .finish()
    }
}

impl RegisterAccessor<u32> for CR0
{
    type Flags = CR0Flags;

    #[inline]
    fn read() -> Self::Flags { Self::Flags::from_bits_truncate(Self::read_raw()) }

    fn read_raw() -> u32
    {
        let out: u32;
        unsafe {
            asm!("mov {:e}, cr0",
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
        asm!("mov cr0, {:e}",
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
            r | if b == true {
                f
            } else {
                Self::Flags::from_bits_truncate(0)
            },
        );
    }
}
