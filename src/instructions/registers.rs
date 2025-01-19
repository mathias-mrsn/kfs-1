use core::arch::asm;

/// Reads the CR0 (Control Register 0) register value
///
/// Returns the current control register 0 value as a 32-bit value
#[inline(always)]
pub fn rdcr0() -> u32
{
    let out: u32;
    unsafe {
        asm!("mov {:e}, cr0",
            out(reg) out,
            options(readonly, nostack, preserves_flags)
        );
    }
    out
}

/// Reads the CS (Code Segment) register value
///
/// Returns the current code segment selector as a 16-bit value
#[inline]
pub fn rdcs() -> u16
{
    let out: u16;
    unsafe {
        asm!("mov {:x}, cs",
            out(reg) out,
            options(readonly, nostack, preserves_flags)
        );
    }
    out
}
