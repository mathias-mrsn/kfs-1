/**
 * TODO: Need to improve some calculations
 * TODO: Test boundaries
 *
 * Ideas for implementation:
 * - Create split screen mode
 * - Add helper functions for CRTC and GFX controllers to dump registers
 *
 */
use crate::utils::writec;

use core::cmp;
use core::fmt;
use core::ptr;

/// This module provides low-level access to the VGA Graphics Controller
/// registers.
pub mod gfxc
{
    use crate::io::{inb, outb};

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
}

/// This module provides low-level access to the VGA CRT Controller
/// registers.
pub mod crtc
{
    use crate::io::{inb, outb};

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
}

/// Default 16-bit word for clearing VGA text mode memory.
/// Represents a space character (0x20) with light gray foreground (0x07).
/// Format: [15:12]=background color, [11:8]=foreground color, [7:0]=ASCII
/// character
const BLANK: u16 = 0x0720;

/// Standard 16-color VGA color palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VGAColor
{
    Black      = 0x00,
    Blue       = 0x01,
    Green      = 0x02,
    Cyan       = 0x03,
    Red        = 0x04,
    Magenta    = 0x05,
    Brown      = 0x06,
    LightGray  = 0x07,
    DarkGray   = 0x08,
    LightBlue  = 0x09,
    LightGreen = 0x0a,
    LightCyan  = 0x0b,
    LightRed   = 0x0c,
    Pink       = 0x0d,
    Yellow     = 0x0e,
    White      = 0x0f,
}

/// Types of text mode cursor shapes available in VGA.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CursorTypes
{
    Underline,
    LowerThird,
    LowerHalf,
    Full,
    None,
}

/// VGA memory mapping ranges.
///
/// Defines the different memory ranges that can be used for VGA memory mapping.
/// Each range corresponds to a different memory size and base address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MemoryRanges
{
    /// A0000h-BFFFFh (128K region)
    Large  = 0,
    /// A0000h-AFFFFh (64K region)
    Medium = 1,
    /// B8000h-BFFFFh (32K region)
    Small  = 3,
}

/// Scrolling directions for VGA text mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDir
{
    /// Moves the visible_origin up.
    VisualUp,
    /// Moves the visible_origin down.
    VisualDown,
    /// Moves the index down and if there isn't enough space,
    /// moves down both the origin and visible_origin.
    Down,
    /// Moves the visible_origin to the vram_base.
    Top,
    /// Moves the visible_origin to vram_end minus screen_size.
    Bottom,
}

/// VGA text mode console driver that provides basic text output functionality
///
/// This structure manages a VGA text mode console by maintaining the state of
/// the video memory and providing methods for text output and scrolling. It
/// supports standard VGA text mode operations including cursor management,
/// color control, and scrolling.
///
/// # Memory Layout
/// ```text
/// vc_screenbuf ------> +---------------+-.
///                      |               |  \
///                      |               |   |
///                      |               |    > area 1
///                      |               |   |
///                      |               |  /
///                      +---------------+-:
///                      |               |  \
/// vc_visible_origin > ^| $> ls         |   |
///                     || file          |    > area 2
///           vc_rows  < | file2         |   |
///                     || $> cat file   |  /
/// vc_origin --------> |+---------------+-:
///                     || Hello         |  \
///                     v| $> uname      |   |
///                      | Darwin        |    > area 3
/// vc_index ------------|--------v      |   |
///                      | $> echo       |  /
/// vc_origin_end -----> +---------------+-'
///                      |<-- vc_cols -->|
///                      .               .
///                      .               .
///                      +---------------- <-- vram_end
/// ```
#[derive(Debug)]
pub struct VgaConsole
{
    /// Base address of VGA memory
    pub vc_vram_base:        u32,
    /// End address of VGA memory
    pub vc_vram_end:         u32,
    /// Current position in VGA memory where next character will be written
    pub vc_index:            u32,
    /// Total size of VGA memory in bytes
    pub vc_vram_size:        u32,
    /// Size of visible screen area in bytes
    pub vc_screen_size:      u32,
    /// Current foreground color for text output
    pub vc_foreground_color: VGAColor,
    /// Current background color for text output
    pub vc_background_color: VGAColor,
    /// Address of the first visible character on screen
    pub vc_visible_origin:   u32,
    /// Starting address of the current text buffer
    pub vc_origin:           u32,
    /// Ending address of the current text buffer
    pub vc_origin_end:       u32,
    /// Number of rows in the display
    pub vc_rows:             u8,
    /// Number of columns in the display
    pub vc_cols:             u8,
    /// Current cursor appearance type
    pub vc_cursor_type:      CursorTypes,
}

impl VgaConsole
{
    /// Creates a new VGA text mode console with the specified configuration.
    ///
    /// This function initializes a new VGA console with the given parameters
    /// and sets up the video memory according to the specified memory
    /// range.
    ///
    /// # Arguments
    ///
    /// * `foreground_color` - The default text color for characters
    /// * `background_color` - The default background color for characters
    /// * `rows` - Number of text rows in the display (typically 25)
    /// * `cols` - Number of text columns in the display (typically 80)
    /// * `memory_range` - The VGA memory mapping range to use:
    ///   - `Large`: A0000h-BFFFFh (128K)
    ///   - `Medium`: A0000h-AFFFFh (64K)
    ///   - `Small`: B8000h-BFFFFh (32K)
    /// * `cursor_type` - Optional cursor appearance type. If `None`, cursor
    ///   remains hidden
    ///
    /// # Returns
    ///
    /// Returns a new `VgaConsole` instance configured with the specified
    /// parameters.
    ///
    /// # Example
    ///
    /// ```rust
    /// let vga = VgaConsole::new(
    ///     VGAColor::White,           // Foreground color
    ///     VGAColor::Black,           // Background color
    ///     25,                        // Rows
    ///     80,                        // Columns
    ///     MemoryRanges::Small,       // Memory range
    ///     Some(CursorTypes::Full),   // Cursor type
    /// );
    /// ```
    pub fn new(
        foreground_color: VGAColor,
        background_color: VGAColor,
        rows: u8,
        cols: u8,
        memory_range: MemoryRanges,
        cursor_type: Option<CursorTypes>,
    ) -> Self
    {
        let vram_base = match memory_range {
            MemoryRanges::Large | MemoryRanges::Medium => {
                gfxc::write(gfxc::Indexes::Misc, memory_range as u8);
                0xa0000
            }
            MemoryRanges::Small => 0xb8000,
        };

        let vram_size = match memory_range {
            MemoryRanges::Large => 0x20000,
            MemoryRanges::Medium => 0xf000,
            MemoryRanges::Small => 0x8000,
        };

        let screen_size: u32 = (rows as u32 * cols as u32) * core::mem::size_of::<u16>() as u32;

        let mut con = Self {
            vc_vram_base:        vram_base,
            vc_vram_end:         (vram_base + vram_size),
            vc_index:            vram_base,
            vc_vram_size:        vram_size,
            vc_screen_size:      screen_size,
            vc_foreground_color: foreground_color,
            vc_background_color: background_color,
            vc_visible_origin:   vram_base,
            vc_origin:           vram_base,
            vc_origin_end:       vram_base + screen_size,
            vc_rows:             rows,
            vc_cols:             cols,
            vc_cursor_type:      CursorTypes::None,
        };

        con.blank();
        con.cursor(cursor_type);
        con.resize(rows, cols);

        con
    }

    /// Updates the CRT Controller's Start Address registers to set the
    /// visible_origin
    ///
    /// This function calculates and sets the offset between the VGA memory base
    ///
    /// This is used internally for scrolling.
    #[inline(always)]
    fn set_mem_start(&mut self)
    {
        /*
         * The value if divided by 2 cause the Start Offset in CRT Controller
         * represent the offset between vram_base and the origin in word (2
         * bytes).
         */
        let start: u16 = ((self.vc_visible_origin - self.vc_vram_base as u32) / 2) as _;

        crtc::write(crtc::Indexes::StartLo, start as u8);
        crtc::write(crtc::Indexes::StartHi, (start >> 8) as u8)
    }

    /// Computes the position of the beginning of the line where the index is
    /// located.
    ///
    /// This is used internally for computing the index position when requesting
    /// a new line.
    #[inline(always)]
    fn start_of_line(
        &mut self,
        mut pos: u32,
    ) -> u32
    {
        pos -= self.vc_vram_base;
        pos - (pos % (self.vc_cols as u32 * 2)) + self.vc_vram_base
    }

    /// Writes a single character to the VGA text buffer using default colors
    ///
    /// This is a convenience wrapper around [`cputc`] that uses the console's
    /// current foreground and background colors.
    ///
    /// # Arguments
    ///
    /// * `c` - The ASCII character to write (values 0x20-0x7E are printable,
    ///   others will display as a special character)
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut vga = VgaConsole::new(/* ... */);
    /// vga.putc(b'A'); // Writes 'A' using current colors
    /// ```
    #[inline(always)]
    pub fn putc(
        &mut self,
        c: u8,
    )
    {
        self.cputc(c, None, None);
    }

    /// Writes a single character to the VGA text buffer with optional custom
    /// colors
    ///
    /// This function writes a character to the current cursor position in VGA
    /// text mode, allowing specification of custom foreground and
    /// background colors. If colors are not specified, it uses the
    /// console's current default colors.
    ///
    /// # Arguments
    ///
    /// * `c` - The ASCII character to write (values 0x20-0x7E are printable)
    /// * `foreground` - Optional custom foreground color (0-15). If None, uses
    ///   console's default
    /// * `background` - Optional custom background color (0-15). If None, uses
    ///   console's default
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut vga = VgaConsole::new(/* ... */);
    ///
    /// // Write 'A' with default colors
    /// vga.cputc(b'A', None, None);
    ///
    /// // Write 'B' with custom red foreground on black background
    /// vga.cputc(b'B', Some(0x04), Some(0x00));
    /// ```
    pub fn cputc(
        &mut self,
        c: u8,
        foreground: Option<u8>,
        background: Option<u8>,
    )
    {
        let bg_color = background.unwrap_or(self.vc_background_color as u8) & 0xf;
        let fg_color = foreground.unwrap_or(self.vc_foreground_color as u8) & 0xf;
        let word = (c as u16) | ((bg_color as u16) << 12) | ((fg_color as u16) << 8);

        unsafe {
            *(self.vc_index as *mut u16) = word;
        }
        self.vc_index += core::mem::size_of::<u16>() as u32;
        self.cursor(None);
    }

    /// Writes a string to the VGA text buffer using default colors
    ///
    /// This is a convenience wrapper around [`cputstr`] that uses the console's
    /// current foreground and background colors.
    ///
    /// # Arguments
    ///
    /// * `str` - The string to write to the VGA buffer. Non-printable ASCII
    ///   characters (except newline) will display as a special character
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut vga = VgaConsole::new(/* ... */);
    /// vga.putstr("Hello World!\n"); // Writes text using current colors
    /// ```
    #[inline(always)]
    pub fn putstr(
        &mut self,
        str: &str,
    )
    {
        self.cputstr(str, None, None);
    }

    /// Writes a string to the VGA text buffer with optional custom colors
    ///
    /// This function writes each character from the input string to the VGA
    /// buffer. It allows specifying custom foreground and background
    /// colors for the text.
    ///
    /// # Arguments
    ///
    /// * `str` - The string to write to the VGA buffer
    /// * `foreground` - Optional custom foreground color (0-15). If None, uses
    ///   console's default
    /// * `background` - Optional custom background color (0-15). If None, uses
    ///   console's default
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut vga = VgaConsole::new(/* ... */);
    ///
    /// // Write with default colors
    /// vga.cputstr("Hello\nWorld!", None, None);
    ///
    /// // Write with custom red foreground on black background
    /// vga.cputstr("Colored Text", Some(0x04), Some(0x00));
    /// ```
    pub fn cputstr(
        &mut self,
        str: &str,
        foreground: Option<u8>,
        background: Option<u8>,
    )
    {
        for byte in str.bytes() {
            match byte {
                // b'\n' => self.scroll(ScrollDir::NewLine, None),
                b'\n' => {
                    self.scroll(ScrollDir::Down, Some(1));
                }
                0x20..=0x7e => self.cputc(byte, foreground, background),
                _ => self.cputc(0xfe, None, None),
            };
        }
    }

    /// Scrolls the VGA text buffer in the specified direction
    ///
    /// # Arguments
    ///
    /// * `dir` - The scrolling direction:
    ///   - `ScrollDir::VisualUp` - Moves the visible window up
    ///   - `ScrollDir::VisualDown` - Moves the visible window down
    ///   - `ScrollDir::Down` - Moves content down, potentially wrapping around
    ///     buffer
    ///   - `ScrollDir::Top` - Jumps to the top of the buffer
    ///   - `ScrollDir::Bottom` - Jumps to the bottom of the buffer
    ///
    /// * `lines` - Optional number of lines to scroll. If None, defaults to 0.
    ///   For safety, cannot exceed half the screen height.
    ///
    /// # Safety
    ///
    /// - Performs unsafe operations when copying memory during buffer wrapping
    /// - Ensures scrolling stays within valid buffer boundaries
    /// - Maintains proper alignment of text lines
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut vga = VgaConsole::new(/* ... */);
    ///
    /// // Scroll content down by 1 line
    /// vga.scroll(ScrollDir::Down, Some(1));
    ///
    /// // Jump to top of buffer
    /// vga.scroll(ScrollDir::Top, None);
    /// ```
    pub fn scroll(
        &mut self,
        dir: ScrollDir,
        lines: Option<u32>,
    )
    {
        if lines.is_some() && lines.unwrap() > self.vc_rows as u32 / 2 {
            return;
        }

        let mut delta: u32 = lines.unwrap_or(0).saturating_mul((self.vc_cols * 2) as u32);
        let oldo: u32 = self.vc_origin;
        match dir {
            ScrollDir::VisualUp if lines.is_some() => {
                self.vc_visible_origin = cmp::max(
                    self.vc_visible_origin.saturating_sub(delta),
                    self.vc_vram_base,
                );
            }
            ScrollDir::VisualDown if lines.is_some() => {
                self.vc_visible_origin =
                    cmp::min(self.vc_visible_origin.saturating_add(delta), self.vc_origin);
            }
            ScrollDir::Down if lines.is_some() => {
                let oldi = self.vc_index;
                if self.vc_origin_end - self.vc_index > delta {
                    self.vc_index = self.start_of_line(self.vc_index + delta)
                } else {
                    self.vc_index = self.start_of_line(self.vc_origin_end - 1)
                };

                delta -= self.vc_index - self.start_of_line(oldi);

                if self.vc_origin_end + delta > self.vc_vram_end {
                    unsafe {
                        ptr::copy(
                            self.vc_origin as *mut u16,
                            self.vc_vram_base as *mut u16,
                            self.vc_screen_size as usize,
                        );
                        writec::<u16>(
                            (self.vc_vram_base as *mut u16).add(self.vc_screen_size as usize),
                            BLANK,
                            (self.vc_vram_size - self.vc_screen_size) as usize,
                        );
                    }
                    self.vc_origin = self.vc_vram_base;
                } else {
                    self.vc_origin += delta;
                }
                self.vc_index = (self.vc_index - oldo) + self.vc_origin;
                self.vc_origin_end = self.vc_origin + self.vc_screen_size;
                self.vc_visible_origin = self.vc_origin;
            }
            ScrollDir::Bottom => {
                self.vc_visible_origin = self.vc_origin;
            }
            ScrollDir::Top => {
                self.vc_visible_origin = self.vc_vram_base;
            }
            _ => {
                panic!("error")
            }
        }
        self.set_mem_start();
        self.cursor(Some(CursorTypes::Full));
    }

    /// Clears the entire VGA text buffer by filling it with blank characters
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut vga = VgaConsole::new(/* ... */);
    /// vga.blank(); // Clears the entire screen
    /// ```
    ///
    /// # Safety
    ///
    /// This function performs direct memory writes to VGA memory through unsafe
    /// operations.
    pub fn blank(&mut self)
    {
        unsafe {
            writec::<u16>(
                self.vc_vram_base as *mut u16,
                BLANK,
                self.vc_vram_size as usize,
            );
        }
        self.vc_origin = self.vc_vram_base;
        self.vc_visible_origin = self.vc_vram_base;
        self.vc_index = self.vc_vram_base;

        self.set_mem_start();
        self.cursor(None);
    }

    /// Sets the VGA text mode cursor size by configuring its start and end scan
    /// lines
    ///
    /// # Arguments
    ///
    /// * `from` - Starting scan line for the cursor (0-15)
    /// * `to` - Ending scan line for the cursor (0-15)
    ///
    /// # Safety
    ///
    /// This function performs direct hardware I/O through the CRTC registers
    /// and should only be called when appropriate hardware access is
    /// guaranteed.
    ///
    /// This is used internally custom the cursor appearance.
    fn cursor_size(
        &mut self,
        from: u8,
        to: u8,
    )
    {
        let mut c_start: u8 = crtc::read(crtc::Indexes::CursorStart);
        let mut c_end: u8 = crtc::read(crtc::Indexes::CursorEnd);

        c_start = (c_start & 0xc0) | from;
        c_end = (c_end & 0xe0) | to;

        crtc::write(crtc::Indexes::CursorStart, c_start);
        crtc::write(crtc::Indexes::CursorEnd, c_end);
    }

    /// Updates the hardware cursor position and optionally changes its
    /// appearance
    ///
    /// # Arguments
    ///
    /// * `cursor_type` - Optional new cursor appearance to set:
    ///   - `None` - Only updates cursor position without changing appearance
    ///   - `Some(CursorTypes::Full)` - Full block cursor (scan lines 0-16)
    ///   - `Some(CursorTypes::LowerHalf)` - Lower half block (scan lines 8-16)
    ///   - `Some(CursorTypes::LowerThird)` - Lower third block (scan lines
    ///     10-16)
    ///   - `Some(CursorTypes::Underline)` - Single line at bottom (scan line
    ///     15-16)
    ///   - `Some(CursorTypes::None)` - Hides the cursor
    ///
    /// # Safety
    ///
    /// This function performs direct hardware I/O through the CRTC registers
    /// and should only be called when appropriate hardware access is
    /// guaranteed.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut vga = VgaConsole::new(/* ... */);
    ///
    /// // Just update cursor position
    /// vga.cursor(None);
    ///
    /// // Change to underline cursor
    /// vga.cursor(Some(CursorTypes::Underline));
    ///
    /// // Hide cursor
    /// vga.cursor(Some(CursorTypes::None));
    /// ```
    pub fn cursor(
        &mut self,
        cursor_type: Option<CursorTypes>,
    )
    {
        let pos = (self.vc_index as u32 - self.vc_vram_base) / 2;
        crtc::write(crtc::Indexes::CursorLo, pos as u8);
        crtc::write(crtc::Indexes::CursorHi, (pos >> 8) as u8);

        if cursor_type.is_some() {
            const CURSOR_ENABLE_MASK: u8 = 0xdf;
            const CURSOR_DISABLE_MASK: u8 = 0x20;
            let c = crtc::read(crtc::Indexes::CursorStart);

            if self.vc_cursor_type == CursorTypes::None {
                crtc::write(crtc::Indexes::CursorStart, c & CURSOR_ENABLE_MASK);
            }

            match cursor_type.unwrap() {
                CursorTypes::Full => self.cursor_size(0, 16),
                CursorTypes::LowerHalf => self.cursor_size(8, 16),
                CursorTypes::LowerThird => self.cursor_size(10, 16),
                CursorTypes::Underline => self.cursor_size(15, 16),
                CursorTypes::None => {
                    crtc::write(crtc::Indexes::CursorStart, c | CURSOR_DISABLE_MASK)
                }
            }
        }
    }

    /// Resizes the VGA text mode display to the specified dimensions
    ///
    /// This function adjusts the VGA display size by configuring various CRT
    /// Controller registers to achieve the desired text mode dimensions. It
    /// handles scan line calculations and maintains proper synchronization.
    ///
    /// # Arguments
    ///
    /// * `height` - The desired height in character rows (typically 25 or 50)
    /// * `width` - The desired width in character columns (typically 80 or 40)
    ///
    /// # Safety
    ///
    /// This function performs direct hardware I/O through the CRTC registers
    /// and should only be called when appropriate hardware access is
    /// guaranteed.
    ///
    /// This function is a Rust implementation of the original vgacon_doresize
    /// function from the Linux kernel.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut vga = VgaConsole::new(/* ... */);
    ///
    /// // Resize to 25x80 text mode
    /// vga.resize(25, 80);
    ///
    /// // Resize to 50x40 text mode
    /// vga.resize(50, 40);
    /// ```
    pub fn resize(
        &mut self,
        height: u8,
        width: u8,
    )
    {
        let mut scanlines: u32 = height as u32 * 16;

        /* If Scan Doubling enabled, 200-scan-line video data is converted to
         * 400-scan-line output */
        let max_scan: u8 = crtc::read(crtc::Indexes::MaxScan);
        if (max_scan & 0x80) != 0 {
            scanlines <<= 1;
        }

        /* If SLDIV enabled, divide scan line clock by 2 */
        let mode = crtc::read(crtc::Indexes::Mode);
        if (mode & 0x04) != 0 {
            scanlines >>= 1;
        }
        scanlines -= 1;
        let scanlines_lo = scanlines & 0xff;

        /*
         * Set the two higher bits of the Vertical Display End
         */
        let mut r7 = crtc::read(crtc::Indexes::Overflow) & !0x42;
        if (scanlines & 0x100) != 0 {
            r7 |= 0x02;
        }
        if (scanlines & 0x200) != 0 {
            r7 |= 0x40;
        }

        /* Enable protection */
        let vsync_end = crtc::read(crtc::Indexes::VSyncEnd);
        crtc::write(crtc::Indexes::VSyncEnd, vsync_end & !0x80);

        /* Reduire the size of the window */
        crtc::write(crtc::Indexes::HDisp, width - 1);
        crtc::write(crtc::Indexes::VDispEnd, scanlines_lo as u8);
        crtc::write(crtc::Indexes::Offset, width >> 1);
        crtc::write(crtc::Indexes::VSyncEnd, scanlines_lo as u8);
        crtc::write(crtc::Indexes::Overflow, r7);

        /* Disable protection */
        crtc::write(crtc::Indexes::VSyncEnd, vsync_end);

        self.vc_cols = width;
        self.vc_rows = height;
    }
}

/// Implements the [`core::fmt::Write`] trait for [`VgaConsole`], allowing it to
/// be used with Rust's formatting macros like `write!` and `writeln!`.
///
/// This implementation enables formatted text output to the VGA console using
/// Rust's standard formatting infrastructure.
///
/// # Examples
///
/// ```rust
/// use core::fmt::Write;
/// let mut vga = VgaConsole::new(/* ... */);
///
/// // Using write! macro
/// write!(vga, "Hello {}", "World").unwrap();
///
/// // Using writeln! macro
/// writeln!(vga, "Value: {}", 42).unwrap();
/// ```
impl fmt::Write for VgaConsole
{
    fn write_str(
        &mut self,
        s: &str,
    ) -> fmt::Result
    {
        self.putstr(s);
        Ok(())
    }

    fn write_char(
        &mut self,
        c: char,
    ) -> fmt::Result
    {
        self.putc(c as u8);
        Ok(())
    }
}

#[test_case]
fn test_putstr()
{
    let mut vga: VgaConsole = VgaConsole::new(
        VGAColor::White,
        VGAColor::Black,
        25,
        80,
        MemoryRanges::Small,
        Some(CursorTypes::Full),
    );
    vga.putstr("\nHello\n");
    for _i in 0..25 {
        vga.putstr("0123456789abcdefghijklmnopqrstuvwxyz");
    }
    for _i in 0..15 {
        vga.putstr("newline\n");
    }
    vga.putstr("end");
    unsafe {
        assert_eq!(*(vga.vc_origin as *mut u16).offset(0), 0x0f00 | 'g' as u16);
    }
    vga.scroll(ScrollDir::Top, None);
    vga.scroll(ScrollDir::Bottom, None);
    vga.putstr("\nh\nhh\n");

    unsafe {
        assert_eq!(*(vga.vc_origin as *mut u16).offset(0), 0x0f00 | '4' as u16);
    }
}

#[test_case]
fn test_scroll()
{
    let mut vga: VgaConsole = VgaConsole::new(
        VGAColor::White,
        VGAColor::Black,
        25,
        80,
        MemoryRanges::Small,
        Some(CursorTypes::Full),
    );

    for _i in 0..1000 {
        vga.scroll(ScrollDir::Down, Some(10));
        vga.putstr("Hello");
    }

    unsafe {
        assert_eq!(0, 1);
    }
}
