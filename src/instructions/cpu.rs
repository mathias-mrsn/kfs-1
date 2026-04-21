use core::arch::asm;

/// Disables maskable interrupts on the current CPU.
///
/// # Safety
/// The caller must ensure that interrupts may be disabled here without
/// breaking kernel synchronization, forward progress, or interrupt-handling
/// invariants.
#[inline]
pub unsafe fn cli()
{
        asm!("cli", options(readonly, nostack, preserves_flags));
}

/// Enables maskable interrupts on the current CPU.
///
/// # Safety
/// The caller must ensure that interrupt handlers may safely run after this
/// point and that all required CPU and kernel state has been initialized.
#[inline]
pub unsafe fn sti()
{
        asm!("sti", options(readonly, nostack, preserves_flags));
}
