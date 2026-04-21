//! VGA Graphics Controller access.
//!
//! The Graphics Controller is the VGA hardware block that controls how CPU
//! accesses to VGA memory are interpreted and how display memory is mapped.
//!
//! In this project, it is mainly used to configure the VGA memory map through
//! the Misc register, but the controller also exposes registers related to
//! read/write modes, plane selection, set/reset logic, and bit masking.
//!
//! This module assumes a VGA-compatible Graphics Controller accessed through
//! the I/O port pair `0x3CE`/`0x3CF`:
//! - `0x3CE`: register index port,
//! - `0x3CF`: register data port.
//!
//! Access is stateful: the register index must be written first, then the
//! register value is read or written through the data port. Because of this,
//! Graphics Controller accesses must not be interleaved with other VGA register
//! accesses.

use crate::instructions::io::{inb, outb};

const INDEX_PORT: u16 = 0x3CE;
const DATA_PORT: u16 = 0x3CF;

/// VGA Graphics Controller register indices.
///
/// These values select registers in the Graphics Controller indexed I/O
/// interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum Register
{
        // Set/reset and color compare
        SetResetValue     = 0x00,
        SetResetEnable    = 0x01,
        ColorCompareValue = 0x02,
        // Data path and plane selection
        DataRotate        = 0x03,
        ReadMapSelect     = 0x04,
        GraphicsMode      = 0x05,
        // Memory mapping and compare mask
        /// Selects the VGA memory map and related addressing behavior.
        Miscellaneous     = 0x06,
        ColorDontCareMask = 0x07,
        // Final write mask
        /// Selects which bits are affected during writes to VGA memory.
        BitMask           = 0x08,
}

/// Indexed register write primitive.
///
/// # Safety
/// Access must be valid and serialized.
#[inline(always)]
pub(super) unsafe fn write(
        reg: Register,
        value: u8,
)
{
        outb(INDEX_PORT, reg as u8);
        outb(DATA_PORT, value);
}

/// Indexed register read primitive.
#[inline(always)]
pub(super) fn read(reg: Register) -> u8
{
        // SAFETY: By using predefined register indices we unsure that unsafe functions
        // are used correctly, and wont be used to write to ports that are not
        // meant to be accessed.
        unsafe {
                outb(INDEX_PORT, reg as u8);
                inb(DATA_PORT)
        }
}
