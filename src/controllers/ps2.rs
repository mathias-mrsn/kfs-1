/// This module provides low-level access to the PS2 Controller
use crate::controllers::{inb, outb};

/// PS2 Controller Command port
pub const C_PORT: u16 = 0x64;
/// PS2 Controller Data port
pub const D_PORT: u16 = 0x60;

/// Write a value to a PS2 Controller
///
/// # Arguments
/// * `value` - The value to write
///
/// # Safety
/// This function performs direct hardware I/O and should only be called
/// when appropriate hardware access is guaranteed.
#[inline(always)]
pub fn write(value: u8)
{
    unsafe {
        outb(D_PORT, value);
    }
}

/// Read a value from a PS2 Controller
///
/// # Returns
/// The value read from the specified register
///
/// # Safety
/// This function performs direct hardware I/O and should only be called
/// when appropriate hardware access is guaranteed.
#[inline(always)]
pub fn read() -> u8
{
    unsafe { inb(D_PORT) }
}
