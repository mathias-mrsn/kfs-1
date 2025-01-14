/**
 * https://stackoverflow.com/questions/3215878/what-are-in-out-instructions-in-x86-used-for
 * https://c9x.me/x86/html/file_module_x86_id_139.html
 * https://c9x.me/x86/html/file_module_x86_id_222.html
 */
use core::arch::asm;

/// Reads a byte (8 bits) from the specified I/O port.
///
/// # Safety
/// This function is unsafe because it performs direct I/O port access.
///
/// # Arguments
/// * `port` - The I/O port number to read from
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
///
/// # Arguments
/// * `port` - The I/O port number to write to
/// * `value` - The byte value to write
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
///
/// # Arguments
/// * `port` - The I/O port number to read from
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
///
/// # Arguments
/// * `port` - The I/O port number to write to
/// * `value` - The word value to write
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
///
/// # Arguments
/// * `port` - The I/O port number to read from
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
///
/// # Arguments
/// * `port` - The I/O port number to write to
/// * `value` - The double word value to write
#[inline(always)]
pub unsafe fn outdw(
    port: u16,
    value: u32,
)
{
    asm!("out dx, eax", in("dx") port, in("eax") value);
}
