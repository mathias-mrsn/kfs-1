//! https://stackoverflow.com/questions/3215878/what-are-in-out-instructions-in-x86-used-for
//! https://c9x.me/x86/html/file_module_x86_id_139.html
//! https://c9x.me/x86/html/file_module_x86_id_222.html
//!
//! On x86, legacy hardware and some low-level devices are often accessed
//! through Port-Mapped I/O (PMIO).
//!
//! PMIO is a mechanism for performing input/output between the CPU and
//! peripheral devices through a dedicated I/O address space, separate from
//! the normal memory address space. The `in` and `out` instructions are used
//! for this purpose: `in` reads data from an I/O port, and `out` writes data
//! to an I/O port.
//!
//! Architecturally, x86 I/O ports use a 16-bit address space, which allows
//! up to 65,536 ports (2^16). This is different from Memory-Mapped I/O
//! (MMIO), where devices are accessed through normal memory loads and
//! stores.
//!
//! PMIO is commonly used for legacy x86 devices such as the PIC, PIT, serial
//! ports, keyboard controller, and some VGA registers.
use core::arch::asm;

/// Reads one byte from the specified I/O port.
///
/// # Safety
/// The caller must ensure that `port` is valid for an 8-bit read and that
/// performing this access is safe for the current hardware state.
#[inline(always)]
pub unsafe fn inb(port: u16) -> u8
{
        let output: u8;
        unsafe {
                asm!("in al, dx", out("al") output, in("dx") port);
        }
        output
}

/// Reads one 32-bit double word from the specified I/O port.
///
/// # Safety
/// The caller must ensure that `port` is valid for a 32-bit read and that
/// performing this access is safe for the current hardware state.
#[inline(always)]
pub unsafe fn outb(
        port: u16,
        value: u8,
)
{
        asm!("out dx, al", in("dx") port, in("al") value);
}

/// Reads one byte from the specified I/O port.
///
/// # Safety
/// The caller must ensure that `port` is valid for an 8-bit read and that
/// performing this access is safe for the current hardware state.
#[inline(always)]
pub unsafe fn inw(port: u16) -> u16
{
        let output: u16;
        unsafe {
                asm!("in ax, dx", out("ax") output, in("dx") port);
        }
        output
}

/// Writes one byte to the specified I/O port.
///
/// # Safety
/// The caller must ensure that `port` is valid for an 8-bit write and that
/// performing this access is safe for the current hardware state.
#[inline(always)]
pub unsafe fn outw(
        port: u16,
        value: u16,
)
{
        asm!("out dx, ax", in("dx") port, in("ax") value);
}

/// Reads one 16-bit word from the specified I/O port.
///
/// # Safety
/// The caller must ensure that `port` is valid for a 16-bit read and that
/// performing this access is safe for the current hardware state.
#[inline(always)]
pub unsafe fn indw(port: u16) -> u32
{
        let output: u32;
        unsafe {
                asm!("in eax, dx", out("eax") output, in("dx") port);
        }
        output
}

/// Writes one 16-bit word to the specified I/O port.
///
/// # Safety
/// The caller must ensure that `port` is valid for a 16-bit write and that
/// performing this access is safe for the current hardware state.
#[inline(always)]
pub unsafe fn outdw(
        port: u16,
        value: u32,
)
{
        asm!("out dx, eax", in("dx") port, in("eax") value);
}
