use super::RegisterAccessor;
use bitflags::bitflags;
use core::arch::asm;

pub struct IA32EFER;

bitflags! {
    #[repr(transparent)]
    #[derive(PartialEq, Eq, Debug, Clone, Copy)]
    pub struct IA32EFERFlags: u64 {
        const LME = 1 << 8;
        const NXE = 1 << 11;
    }
}

impl RegisterAccessor<u64> for IA32EFER
{
    type Flags = IA32EFERFlags;

    #[inline]
    fn read() -> Self::Flags { Self::Flags::from_bits_truncate(Self::read_raw()) }

    fn read_raw() -> u64
    {
        let low: u32;
        let high: u32;
        unsafe {
            asm!(
                "rdmsr",
                in("ecx") 0xC0000080u32, // The MSR address for IA32_EFER
                out("eax") low,
                out("edx") high,
            );
        }
        ((high as u64) << 32) | (low as u64)
    }

    fn read_bit(f: Self::Flags) -> bool
    {
        let r = Self::read_raw();
        r & f.bits() != 0
    }

    #[inline]
    unsafe fn write(f: Self::Flags) { Self::write_raw(f.bits()); }

    unsafe fn write_raw(v: u64)
    {
        asm!(
            "wrmsr",
            in("ecx") 0xC0000080u32, // The MSR address for IA32_EFER
            in("eax") (v as u32),
            in("edx") ((v >> 32) as u32),
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
