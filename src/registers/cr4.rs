use super::RegisterAccessor;
use bitflags::bitflags;
use core::arch::asm;

pub struct CR4;

bitflags! {
    #[repr(transparent)]
    #[derive(PartialEq, Eq, Debug, Clone, Copy)]
    pub struct CR4Flags: u32 {
        const PSE = 1 << 4;
        const PAE = 1 << 5;
        const PGE = 1 << 7;
        const LA57 = 1 << 12;
        const PCIDE = 1 << 17;
        const SMEP = 1 << 20;
        const SMAP = 1 << 21;
        const PEK = 1 << 22;
        const CET = 1 << 23;
        const PKS = 1 << 24;
    }
}

impl RegisterAccessor<u32> for CR4
{
    type Flags = CR4Flags;

    #[inline]
    fn read() -> Self::Flags { Self::Flags::from_bits_truncate(Self::read_raw()) }

    fn read_raw() -> u32
    {
        let out: u32;
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

    unsafe fn write_raw(v: u32)
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
