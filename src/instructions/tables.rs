use crate::cpu::gdt::DescriptorTablePointer;
use core::arch::asm;

#[inline(always)]
pub fn sgdt() -> DescriptorTablePointer
{
    let mut gdt: DescriptorTablePointer = DescriptorTablePointer {
        limit: 0,
        base:  0x00 as _,
    };
    unsafe {
        asm!(r#"
            sgdt [{}]
        "#, in(reg) &mut gdt, options(nostack, preserves_flags));
    }
    gdt
}

#[inline(always)]
pub unsafe fn lgdt(ptr: &DescriptorTablePointer)
{
    unsafe {
        asm!(r#"
            lgdt [{}]
        "#, in(reg) ptr, options(readonly, nostack, preserves_flags));
    }
}
