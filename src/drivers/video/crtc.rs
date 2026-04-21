//! VGA Cathode Ray Tube Controller (CRTC) access.
//!
//! The CRTC is the VGA hardware block responsible for display timing and for
//! selecting how the display is scanned out from video memory.
//!
//! In text mode, the CRTC is the block used to:
//! - choose which part of VRAM is currently visible through the start address,
//! - control the hardware text cursor position,
//! - control the hardware text cursor shape,
//! - expose timing and geometry-related registers used when changing the text
//!   mode layout.
//!
//! This module assumes a color VGA-compatible CRTC accessed through the I/O
//! port pair `0x3D4`/`0x3D5`:
//! - `0x3D4`: register index port,
//! - `0x3D5`: register data port.
//!
//! Access is stateful: the register index must be written first, then the
//! register value is read or written through the data port. Because of this,
//! CRTC accesses must not be interleaved with other VGA register accesses.
//!
//! Reference: http://www.osdever.net/FreeVGA/vga/crtcreg.htm

use crate::instructions::io::{inb, outb};

const INDEX_PORT: u16 = 0x3D4;
const DATA_PORT: u16 = 0x3D5;

/// VGA CRTC register indices.
///
/// These values select registers in the CRTC indexed I/O interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum Register
{
        // Horizontal timing
        HorizontalTotal         = 0x00,
        HorizontalDisplayEnd    = 0x01,
        HorizontalBlankingStart = 0x02,
        HorizontalBlankingEnd   = 0x03,
        HorizontalRetraceStart  = 0x04,
        HorizontalRetraceEnd    = 0x05,
        // Vertical timing
        VerticalTotal           = 0x06,
        /// Carries high bits for several vertical timing registers.
        Overflow                = 0x07,
        /// Controls row scan offset within a character cell.
        PresetRowScan           = 0x08,
        /// Controls character height and scanline-related mode bits.
        MaximumScanLine         = 0x09,
        // Cursor and display start
        CursorStart             = 0x0A,
        CursorEnd               = 0x0B,
        StartAddressHigh        = 0x0C,
        StartAddressLow         = 0x0D,
        CursorLocationHigh      = 0x0E,
        CursorLocationLow       = 0x0F,
        // Retrace and display window
        VerticalRetraceStart    = 0x10,
        VerticalRetraceEnd      = 0x11,
        VerticalDisplayEnd      = 0x12,
        /// Logical line stride in words.
        Offset                  = 0x13,
        UnderlineLocation       = 0x14,
        VerticalBlankingStart   = 0x15,
        VerticalBlankingEnd     = 0x16,
        ModeControl             = 0x17,
        /// Compares the current scanline against this value.
        LineCompare             = 0x18,
}

/// Writes `value` to the selected VGA controller register.
///
/// # Safety
/// Callers must ensure that the value being written is valid for the selected
/// register, cause this value may cause undefined behavior in the VGA hardware
/// if invalid.
#[inline(always)]
pub(super) unsafe fn write(
        reg: Register,
        value: u8,
)
{
        outb(INDEX_PORT, reg as u8);
        outb(DATA_PORT, value);
}

/// Reads the value of the selected VGA controller register.
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
