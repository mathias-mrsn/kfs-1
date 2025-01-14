/// This module provides low-level access to the VGA CRT Controller
/// registers.
use crate::controllers::{inb, outb};

/// VGA CRT Controller Address Register port
pub const AR_PORT: u16 = 0x3D4;

/// VGA CRT Controller Address Data port
pub const DR_PORT: u16 = 0x3D5;

/// CRT Controller register indexes
///
/// These indexes are used to select which CRT Controller register to
/// access when using the `write` and `read` functions. These indexes are
/// output inside the AR_PORT Port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Indexes
{
    /// Horizontal Total Register
    HTotal      = 0x00,
    /// End Horizontal Display Register
    HDisp       = 0x01,
    /// Start Horizontal Blanking Register
    HBlankStart = 0x02,
    /// End Horizontal Blanking Register
    HBlankEnd   = 0x03,
    /// Start Horizontal Retrace Register
    HSyncStart  = 0x04,
    /// End Horizontal Retrace Register
    HSyncEnd    = 0x05,
    /// Vertical Total Register
    VTotal      = 0x06,
    /// Overflow Register
    Overflow    = 0x07,
    /// Preset Row Scan Register
    PresetRow   = 0x08,
    /// Maximum Scan Line Register
    MaxScan     = 0x09,
    /// Cursor Start Register
    CursorStart = 0x0A,
    /// Cursor End Register
    CursorEnd   = 0x0B,
    /// Start Address High Register
    StartHi     = 0x0C,
    /// Start Address Low Register
    StartLo     = 0x0D,
    /// Cursor Location High Register
    CursorHi    = 0x0E,
    /// Cursor Location Low Register
    CursorLo    = 0x0F,
    /// Vertical Retrace Start Register
    VSyncStart  = 0x10,
    /// Vertical Retrace End Register
    VSyncEnd    = 0x11,
    /// Vertical Display End Register
    VDispEnd    = 0x12,
    /// Offset Register
    Offset      = 0x13,
    /// Underline Location Register
    Underline   = 0x14,
    /// Start Vertical Blanking Register
    VBlankStart = 0x15,
    /// End Vertical Blanking
    VBlankEnd   = 0x16,
    /// CRTC Mode Control Register
    Mode        = 0x17,
    /// Line Compare Register
    LineCompare = 0x18,
}

/// Write a value to a CRT Controller register
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

/// Read a value from a CRT Controller register
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
