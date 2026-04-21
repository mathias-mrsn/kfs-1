//! TODO: Need to improve some calculations.
//! TODO: Need to clear mutex lock.
//! BUG: Panic when scrolling down cause vc_index is sometimes higher than
//! vc_end_screen.
//!
//! Ideas for implementation:
//! - Create split screen mode.
//! - Add helper functions for CRTC and GFX controllers to dump registers.
use core::fmt;
use core::ptr;
use core::{cmp, slice};

use super::{crtc, gfxc};

/// Default 16-bit word for clearing VGA text mode memory.
/// Represents a space character (0x20) with light gray foreground (0x07).
/// Format: [15:12]=background color, [11:8]=foreground color, [7:0]=ASCII
/// character
const BLANK: u16 = 0x0720;

/// Standard 16-color VGA color palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum VGAColor
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
pub(crate) enum CursorTypes
{
        Underline,
        LowerThird,
        LowerHalf,
        Full,
        None,
}

/// VGA memory mapping ranges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum MemoryRanges
{
        /// A0000h-BFFFFh (128K region)
        Large  = 0,
        /// A0000h-AFFFFh (64K region)
        Medium = 1,
        /// B8000h-BFFFFh (32K region)
        Small  = 3,
}

/// Standard VGA text mode resolutions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Resolution
{
        /// 40 columns × 10 rows text mode
        R40_10,
        /// 40 columns × 25 rows text mode
        R40_25,
        /// 40 columns × 50 rows text mode
        R40_50,
        /// 80 columns × 10 rows text mode
        R80_10,
        /// 80 columns × 25 rows text mode (most common)
        R80_25,
        /// 80 columns × 50 rows text mode
        R80_50,
        /// 120 columns × 25 rows text mode
        R120_25,
        /// 120 columns × 50 rows text mode
        R120_50,
}

/// Scrolling directions for VGA text mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScrollDir
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
///
/// vc_vram_base ------> +---------------+-.
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
#[derive(Debug, Clone, Copy)]
pub(crate) struct VgaConsole
{
        /// Base address of VGA memory
        vc_vram_base:        u32,
        /// End address of VGA memory
        vc_vram_end:         u32,
        /// Current position in VGA memory where next character will be written
        vc_index:            u32,
        /// Total size of VGA memory in bytes
        vc_vram_size:        u32,
        /// Size of visible screen area in bytes
        vc_screen_size:      u32,
        /// Current foreground color for text output
        vc_foreground_color: VGAColor,
        /// Current background color for text output
        vc_background_color: VGAColor,
        /// Address of the first visible character on screen
        vc_visible_origin:   u32,
        /// Starting address of the current text buffer
        vc_origin:           u32,
        /// Ending address of the current text buffer
        vc_origin_end:       u32,
        /// Number of rows in the display
        vc_rows:             u8,
        /// Number of columns in the display
        vc_cols:             u8,
        /// Current cursor appearance type
        vc_cursor_type:      CursorTypes,
}

impl VgaConsole
{
        /// Creates a new VGA text mode console with the specified
        /// configuration.
        pub(crate) fn new(
                foreground_color: VGAColor,
                background_color: VGAColor,
                resolution: Resolution,
                memory_range: MemoryRanges,
                cursor_type: Option<CursorTypes>,
        ) -> Self
        {
                let vram_base = match memory_range {
                        MemoryRanges::Large | MemoryRanges::Medium => 0xa0000,
                        MemoryRanges::Small => 0xb8000,
                };

                let vram_size = match memory_range {
                        MemoryRanges::Large => 0x20000,
                        MemoryRanges::Medium => 0xffff,
                        MemoryRanges::Small => 0x8000,
                };

                let (cols, rows): (u8, u8) = match resolution {
                        Resolution::R40_10 => (40, 10),
                        Resolution::R40_25 => (40, 25),
                        Resolution::R40_50 => (40, 50),
                        Resolution::R80_10 => (80, 10),
                        Resolution::R80_25 => (80, 25),
                        Resolution::R80_50 => (80, 50),
                        Resolution::R120_25 => (120, 25),
                        Resolution::R120_50 => (120, 50),
                };

                let misc: u8 = gfxc::read(gfxc::Register::Miscellaneous) & 0xf2;
                unsafe {
                        gfxc::write(
                                gfxc::Register::Miscellaneous,
                                misc | (memory_range as u8) << 2,
                        );
                }

                let screen_size: u32 =
                        rows as u32 * cols as u32 * core::mem::size_of::<u16>() as u32;

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
                con.resize(cols, rows);

                con
        }

        /// Updates the CRT Controller's Start Address registers to set the
        /// visible_origin
        #[inline(always)]
        fn set_mem_start(&mut self)
        {
                /*
                 * The value if divided by 2 cause the Start Offset in CRT Controller
                 * represent the offset between vram_base and the origin in word (2
                 * bytes).
                 */
                let start: u16 = ((self.vc_visible_origin - self.vc_vram_base as u32) / 2) as _;

                unsafe {
                        crtc::write(crtc::Register::StartAddressLow, start as u8);
                        crtc::write(crtc::Register::StartAddressHigh, (start >> 8) as u8)
                }
        }

        /// Computes the position of the beginning of the line where the index
        /// is located.
        #[inline(always)]
        fn start_of_line(
                &mut self,
                mut pos: u32,
        ) -> u32
        {
                pos -= self.vc_vram_base;
                pos - (pos % (self.vc_cols as u32 * 2)) + self.vc_vram_base
        }

        /// Writes a single character to the VGA text buffer using default
        /// colors
        #[inline(always)]
        fn putc(
                &mut self,
                c: u8,
        )
        {
                self.cputc(c, None, None);
        }

        /// Writes a single character to the VGA text buffer with optional
        /// custom colors
        fn cputc(
                &mut self,
                c: u8,
                foreground: Option<u8>,
                background: Option<u8>,
        )
        {
                let bg_color = background.unwrap_or(self.vc_background_color as u8) & 0xf;
                let fg_color = foreground.unwrap_or(self.vc_foreground_color as u8) & 0xf;
                let word = (c as u16) | ((bg_color as u16) << 12) | ((fg_color as u16) << 8);

                if self.vc_index == self.vc_origin_end {
                        self.scroll(ScrollDir::Down, Some(1));
                }

                unsafe {
                        *(self.vc_index as *mut u16) = word;
                }

                self.vc_index += core::mem::size_of::<u16>() as u32;
                self.cursor(None);
        }

        /// Writes a string to the VGA text buffer using default colors
        #[inline(always)]
        fn putstr(
                &mut self,
                str: &str,
        )
        {
                self.cputstr(str, None, None);
        }

        /// Writes a string to the VGA text buffer with optional custom colors
        fn cputstr(
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
        fn scroll(
                &mut self,
                dir: ScrollDir,
                lines: Option<u32>,
        )
        {
                if let Some(lines) = lines {
                        debug_assert!(lines > 0);
                        if lines > self.vc_rows as u32 / 2 {
                                return;
                        }
                }

                debug_assert!(self.vc_index <= self.vc_origin_end);

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
                                self.vc_visible_origin = cmp::min(
                                        self.vc_visible_origin.saturating_add(delta),
                                        self.vc_origin,
                                );
                        }
                        ScrollDir::Down if lines.is_some() => {
                                let oldi = self.vc_index;
                                if self.vc_origin_end - self.vc_index > delta {
                                        // If there is enough space to scroll down.
                                        self.vc_index = self.start_of_line(self.vc_index + delta)
                                } else {
                                        // Otherwise, move the cursor to the end of the screen.
                                        self.vc_index = self.start_of_line(self.vc_origin_end)
                                };

                                // Substract from delta the number of character already present on
                                // screen.
                                delta -= self.vc_index - self.start_of_line(oldi);

                                // Ensure that delta is a multiple of the screen size.
                                debug_assert!(delta % (self.vc_cols as u32 * 2) == 0);

                                // If the buffer doesnt have enough place to scroll. Copy the screen
                                // to the beginning of the buffer
                                // and empty it.
                                if self.vc_origin_end + delta > self.vc_vram_end {
                                        unsafe {
                                                ptr::copy(
                                                        self.vc_origin as *mut u8,
                                                        self.vc_vram_base as *mut u8,
                                                        (self.vc_screen_size.saturating_sub(delta))
                                                                as usize,
                                                );
                                        }
                                        self.vc_origin = self.vc_vram_base;
                                } else {
                                        // Otherwise just move the current
                                        self.vc_origin += delta;
                                }
                                // TEST: THIS
                                self.vc_index = self
                                        .vc_index
                                        .saturating_sub(oldo)
                                        .saturating_sub(delta)
                                        .saturating_add(self.vc_origin);
                                self.vc_origin_end = self.vc_origin + self.vc_screen_size;
                                self.vc_visible_origin = self.vc_origin;
                                unsafe {
                                        let s: &mut [u16] = slice::from_raw_parts_mut(
                                                (self.vc_origin as *mut u16).add(((self
                                                        .vc_screen_size
                                                        - delta)
                                                        / 2)
                                                        as usize),
                                                delta as usize,
                                        );
                                        s.fill(BLANK);
                                }
                        }
                        ScrollDir::Bottom => {
                                self.vc_visible_origin = self.vc_origin;
                        }
                        ScrollDir::Top => {
                                self.vc_visible_origin = self.vc_vram_base;
                        }
                        _ => {
                                panic!("VGA Error: Unknown scroll direction")
                        }
                }
                self.set_mem_start();
                self.cursor(Some(CursorTypes::Full));
        }

        /// Clears the entire VGA text buffer by filling it with blank
        /// characters
        fn blank(&mut self)
        {
                unsafe {
                        let s: &mut [u16] = slice::from_raw_parts_mut(
                                self.vc_vram_base as *mut u16,
                                self.vc_vram_size as usize,
                        );
                        s.fill(BLANK);
                }
                self.vc_origin = self.vc_vram_base;
                self.vc_visible_origin = self.vc_vram_base;
                self.vc_index = self.vc_vram_base;

                self.set_mem_start();
                self.cursor(None);
        }

        /// Sets the VGA text mode cursor size by configuring its start and end
        /// scan lines
        fn cursor_size(
                &mut self,
                from: u8,
                to: u8,
        )
        {
                let mut c_start: u8 = crtc::read(crtc::Register::CursorStart);
                let mut c_end: u8 = crtc::read(crtc::Register::CursorEnd);

                c_start = (c_start & 0xc0) | from;
                c_end = (c_end & 0xe0) | to;

                unsafe {
                        crtc::write(crtc::Register::CursorStart, c_start);
                        crtc::write(crtc::Register::CursorEnd, c_end);
                }
        }

        /// Updates the hardware cursor position and optionally changes its
        /// appearance
        fn cursor(
                &mut self,
                cursor_type: Option<CursorTypes>,
        )
        {
                let pos = (self.vc_index as u32 - self.vc_vram_base) / 2;
                unsafe {
                        crtc::write(crtc::Register::CursorLocationLow, pos as u8);
                        crtc::write(crtc::Register::CursorLocationHigh, (pos >> 8) as u8);
                }

                if cursor_type.is_some() {
                        const CURSOR_ENABLE_MASK: u8 = 0xdf;
                        const CURSOR_DISABLE_MASK: u8 = 0x20;
                        let c = crtc::read(crtc::Register::CursorStart);

                        if self.vc_cursor_type == CursorTypes::None {
                                unsafe {
                                        crtc::write(
                                                crtc::Register::CursorStart,
                                                c & CURSOR_ENABLE_MASK,
                                        );
                                }
                        }

                        match cursor_type.unwrap() {
                                CursorTypes::Full => self.cursor_size(0, 16),
                                CursorTypes::LowerHalf => self.cursor_size(8, 16),
                                CursorTypes::LowerThird => self.cursor_size(10, 16),
                                CursorTypes::Underline => self.cursor_size(15, 16),
                                CursorTypes::None => unsafe {
                                        crtc::write(
                                                crtc::Register::CursorStart,
                                                c | CURSOR_DISABLE_MASK,
                                        )
                                },
                        }
                }
        }

        /// Resizes the VGA text mode display to the specified dimensions
        fn resize(
                &mut self,
                width: u8,
                height: u8,
        )
        {
                let mut scanlines: u32 = height as u32 * 16;

                /* If Scan Doubling enabled, 200-scan-line video data is converted to
                 * 400-scan-line output */
                let max_scan: u8 = crtc::read(crtc::Register::MaximumScanLine);
                if (max_scan & 0x80) != 0 {
                        scanlines <<= 1;
                }

                /* If SLDIV enabled, divide scan line clock by 2 */
                let mode = crtc::read(crtc::Register::ModeControl);
                if (mode & 0x04) != 0 {
                        scanlines >>= 1;
                }
                scanlines -= 1;
                let scanlines_lo = scanlines & 0xff;

                /*
                 * Set the two higher bits of the Vertical Display End
                 */
                let mut r7 = crtc::read(crtc::Register::Overflow) & !0x42;
                if (scanlines & 0x100) != 0 {
                        r7 |= 0x02;
                }
                if (scanlines & 0x200) != 0 {
                        r7 |= 0x40;
                }

                /* Disable write protection */
                let vsync_end = crtc::read(crtc::Register::VerticalRetraceEnd);
                unsafe {
                        crtc::write(crtc::Register::VerticalRetraceEnd, vsync_end & !0x80);

                        /* Reduire the size of the window */
                        crtc::write(crtc::Register::HorizontalDisplayEnd, width - 1);
                        crtc::write(crtc::Register::VerticalDisplayEnd, scanlines_lo as u8);
                        crtc::write(crtc::Register::Offset, width >> 1);
                        crtc::write(crtc::Register::VerticalRetraceEnd, scanlines_lo as u8);
                        crtc::write(crtc::Register::Overflow, r7);

                        /* Restore write protection state */
                        crtc::write(crtc::Register::VerticalRetraceEnd, vsync_end);
                }

                self.vc_cols = width;
                self.vc_rows = height;
        }

        fn base_as_ptr(&self) -> *const () { self.vc_vram_base as *const () }

        fn size(&self) -> u32 { self.vc_vram_size }
}

/// Implements the [`core::fmt::Write`] trait for [`VgaConsole`], allowing it to
/// be used with Rust's formatting macros like `write!` and `writeln!`.
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
