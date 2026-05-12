use core::arch::asm;

const EFLAGS_INTERRUPT_FLAG: u32 = 1 << 9;

/// Returns the current EFLAGS register value.
#[inline]
fn read_eflags() -> u32
{
    let flags: u32;

    // SAFETY: Reading the current CPU flags with pushfd/pop does not modify
    // architectural state other than the temporary stack slot used by the
    // instructions themselves.
    unsafe {
        asm!("pushfd", "pop {flags:e}", flags = out(reg) flags, options(nomem, preserves_flags));
    }

    flags
}

/// Returns whether maskable interrupts are currently enabled.
#[inline]
pub fn interrupts_enabled() -> bool { (read_eflags() & EFLAGS_INTERRUPT_FLAG) != 0 }

/// Runs `f` with maskable interrupts disabled on the current CPU.
///
/// If interrupts were already disabled, their state is preserved.
#[inline]
pub fn without_interrupts<R>(f: impl FnOnce() -> R) -> R
{
    let were_enabled = interrupts_enabled();

    if were_enabled {
        // SAFETY: The caller of `without_interrupts` is requesting a critical
        // section that must not be interrupted by maskable interrupts.
        unsafe {
            cli();
        }
    }

    let result = f();

    if were_enabled {
        // SAFETY: Interrupts were enabled on entry, so restoring them preserves
        // the prior CPU state.
        unsafe {
            sti();
        }
    }

    result
}

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
