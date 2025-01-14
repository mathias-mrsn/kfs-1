use core::arch::asm;

/// Reads the CR0 control register value.
///
/// The CR0 register contains system control flags that control operating mode
/// and states of the processor, including protected mode enable, paging, etc.
///
/// # Safety
/// This function is unsafe because reading CR0:
/// - Requires privileged access level
/// - Can affect system-wide CPU operation mode
/// - Should only be used in kernel-level code
#[inline(always)]
pub unsafe fn rdcr0() -> u32
{
    let out: u32;
    asm!("mov {:e}, cr0 ",
        out(reg) out,
        options(readonly, nostack, preserves_flags)
    );
    out
}
