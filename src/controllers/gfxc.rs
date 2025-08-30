/// This module provides low-level access to the VGA Graphics Controller
/// registers.
use crate::instructions::io::{inb, outb};

/// VGA Graphics Controller Address Register port
pub const AR_PORT: u16 = 0x3CE;
/// VGA Graphics Controller Data Register port
pub const DR_PORT: u16 = 0x3CF;

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

#[inline(always)]
pub fn read(index: Indexes) -> u8
{
        unsafe {
                outb(AR_PORT, index as u8);
                inb(DR_PORT)
        }
}
