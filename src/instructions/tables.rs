use crate::cpu::DescriptorTablePointer;
use core::arch::asm;

/// Stores the current Global Descriptor Table Register (GDTR) value.
///
/// # Returns
/// Returns a `DescriptorTablePointer` containing the current GDT's limit and
/// base address.
#[inline(always)]
pub fn sgdt() -> DescriptorTablePointer
{
    let mut gdt: DescriptorTablePointer = DescriptorTablePointer {
        limit: 0,
        base:  0x00 as _,
    };
    unsafe {
        asm!(" sgdt [{}] ", in(reg) &mut gdt, options(nostack, preserves_flags));
    }
    gdt
}

/// Loads a new Global Descriptor Table (GDT).
///
/// # Safety
/// This function is unsafe because:
/// - Loading an invalid GDT can cause system crashes
/// - It can break memory segmentation if configured incorrectly
#[unsafe(no_mangle)]
#[inline(always)]
pub unsafe fn lgdt(ptr: &DescriptorTablePointer)
{
    unsafe {
        asm!(" lgdt [{}] ", in(reg) ptr, options(readonly, nostack, preserves_flags));
    }
}

/// Loads a new Interrupt Descriptor Table (IDT)
///
/// # Safety
/// This function is unsafe because:
/// - Loading an invalid IDT can cause system crashes
/// - It can break interrupt handling if configured incorrectly
#[inline(always)]
pub unsafe fn lidt(ptr: &DescriptorTablePointer)
{
    unsafe {
        asm!(" lidt [{}] ", in(reg) ptr, options(readonly, nostack, preserves_flags));
    }
}

/// Stores the current Interrupt Descriptor Table Register (IDTR) value
///
/// # Returns
/// Returns a `DescriptorTablePointer` containing the current IDT's limit and
/// base address
#[inline(always)]
pub fn sidt() -> DescriptorTablePointer
{
    let mut idt: DescriptorTablePointer = DescriptorTablePointer {
        limit: 0,
        base:  0x00 as _,
    };
    unsafe {
        asm!(" sidt [{}] ", in(reg) &mut idt, options(nostack, preserves_flags));
    }
    idt
}
