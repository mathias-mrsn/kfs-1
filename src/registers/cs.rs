use super::RegisterAccessor;
use bitflags::bitflags;
use core::arch::asm;

pub struct CS;

bitflags! {
    #[repr(transparent)]
    #[derive(PartialEq, Eq, Debug, Clone, Copy)]
    pub struct CSFlags: u16 {}
}

impl RegisterAccessor<u16> for CS
{
    type Flags = CSFlags;

    #[inline]
    fn read() -> Self::Flags { Self::Flags::from_bits_truncate(Self::read_raw()) }

    fn read_raw() -> u16
    {
        let out: u16;
        unsafe {
            asm!("mov {:e}, cr4",
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

    unsafe fn write_raw(v: u16)
    {
        asm!("mov cr4, {:e}",
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
