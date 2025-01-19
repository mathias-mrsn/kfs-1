use core::arch::asm;

/// Disables CPU interrupts (Clear Interrupt Flag)
///
/// # Safety
/// This function is unsafe as it directly manipulates CPU interrupt state
#[inline]
pub unsafe fn cli()
{
    asm!("cli", options(readonly, nostack, preserves_flags));
}

/// Enables CPU interrupts (Set Interrupt Flag)
///
/// # Safety
/// This function is unsafe as it directly manipulates CPU interrupt state
#[inline]
pub unsafe fn sti()
{
    asm!("sti", options(readonly, nostack, preserves_flags));
}
