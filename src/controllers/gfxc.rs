/// This module provides low-level access to the VGA Graphics Controller
/// registers.
use crate::controllers::{inb, outb};

/// VGA Graphics Controller Address Register port
pub const AR_PORT: u16 = 0x3CE;
/// VGA Graphics Controller Data Register port
pub const DR_PORT: u16 = 0x3CF;

/// Graphics Controller register indexes
///
/// These indexes are used to select which Graphics Controller register to
/// access when using the `write` and `read` functions. These indexes are
/// output inside the AR_PORT Port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Indexes
{
    /// Set/Reset Register
    SrValue      = 0x00,
    /// Enable Set/Reset Register
    SrEnable     = 0x01,
    /// VGAColor Compare Register
    CompareValue = 0x02,
    /// Data Rotate Register
    DataRotate   = 0x03,
    /// Read Map Select Register
    PlaneRead    = 0x04,
    /// Graphics Mode Register
    Mode         = 0x05,
    /// Miscellaneous Graphics Register
    Misc         = 0x06,
    /// VGAColor Don't Care Register
    CompareMask  = 0x07,
    /// Bit Mask Register
    BitMask      = 0x08,
}

/// Write a value to a Graphics Controller register
///
/// # Arguments
/// * `index` - The register index to write to
/// * `value` - The value to write
///
/// # Safety
/// This function performs direct hardware I/O and should only be called
/// when appropriate hardware access is guaranteed.
#[inline(always)]
pub fn write(
    index: Indexes,
    value: u8,
)
{
    unsafe {
        outb(AR_PORT, index as u8);
        outb(DR_PORT, value);
    }
}

/// Read a value from a Graphics Controller register
///
/// # Arguments
/// * `index` - The register index to read from
///
/// # Returns
/// The value read from the specified register
///
/// # Safety
/// This function performs direct hardware I/O and should only be called
/// when appropriate hardware access is guaranteed.
#[inline(always)]
pub fn read(index: Indexes) -> u8
{
    unsafe {
        outb(AR_PORT, index as u8);
        inb(DR_PORT)
    }
}
