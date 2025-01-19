use core::arch::asm;

/// Reads a byte (8 bits) from the specified I/O port.
///
/// # Safety
/// This function is unsafe because it performs direct I/O port access.
#[inline(always)]
pub unsafe fn inb(port: u16) -> u8
{
    let output: u8;
    unsafe {
        asm!("in al, dx", out("al") output, in("dx") port);
    }
    output
}

/// Writes a byte (8 bits) to the specified I/O port.
///
/// # Safety
/// This function is unsafe because it performs direct I/O port access.
#[inline(always)]
pub unsafe fn outb(
    port: u16,
    value: u8,
)
{
    asm!("out dx, al", in("dx") port, in("al") value);
}

/// Reads a word (16 bits) from the specified I/O port.
///
/// # Safety
/// This function is unsafe because it performs direct I/O port access.
#[inline(always)]
pub unsafe fn inw(port: u16) -> u16
{
    let output: u16;
    unsafe {
        asm!("in ax, dx", out("ax") output, in("dx") port);
    }
    output
}

/// Writes a word (16 bits) to the specified I/O port.
///
/// # Safety
/// This function is unsafe because it performs direct I/O port access.
#[inline(always)]
pub unsafe fn outw(
    port: u16,
    value: u16,
)
{
    asm!("out dx, ax", in("dx") port, in("ax") value);
}

/// Reads a double word (32 bits) from the specified I/O port.
///
/// # Safety
/// This function is unsafe because it performs direct I/O port access.
#[inline(always)]
pub unsafe fn indw(port: u16) -> u32
{
    let output: u32;
    unsafe {
        asm!("in eax, dx", out("eax") output, in("dx") port);
    }
    output
}

/// Writes a double word (32 bits) to the specified I/O port.
///
/// # Safety
/// This function is unsafe because it performs direct I/O port access.
#[inline(always)]
pub unsafe fn outdw(
    port: u16,
    value: u32,
)
{
    asm!("out dx, eax", in("dx") port, in("eax") value);
}
