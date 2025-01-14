use crate::cpu::gdt::DescriptorTablePointer;
use core::arch::asm;

/// Stores the current Global Descriptor Table Register (GDTR) value.
///
/// This function executes the SGDT instruction to retrieve the current GDT
/// limit and base address. Unlike LGDT, this operation is allowed at any
/// privilege level.
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
/// This function executes the LGDT instruction to set up a new GDT using the
/// provided descriptor table pointer.
///
/// # Safety
/// This function is unsafe because:
/// - Loading an invalid GDT can cause system crashes
/// - It can break memory segmentation if configured incorrectly
///
/// # Arguments
/// * `ptr` - Reference to a `DescriptorTablePointer` containing the new GDT's
///   limit and base address
#[inline(always)]
pub unsafe fn lgdt(ptr: &DescriptorTablePointer)
{
    unsafe {
        asm!(" lgdt [{}] ", in(reg) ptr, options(readonly, nostack, preserves_flags));
    }
}
