//! VGA text mode console driver that provides basic text output functionality
//!
//! This structure manages a VGA text mode console by maintaining the state of
//! the video memory and providing methods for text output and scrolling. It
//! supports standard VGA text mode operations including cursor management,
//! color control, and scrolling.
//!
//! # Memory Layout
//!
//! phys_base ------> +---------------+-.
//!                      |               |  \
//!                      |               |   |
//!                      |               |    > area 1
//!                      |               |   |
//!                      |               |  /
//!                      +---------------+-:
//!                      |               |  \
//! user_base -------> | status line    |   |
//! visible_origin > ^| $> ls         |   |
//!                     || file          |    > area 2
//!           rows     < | file2         |   |
//!                     || $> cat file   |  /
//! origin -----------> |+---------------+-:
//!                     || Hello         |  \
//!                     v| $> uname      |   |
//!                      | Darwin        |    > area 3
//! index -------------->|--------v      |   |
//!                      | $> echo       |  /
//! origin_end --------> +---------------+-'
//!                      |<--- cols ----->|
//!                      .               .
//!                      .               .
//!                      +---------------- <-- vram_end
//!
use core::fmt;
use core::ptr;
use core::slice;

use super::{crtc, gfxc};

/// Default 16-bit word for clearing VGA text mode memory.
const BLANK: u16 = 0x0720;
const VGA_CELL_SIZE: usize = core::mem::size_of::<u16>();
/// Bit masks for enabling and disabling the VGA text mode cursor by using the
/// CRTC CursorStart Register.
const CURSOR_ENABLE_MASK: u8 = 0xdf;
const CURSOR_DISABLE_MASK: u8 = 0x20;

const MAX_SCANLINE_MASK: u8 = 0x1f;

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
pub(crate) enum CursorType
{
    Underline,
    LowerThird,
    LowerHalf,
    Full,
    None,
}

impl CursorType
{
    fn sizes(
        self,
        max_scanline: u8,
    ) -> (u8, u8)
    {
        match self {
            CursorType::Full => (0, max_scanline),
            CursorType::LowerHalf => (max_scanline.div_ceil(2), max_scanline),
            CursorType::LowerThird => ((((max_scanline + 1) * 2) / 3), max_scanline),
            CursorType::Underline => (max_scanline, max_scanline),
            CursorType::None => (0, 0),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Mode
{
    Terminal,
    Visual,
}

impl Mode
{
    fn as_status_text(self) -> &'static str
    {
        match self {
            Mode::Terminal => "-- TERMINAL --",
            Mode::Visual => "-- VISUAL --",
        }
    }
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

impl MemoryRanges
{
    /// Returns the base address and size of the VGA memory range corresponding
    fn layout(self) -> (usize, usize)
    {
        match self {
            Self::Large => (0xa0000, 0x20000),
            Self::Medium => (0xa0000, 0x10000),
            Self::Small => (0xb8000, 0x8000),
        }
    }

    /// Returns the bits to set in the Miscellaneous Output Register to select
    fn gfxc_memory_map_bits(self) -> u8
    {
        match self {
            Self::Large => 0,
            Self::Medium => 1,
            Self::Small => 3,
        }
    }
}

/// Standard VGA text mode resolutions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Resolution
{
    R40_10,
    R40_25,
    R40_50,
    R80_10,
    R80_25,
    R80_50,
    R120_25,
    R120_50,
}

impl Resolution
{
    /// Returns the number of columns and rows for the given resolution.
    fn dimensions(self) -> (usize, usize)
    {
        match self {
            Resolution::R40_10 => (40, 10),
            Resolution::R40_25 => (40, 25),
            Resolution::R40_50 => (40, 50),
            Resolution::R80_10 => (80, 10),
            Resolution::R80_25 => (80, 25),
            Resolution::R80_50 => (80, 50),
            Resolution::R120_25 => (120, 25),
            Resolution::R120_50 => (120, 50),
        }
    }
}

/// Scrolling directions for VGA text mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VisualAction
{
    ViewLinesUp(usize),
    ViewLinesDown(usize),
    ViewPagesUp(usize),
    ViewPagesDown(usize),
    ToTop,
    ToBottom,
    FollowOutput,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct VgaConsole
{
    /// Physical base address of the selected VGA memory window.
    phys_base:              usize,
    /// Base address of the user-visible scrollable text region.
    user_base:              usize,
    /// End address of the scrollable VGA memory region.
    vram_end:               usize,
    /// Current position in VGA memory where next character will be written
    index:                  usize,
    /// Total size of the scrollable VGA memory region in bytes.
    vram_size:              usize,
    /// Size of visible screen area in bytes
    screen_size:            usize,
    /// Current foreground color for text output
    foreground_color:       VGAColor,
    /// Current background color for text output
    background_color:       VGAColor,
    /// Foreground color used by the fixed status line.
    status_line_text_color: VGAColor,
    /// Background color used by the fixed status line.
    status_line_bg_color:   VGAColor,
    /// Address of the first visible character on screen
    visible_origin:         usize,
    /// Starting address of the current text buffer
    origin:                 usize,
    /// Ending address of the current text buffer
    origin_end:             usize,
    /// Number of rows in the display
    rows:                   usize,
    /// Number of columns in the display
    cols:                   usize,
    /// Current cursor appearance type
    cursor_type:            CursorType,
    /// Whether the visible window is detached from the active output position.
    visual_mode:            Mode,
    /// Whether the split-screen status line is currently enabled.
    status_line_enabled:    bool,
}

impl VgaConsole
{
    /// Creates a new VGA text mode console with the specified
    /// configuration.
    pub(crate) fn new(
        foreground_color: VGAColor,
        background_color: VGAColor,
        status_line_text_color: VGAColor,
        status_line_bg_color: VGAColor,
        resolution: Resolution,
        memory_range: MemoryRanges,
        cursor_type: CursorType,
    ) -> Self
    {
        let (phys_base, mut vram_size) = memory_range.layout();
        let (cols, rows) = resolution.dimensions();
        let line_stride = cols * core::mem::size_of::<u16>();

        let user_base = phys_base + line_stride;
        vram_size -= line_stride;

        let screen_size = cols * rows * core::mem::size_of::<u16>();
        debug_assert!(screen_size <= vram_size);

        let misc = gfxc::read(gfxc::Register::Miscellaneous) & !0x0c;
        // SAFETY: The selected memory map value is valid for the GFXC Miscellaneous
        // register.
        unsafe {
            gfxc::write(
                gfxc::Register::Miscellaneous,
                misc | (memory_range.gfxc_memory_map_bits() << 2),
            );
        }

        let mut con = Self {
            phys_base,
            user_base,
            vram_end: user_base + vram_size,
            index: user_base,
            vram_size,
            screen_size,
            foreground_color,
            background_color,
            status_line_text_color,
            status_line_bg_color,
            visible_origin: user_base,
            origin: user_base,
            origin_end: user_base + screen_size,
            rows,
            cols,
            cursor_type: CursorType::None,
            visual_mode: Mode::Visual,
            status_line_enabled: false,
        };
        con.resize(cols, rows);
        con.blank();
        con.enable_status_line();
        con.update_status_line();
        con.set_cursor_type(cursor_type);
        con.set_mem_start();
        con
    }

    #[inline(always)]
    fn line_stride(&self) -> usize { self.cols * core::mem::size_of::<u16>() }

    #[inline(always)]
    fn user_visible_rows(&self) -> usize
    {
        self.rows
            .saturating_sub(usize::from(self.status_line_enabled))
    }

    #[inline(always)]
    fn page_stride(&self) -> usize
    {
        self.rows
            .checked_mul(self.line_stride())
            .expect("VGA Error: page stride overflow")
    }

    /// Returns the effective number of scanlines per text row.
    ///
    /// This starts from the character cell height encoded in the CRTC Maximum
    /// Scan Line register, then applies the scan-doubling and SLDIV timing
    /// modifiers.
    fn scanlines_per_row(&self) -> usize
    {
        let max_scan = crtc::read(crtc::Register::MaximumScanLine);
        let mode = crtc::read(crtc::Register::ModeControl);

        let mut scanlines = usize::from(
            (max_scan & MAX_SCANLINE_MASK)
                .checked_add(1)
                .expect("VGA Error: scanline count overflow"),
        );

        if (max_scan & 0x80) != 0 {
            scanlines <<= 1;
        }

        if (mode & 0x04) != 0 {
            scanlines >>= 1;
        }

        scanlines
    }

    /// Updates the CRTC start address from `visible_origin`.
    ///
    /// The CRTC start offset is expressed in words relative to `phys_base`.
    ///
    /// # Panics
    /// Panics if `visible_origin` is below `user_base` or out of bounds.
    fn set_mem_start(&self)
    {
        let visible_offset_bytes = self
            .visible_origin
            .checked_sub(self.phys_base)
            .expect("VGA invariant violated: visible_origin is below phys_base");
        if !visible_offset_bytes.is_multiple_of(core::mem::size_of::<u16>()) {
            panic!("VGA invariant violated: visible_origin is not word-aligned");
        }

        let visible_offset_words = visible_offset_bytes / core::mem::size_of::<u16>();
        let max_start_words = self
            .vram_size
            .checked_sub(self.screen_size)
            .expect("VGA invariant violated: screen size is larger than VRAM")
            / core::mem::size_of::<u16>();
        if visible_offset_words > max_start_words {
            panic!("VGA Error: visible_origin is out of bounds");
        }

        let start = u16::try_from(visible_offset_words)
            .expect("VGA invariant violated: CRTC start offset does not fit in u16");

        // SAFETY: The caller maintains valid VGA state, and CRTC accesses are
        // serialized.
        unsafe {
            crtc::write(crtc::Register::StartAddressLow, start as u8);
            crtc::write(crtc::Register::StartAddressHigh, (start >> 8) as u8);
        }
    }

    /// Computes the position of the beginning of the line where the index
    /// is located.
    #[inline(always)]
    fn start_of_line(
        &mut self,
        mut pos: usize,
    ) -> usize
    {
        pos -= self.user_base;
        pos - (pos % self.line_stride()) + self.user_base
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
        let bg_color = background.unwrap_or(self.background_color as u8) & 0xf;
        let fg_color = foreground.unwrap_or(self.foreground_color as u8) & 0xf;
        let word = (c as u16) | ((bg_color as u16) << 12) | ((fg_color as u16) << 8);

        if self.index == self.origin_end {
            // self.visual(VisualAction::AdvanceLines(1));
            self.new_line();
        }

        unsafe {
            *(self.index as *mut u16) = word;
        }

        self.index += core::mem::size_of::<u16>();
        self.update_cursor_position();
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
                    self.new_line();
                }
                0x20..=0x7e => self.cputc(byte, foreground, background),
                _ => self.cputc(0xfe, None, None),
            };
        }
    }

    /// Advances the cursor to the beginning of the next line.
    ///
    /// If the cursor is already on the last visible row, the visible window is
    /// scrolled down by one line. The newly exposed last line is cleared.
    fn new_line(&mut self)
    {
        let line_stride = self.line_stride();
        let current_line_start = self.start_of_line(self.index);
        let next_line_start = current_line_start
            .checked_add(line_stride)
            .expect("VGA invariant violated: next line start overflowed");

        if next_line_start < self.origin_end {
            self.index = next_line_start;
            self.update_cursor_position();
            return;
        }

        if self
            .origin_end
            .checked_add(line_stride)
            .is_some_and(|end| end <= self.vram_end)
        {
            self.origin += line_stride;
            self.visible_origin = self.origin;
            self.origin_end += line_stride;
            self.index = next_line_start;

            // SAFETY: `index` points to the beginning of the newly exposed last
            // line, and `line_stride / size_of::<u16>()` is the number of text
            // cells in that line.
            unsafe {
                let line = slice::from_raw_parts_mut(
                    self.index as *mut u16,
                    line_stride / core::mem::size_of::<u16>(),
                );
                line.fill(BLANK);
            }
            self.set_mem_start();
            self.update_cursor_position();
            return;
        }

        let bytes_to_keep = self
            .screen_size
            .checked_sub(line_stride)
            .expect("VGA invariant violated: screen is smaller than one line");

        // SAFETY: Source and destination are valid VGA memory ranges, and `ptr::copy`
        // correctly handles overlap.
        unsafe {
            ptr::copy(
                (self.origin + line_stride) as *const u8,
                self.user_base as *mut u8,
                bytes_to_keep,
            );
            let last_line = slice::from_raw_parts_mut(
                (self.user_base + bytes_to_keep) as *mut u16,
                line_stride / core::mem::size_of::<u16>(),
            );
            last_line.fill(BLANK);
        }

        self.origin = self.user_base;
        self.visible_origin = self.user_base;
        self.origin_end = self.user_base + self.screen_size;
        self.index = self.origin_end - line_stride;
        self.set_mem_start();
        self.update_cursor_position();
    }

    /// Updates the visible VGA window according to a visual scrolling action.
    ///
    /// This only changes `visible_origin`. It does not modify the logical
    /// write position or the backing buffer contents.
    fn scroll_view(
        &mut self,
        action: VisualAction,
    )
    {
        let line_stride = self.line_stride();
        let page_stride = self.page_stride();
        let min_visible_origin = self.user_base;
        let max_visible_origin = self.origin;

        self.visible_origin = match action {
            VisualAction::ViewLinesUp(lines) => {
                self.visual_mode = Mode::Visual;
                let delta = lines
                    .checked_mul(line_stride)
                    .expect("VGA Error: scroll delta overflow");
                self.visible_origin
                    .saturating_sub(delta)
                    .max(min_visible_origin)
            }
            VisualAction::ViewLinesDown(lines) => {
                self.visual_mode = Mode::Visual;
                let delta = lines
                    .checked_mul(line_stride)
                    .expect("VGA Error: scroll delta overflow");
                self.visible_origin
                    .saturating_add(delta)
                    .min(max_visible_origin)
            }
            VisualAction::ViewPagesUp(pages) => {
                self.visual_mode = Mode::Visual;
                let delta = pages
                    .checked_mul(page_stride)
                    .expect("VGA Error: scroll delta overflow");
                self.visible_origin
                    .saturating_sub(delta)
                    .max(min_visible_origin)
            }
            VisualAction::ViewPagesDown(pages) => {
                self.visual_mode = Mode::Visual;
                let delta = pages
                    .checked_mul(page_stride)
                    .expect("VGA Error: scroll delta overflow");
                self.visible_origin
                    .saturating_add(delta)
                    .min(max_visible_origin)
            }
            VisualAction::ToTop => {
                self.visual_mode = Mode::Visual;
                min_visible_origin
            }
            VisualAction::ToBottom => {
                self.visual_mode = Mode::Visual;
                max_visible_origin
            }
            VisualAction::FollowOutput => {
                self.visual_mode = Mode::Terminal;
                let current_line_start = self.start_of_line(self.index);
                let visible_window_without_last_line = self
                    .screen_size
                    .checked_sub(line_stride)
                    .expect("VGA invariant violated: screen size is smaller than one line");
                current_line_start
                    .saturating_sub(visible_window_without_last_line)
                    .clamp(min_visible_origin, max_visible_origin)
            }
        };
        debug_assert!(
            self.visible_origin >= min_visible_origin && self.visible_origin <= max_visible_origin,
            "VGA invariant violated: visible_origin is out of bounds"
        );
        debug_assert!(
            self.visible_origin
                .checked_sub(self.user_base)
                .is_some_and(|offset| offset.is_multiple_of(line_stride)),
            "VGA invariant violated: visible_origin is not line-aligned"
        );
        self.update_status_line();
        self.set_mem_start();
    }

    /// Clears the VGA text buffer and resets the console state to the beginning
    /// of the selected VGA memory window.
    fn blank(&mut self)
    {
        let vram_words = self.vram_size / core::mem::size_of::<u16>();

        // SAFETY: `user_base` points to the selected VGA text buffer and
        // `vram_words` is the number of 16-bit cells contained in that buffer.
        unsafe {
            let buffer = slice::from_raw_parts_mut(self.user_base as *mut u16, vram_words);
            buffer.fill(BLANK);
        }

        self.origin = self.user_base;
        self.visible_origin = self.user_base;
        self.index = self.user_base;
        self.origin_end = self.origin + self.screen_size;
        self.visual_mode = Mode::Terminal;

        self.update_status_line();
        self.set_mem_start();
        self.update_cursor_position();
    }

    /// Programs the VGA text-mode cursor scanline range.
    fn cursor_size(
        &self,
        start_scanline: u8,
        end_scanline: u8,
    )
    {
        let max_scanline = crtc::read(crtc::Register::MaximumScanLine) & MAX_SCANLINE_MASK;
        assert!(
            start_scanline <= max_scanline,
            "VGA Error: cursor start scanline is out of bounds"
        );
        assert!(
            end_scanline <= max_scanline,
            "VGA Error: cursor end scanline is out of bounds"
        );
        assert!(
            start_scanline <= end_scanline,
            "VGA Error: cursor start scanline must be less than or equal to end scanline"
        );

        let mut cursor_start = crtc::read(crtc::Register::CursorStart);
        let mut cursor_end = crtc::read(crtc::Register::CursorEnd);
        cursor_start = (cursor_start & 0xc0) | start_scanline;
        cursor_end = (cursor_end & 0xe0) | end_scanline;

        // SAFETY: The scanline range has been validated against the current
        // character cell height, and CRTC accesses are serialized by the console.
        unsafe {
            crtc::write(crtc::Register::CursorStart, cursor_start);
            crtc::write(crtc::Register::CursorEnd, cursor_end);
        }
    }

    /// Programs the CRTC cursor position from the current console index.
    ///
    /// # Panics
    /// Panics if `index` is below `user_base` or if the computed cursor
    /// position does not fit in the CRTC cursor registers.
    fn update_cursor_position(&self)
    {
        let cursor_offset_bytes = self
            .index
            .checked_sub(self.phys_base)
            .expect("VGA invariant violated: cursor index is below phys_base");

        assert_eq!(cursor_offset_bytes % core::mem::size_of::<u16>(), 0);

        let cursor_offset_words = u16::try_from(cursor_offset_bytes / core::mem::size_of::<u16>())
            .expect("VGA invariant violated: cursor location does not fit in CRTC register");
        let cursor_lo = (cursor_offset_words & 0x00ff) as u8;
        let cursor_hi = ((cursor_offset_words >> 8) & 0x00ff) as u8;

        // SAFETY: The cursor location is a valid 16-bit CRTC word offset, and CRTC
        // accesses are serialized by the console.
        unsafe {
            crtc::write(crtc::Register::CursorLocationLow, cursor_lo);
            crtc::write(crtc::Register::CursorLocationHigh, cursor_hi);
        }
    }

    /// Applies the requested VGA hardware cursor type.
    ///
    /// `CursorType::None` disables the cursor. All other variants enable it and
    /// program the cursor scanline range within the current character cell.
    fn set_cursor_type(
        &mut self,
        cursor_type: CursorType,
    )
    {
        if self.cursor_type == cursor_type {
            return;
        }
        match cursor_type {
            CursorType::None => {
                self.disable_cursor();
            }
            _ => {
                if self.cursor_type == CursorType::None {
                    self.enable_cursor();
                }
                let max_scanline = crtc::read(crtc::Register::MaximumScanLine) & MAX_SCANLINE_MASK;
                let (start, end) = cursor_type.sizes(max_scanline);
                self.cursor_size(start, end);
            }
        }
        self.cursor_type = cursor_type;
    }

    /// Applies a bitmask to the CRTC CursorStart register to enable the VGA
    /// text mode cursor
    #[inline(always)]
    fn enable_cursor(&mut self)
    {
        let c = crtc::read(crtc::Register::CursorStart);

        // SAFETY: Written value is guaranteed to be a valid CRTC CursorStart value
        unsafe {
            crtc::write(crtc::Register::CursorStart, c & CURSOR_ENABLE_MASK);
        }
    }

    /// Applies a bitmask to the CRTC CursorStart register to disable the VGA
    /// text mode cursor
    #[inline(always)]
    fn disable_cursor(&mut self)
    {
        let c = crtc::read(crtc::Register::CursorStart);

        // SAFETY: Written value is guaranteed to be a valid CRTC CursorStart value
        unsafe {
            crtc::write(crtc::Register::CursorStart, c | CURSOR_DISABLE_MASK);
        }
    }

    /// Resizes the visible VGA text area.
    ///
    /// This reprograms the CRTC display width and height registers from the
    /// requested text-mode dimensions.
    ///
    /// Reference: https://github.com/torvalds/linux/blob/dd6c438c3e64a5ff0b5d7e78f7f9be547803ef1b/drivers/video/console/vgacon.c#L556
    fn resize(
        &mut self,
        width: usize,
        height: usize,
    )
    {
        assert!(width > 5, "VGA Error: width must be greater than 5");
        assert!(height > 5, "VGA Error: height must be greater than 5");

        let total_vram_size = self
            .vram_end
            .checked_sub(self.phys_base)
            .expect("VGA invariant violated: vram_end is below phys_base");
        let line_stride = width
            .checked_mul(core::mem::size_of::<u16>())
            .expect("VGA Error: line stride overflow");
        let user_base = self
            .phys_base
            .checked_add(line_stride)
            .expect("VGA Error: user base overflow");
        let vram_size = total_vram_size
            .checked_sub(line_stride)
            .expect("VGA Error: line stride is larger than VRAM size");
        let screen_size = width
            .checked_mul(height)
            .and_then(|cells| cells.checked_mul(core::mem::size_of::<u16>()))
            .expect("VGA Error: screen size overflow");

        assert!(
            screen_size <= vram_size,
            "VGA Error: screen size exceeds VRAM size"
        );

        let mut vertical_display_end = height
            .checked_mul(self.scanlines_per_row())
            .expect("VGA Error: vertical display end overflow");

        // The CRTC expects the index of the last visible scanline.
        vertical_display_end -= 1;

        // Vertical Display End stores only the low 8 bits. Bits 8 and 9 are stored
        // in the Overflow register.
        let vertical_display_end_lo = (vertical_display_end & 0xff) as u8;
        let mut overflow = crtc::read(crtc::Register::Overflow) & !0x42;
        if (vertical_display_end & 0x100) != 0 {
            overflow |= 0x02;
        }
        if (vertical_display_end & 0x200) != 0 {
            overflow |= 0x40;
        }

        let horizontal_display_end =
            u8::try_from(width - 1).expect("VGA Error: width must be between 1 and 256");

        // In VGA text mode, Offset is programmed in CRTC address units.
        let offset = u8::try_from(width >> 1).expect("VGA Error: width must be between 1 and 256");
        let vertical_retrace_end = crtc::read(crtc::Register::VerticalRetraceEnd);

        // SAFETY: CRTC accesses are serialized by the console, and the values
        // written here are constrained to the register widths expected by the VGA.
        unsafe {
            // Disable protection for CRTC registers 00h..07h while updating timing.
            crtc::write(
                crtc::Register::VerticalRetraceEnd,
                vertical_retrace_end & !0x80,
            );
            crtc::write(crtc::Register::HorizontalDisplayEnd, horizontal_display_end);
            crtc::write(crtc::Register::VerticalDisplayEnd, vertical_display_end_lo);
            crtc::write(crtc::Register::Offset, offset);
            crtc::write(crtc::Register::Overflow, overflow);
            // Restore the original protection state.
            crtc::write(crtc::Register::VerticalRetraceEnd, vertical_retrace_end);
        }

        let status_line_enabled = self.status_line_enabled;

        self.cols = width;
        self.rows = height;
        self.user_base = user_base;
        self.vram_size = vram_size;
        self.screen_size = screen_size;

        self.origin = user_base;
        self.visible_origin = user_base;
        self.index = user_base;
        self.origin_end = user_base + screen_size;
        self.visual_mode = Mode::Terminal;

        if status_line_enabled {
            self.enable_status_line();
        } else {
            self.disable_status_line();
        }

        self.blank();
    }

    fn base_as_ptr(&self) -> *const () { self.user_base as *const () }

    fn size(&self) -> usize { self.vram_size }
}

impl VgaConsole
{
    /// Write Line Compare register and related CRTC registers to enable
    /// split-screen status line mode.
    ///
    /// This configures the VGA hardware to split the display into two piece,
    /// the first line of the VRAM is reserved to the status line while the
    /// rest of the memory is used for the main text area.
    fn enable_status_line(&mut self)
    {
        let scanlines_per_row = self.scanlines_per_row();

        let line_compare = u16::try_from(
            self.rows
                .checked_sub(1)
                .and_then(|lines| lines.checked_mul(scanlines_per_row))
                .expect("VGA Error: line compare overflow"),
        )
        .expect("VGA Error: line compare does not fit in u16");

        let mut overflow = crtc::read(crtc::Register::Overflow);
        let mut max_scan = crtc::read(crtc::Register::MaximumScanLine);

        overflow = (overflow & !0x10)
            | if (line_compare & 0x0100) != 0 {
                0x10
            } else {
                0x00
            };
        max_scan = (max_scan & !0x40)
            | if (line_compare & 0x0200) != 0 {
                0x40
            } else {
                0x00
            };

        let vertical_retrace_end = crtc::read(crtc::Register::VerticalRetraceEnd);

        // SAFETY: The computed line-compare value is derived from the current text
        // geometry and written into the documented CRTC split-screen fields
        // while register access is serialized.
        unsafe {
            crtc::write(
                crtc::Register::VerticalRetraceEnd,
                vertical_retrace_end & !0x80,
            );
            crtc::write(crtc::Register::LineCompare, (line_compare & 0x00ff) as u8);
            crtc::write(crtc::Register::Overflow, overflow);
            crtc::write(crtc::Register::MaximumScanLine, max_scan);
            crtc::write(crtc::Register::VerticalRetraceEnd, vertical_retrace_end);
        }

        self.status_line_enabled = true;
    }

    /// Write Line Compare register and related CRTC registers to disable
    /// split-screen status line mode.
    ///
    /// This configures the VGA hardware to disable the split-screen mode, while
    /// keeping the reserved memory space for the status line.
    fn disable_status_line(&mut self)
    {
        let line_compare = 0x03ffu16;
        let mut overflow = crtc::read(crtc::Register::Overflow);
        let mut max_scan = crtc::read(crtc::Register::MaximumScanLine);

        overflow = (overflow & !0x10) | 0x10;
        max_scan = (max_scan & !0x40) | 0x40;

        let vertical_retrace_end = crtc::read(crtc::Register::VerticalRetraceEnd);

        // SAFETY: Writing Line Compare to 0x3ff disables split-screen for normal VGA
        // text modes, and the surrounding register writes preserve unrelated
        // CRTC bits.
        unsafe {
            crtc::write(
                crtc::Register::VerticalRetraceEnd,
                vertical_retrace_end & !0x80,
            );
            crtc::write(crtc::Register::LineCompare, (line_compare & 0x00ff) as u8);
            crtc::write(crtc::Register::Overflow, overflow);
            crtc::write(crtc::Register::MaximumScanLine, max_scan);
            crtc::write(crtc::Register::VerticalRetraceEnd, vertical_retrace_end);
        }

        self.status_line_enabled = false;
    }

    fn update_status_line(&mut self)
    {
        let left_text = self.visual_mode.as_status_text().as_bytes();
        let fg = self.status_line_text_color as u16;
        let bg = self.status_line_bg_color as u16;
        let blank = (' ' as u16) | (fg << 8) | (bg << 12);

        let line = unsafe {
            // SAFETY: `phys_base` points to the reserved physical status line.
            slice::from_raw_parts_mut(self.phys_base as *mut u16, self.cols)
        };

        line.fill(blank);

        let left_len = left_text.len().min(self.cols);

        for (index, byte) in left_text.iter().take(left_len).enumerate() {
            line[index] = (*byte as u16) | (fg << 8) | (bg << 12);
        }
    }
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

#[cfg(test)]
mod tests
{
    fn write_cell(
        ptr: usize,
        index: usize,
        value: u16,
    )
    {
        unsafe {
            // SAFETY: The test writes one 16-bit cell into the VGA text buffer.
            *((ptr as *mut u16).add(index)) = value;
        }
    }

    fn read_cell(
        ptr: usize,
        index: usize,
    ) -> u16
    {
        unsafe {
            // SAFETY: The test reads one 16-bit cell from the VGA text buffer.
            *((ptr as *const u16).add(index))
        }
    }

    fn make_console() -> super::super::VgaConsole
    {
        use super::{CursorType, MemoryRanges, Resolution, VGAColor, VgaConsole};

        VgaConsole::new(
            VGAColor::White,
            VGAColor::Black,
            VGAColor::Black,
            VGAColor::Yellow,
            Resolution::R80_25,
            MemoryRanges::Small,
            CursorType::None,
        )
    }

    fn styled_cell(
        ch: u8,
        fg: super::VGAColor,
        bg: super::VGAColor,
    ) -> u16
    {
        (ch as u16) | ((fg as u16) << 8) | ((bg as u16) << 12)
    }

    fn assert_status_line_text(
        c: &super::VgaConsole,
        text: &str,
        fg: super::VGAColor,
        bg: super::VGAColor,
    )
    {
        for (index, byte) in text.bytes().enumerate() {
            assert_eq!(
                read_cell(c.phys_base, index),
                styled_cell(byte, fg, bg),
                "status line mismatch at cell {index}",
            );
        }

        if text.len() < c.cols {
            assert_eq!(
                read_cell(c.phys_base, text.len()),
                styled_cell(b' ', fg, bg),
                "status line trailing cell should be blank",
            );
        }
    }

    fn assert_user_region_blank(c: &super::VgaConsole)
    {
        for index in 0..(c.vram_size / core::mem::size_of::<u16>()) {
            assert_eq!(read_cell(c.user_base, index), super::BLANK, "cell {index}");
        }
    }

    mod writing
    {
        #[test_case]
        fn write_basic_characters()
        {
            use super::super::VGAColor;
            use super::{make_console, read_cell};
            use core::fmt::Write;

            let mut c = make_console();
            for i in 0..=c.cols * c.rows {
                let ch = char::from(b'0' + u8::try_from(i % 10).unwrap());
                write!(c, "{ch}").unwrap();
            }
            assert_eq!(
                read_cell(c.visible_origin, 0),
                ('0' as u16) | ((VGAColor::White as u16) << 8)
            );
            assert_eq!(
                read_cell(c.visible_origin, c.cols - 1),
                ('9' as u16) | ((VGAColor::White as u16) << 8)
            );
            assert_eq!(
                read_cell(c.visible_origin, ((c.rows - 1) * c.cols) - 1),
                ('9' as u16) | ((VGAColor::White as u16) << 8)
            );
        }
    }

    mod start_of_line
    {
        #[test_case]
        fn returns_current_line_base_for_multiple_positions()
        {
            use super::super::VGA_CELL_SIZE;
            use super::make_console;

            let mut c = make_console();
            let line_stride = c.line_stride();
            let target_line = 7;
            let line_start = c.user_base + (target_line * line_stride);
            let original_index = c.index;

            for (case, position) in [
                ("start", line_start),
                ("after_first_cell", line_start + VGA_CELL_SIZE),
                ("middle", line_start + ((c.cols / 2) * VGA_CELL_SIZE)),
                (
                    "before_last_cell",
                    line_start + ((c.cols - 2) * VGA_CELL_SIZE),
                ),
                ("end", line_start + line_stride - VGA_CELL_SIZE),
            ] {
                assert_eq!(c.start_of_line(position), line_start, "case: {case}");
                assert_eq!(c.index, original_index, "case: {case}");
            }
        }
    }

    mod user_visible_rows
    {
        #[test_case]
        fn returns_total_rows_when_status_line_is_disabled()
        {
            let mut c = super::make_console();

            c.disable_status_line();

            assert_eq!(c.user_visible_rows(), c.rows);
        }

        #[test_case]
        fn returns_one_less_row_when_status_line_is_enabled()
        {
            let mut c = super::make_console();

            c.enable_status_line();

            assert_eq!(c.user_visible_rows(), c.rows - 1);
        }
    }

    mod newlines
    {
        #[test_case]
        fn moves_to_next_row_when_screen_has_space()
        {
            use super::super::VGA_CELL_SIZE;
            use super::make_console;

            let c = make_console();
            let line_stride = c.line_stride();
            let middle_offset = (c.cols / 2) * VGA_CELL_SIZE;
            let end_offset = line_stride - VGA_CELL_SIZE;

            for (case, offset) in [("start", 0), ("middle", middle_offset), ("end", end_offset)] {
                let mut c = make_console();
                let old_origin = c.origin;
                let old_visible_origin = c.visible_origin;
                let old_origin_end = c.origin_end;

                c.index = c.user_base + offset;
                c.new_line();

                assert_eq!(c.index, c.user_base + line_stride, "case: {case}");
                assert_eq!(c.origin, old_origin, "case: {case}");
                assert_eq!(c.visible_origin, old_visible_origin, "case: {case}");
                assert_eq!(c.origin_end, old_origin_end, "case: {case}");
            }
        }

        #[test_case]
        fn uses_backing_store_when_screen_is_full()
        {
            use super::super::BLANK;
            use super::super::VGA_CELL_SIZE;
            use super::{make_console, read_cell, write_cell};

            let c = make_console();
            let line_stride = c.line_stride();
            let middle_offset = (c.cols / 2) * VGA_CELL_SIZE;
            let end_offset = line_stride - VGA_CELL_SIZE;

            for (case, offset) in [("start", 0), ("middle", middle_offset), ("end", end_offset)] {
                let mut c = make_console();
                let old_origin = c.origin;
                let old_origin_end = c.origin_end;
                let last_row_start = old_origin_end - line_stride;

                write_cell(old_origin, 0, 'A' as u16);
                write_cell(old_origin + line_stride, 0, 'B' as u16);
                write_cell(old_origin_end, 0, 'Z' as u16);

                c.index = last_row_start + offset;
                c.new_line();

                assert_eq!(c.origin, old_origin + line_stride, "case: {case}");
                assert_eq!(c.visible_origin, old_origin + line_stride, "case: {case}");
                assert_eq!(c.origin_end, old_origin_end + line_stride, "case: {case}");
                assert_eq!(c.index, old_origin_end, "case: {case}");
                assert_eq!(read_cell(c.visible_origin, 0), 'B' as u16, "case: {case}");
                assert_eq!(read_cell(c.index, 0), BLANK, "case: {case}");
            }
        }

        #[test_case]
        fn scrolls_in_place_at_end_of_vram()
        {
            use super::super::BLANK;
            use super::super::VGA_CELL_SIZE;
            use super::{make_console, read_cell, write_cell};

            let c = make_console();
            let line_stride = c.line_stride();
            let middle_offset = (c.cols / 2) * VGA_CELL_SIZE;
            let end_offset = line_stride - VGA_CELL_SIZE;

            for (case, offset) in [("start", 0), ("middle", middle_offset), ("end", end_offset)] {
                let mut c = make_console();
                let old_origin_end = c.origin_end;
                let last_row_start = old_origin_end - line_stride;

                write_cell(c.user_base, 0, 'A' as u16);
                write_cell(c.user_base + line_stride, 0, 'B' as u16);
                write_cell(last_row_start, 0, 'Y' as u16);

                c.vram_end = old_origin_end;
                c.index = last_row_start + offset;
                c.new_line();

                assert_eq!(c.origin, c.user_base, "case: {case}");
                assert_eq!(c.visible_origin, c.user_base, "case: {case}");
                assert_eq!(c.origin_end, c.user_base + c.screen_size, "case: {case}");
                assert_eq!(c.index, c.origin_end - line_stride, "case: {case}");
                assert_eq!(read_cell(c.visible_origin, 0), 'B' as u16, "case: {case}");
                assert_eq!(read_cell(c.index, 0), BLANK, "case: {case}");
            }
        }
    }

    mod set_mem_start
    {
        #[test_case]
        fn writes_expected_start_address_registers()
        {
            use super::super::crtc;
            use super::make_console;

            let mut c = make_console();
            let line_stride = c.line_stride();
            let visible_origin = c.user_base + (3 * line_stride);

            c.visible_origin = visible_origin;
            c.set_mem_start();

            let start = (u16::from(crtc::read(crtc::Register::StartAddressHigh)) << 8)
                | u16::from(crtc::read(crtc::Register::StartAddressLow));

            assert_eq!(start, ((visible_origin - c.phys_base) / 2) as u16);
        }
    }

    mod scroll_view
    {
        #[test_case]
        fn line_up_updates_visible_origin_and_status_line()
        {
            use super::super::Mode;
            use super::{
                assert_status_line_text, make_console, read_cell, styled_cell, write_cell,
            };

            let mut c = make_console();
            let line_stride = c.cols * core::mem::size_of::<u16>();
            let origin = c.user_base + (8 * line_stride);
            let visible_origin = c.user_base + (5 * line_stride);

            for line in 0..=8 {
                let byte = b'0' + u8::try_from(line).unwrap();
                write_cell(
                    c.user_base + (line * line_stride),
                    0,
                    styled_cell(byte, c.foreground_color, c.background_color),
                );
            }

            c.origin = origin;
            c.origin_end = origin + c.screen_size;
            c.visible_origin = visible_origin;

            c.scroll_view(super::super::VisualAction::ViewLinesUp(2));

            assert_eq!(c.visible_origin, c.user_base + (3 * line_stride));
            assert_eq!(c.visual_mode, Mode::Visual);
            assert_eq!(
                read_cell(c.visible_origin, 0),
                styled_cell(b'3', c.foreground_color, c.background_color)
            );
            assert_status_line_text(
                &c,
                Mode::Visual.as_status_text(),
                c.status_line_text_color,
                c.status_line_bg_color,
            );
        }

        #[test_case]
        fn line_down_moves_towards_output_origin()
        {
            use super::{make_console, read_cell, styled_cell, write_cell};

            let mut c = make_console();
            let line_stride = c.cols * core::mem::size_of::<u16>();
            let origin = c.user_base + (8 * line_stride);
            let visible_origin = c.user_base + (2 * line_stride);

            for line in 0..=8 {
                let byte = b'0' + u8::try_from(line).unwrap();
                write_cell(
                    c.user_base + (line * line_stride),
                    0,
                    styled_cell(byte, c.foreground_color, c.background_color),
                );
            }

            c.origin = origin;
            c.origin_end = origin + c.screen_size;
            c.visible_origin = visible_origin;

            c.scroll_view(super::super::VisualAction::ViewLinesDown(3));

            assert_eq!(c.visible_origin, c.user_base + (5 * line_stride));
            assert_eq!(
                read_cell(c.visible_origin, 0),
                styled_cell(b'5', c.foreground_color, c.background_color)
            );
        }

        #[test_case]
        fn page_up_clamps_to_top()
        {
            use super::{make_console, read_cell, styled_cell, write_cell};

            let mut c = make_console();
            let line_stride = c.cols * core::mem::size_of::<u16>();
            let origin = c.user_base + (30 * line_stride);
            let visible_origin = c.user_base + (10 * line_stride);

            for line in 0..=30 {
                let byte = b'A' + u8::try_from(line % 26).unwrap();
                write_cell(
                    c.user_base + (line * line_stride),
                    0,
                    styled_cell(byte, c.foreground_color, c.background_color),
                );
            }

            c.origin = origin;
            c.origin_end = origin + c.screen_size;
            c.visible_origin = visible_origin;

            c.scroll_view(super::super::VisualAction::ViewPagesUp(1));

            assert_eq!(c.visible_origin, c.user_base);
            assert_eq!(
                read_cell(c.visible_origin, 0),
                styled_cell(b'A', c.foreground_color, c.background_color)
            );
        }

        #[test_case]
        fn page_down_moves_by_a_full_screen()
        {
            use super::{make_console, read_cell, styled_cell, write_cell};

            let mut c = make_console();
            let line_stride = c.cols * core::mem::size_of::<u16>();
            let origin = c.user_base + (40 * line_stride);
            let visible_origin = c.user_base + (2 * line_stride);

            for line in 0..=40 {
                let byte = b'A' + u8::try_from(line % 26).unwrap();
                write_cell(
                    c.user_base + (line * line_stride),
                    0,
                    styled_cell(byte, c.foreground_color, c.background_color),
                );
            }

            c.origin = origin;
            c.origin_end = origin + c.screen_size;
            c.visible_origin = visible_origin;

            c.scroll_view(super::super::VisualAction::ViewPagesDown(1));

            assert_eq!(c.visible_origin, visible_origin + c.screen_size);
            assert_eq!(
                read_cell(c.visible_origin, 0),
                styled_cell(b'B', c.foreground_color, c.background_color)
            );
        }

        #[test_case]
        fn to_top_moves_to_first_visible_line()
        {
            use super::{make_console, read_cell, styled_cell, write_cell};

            let mut c = make_console();
            let line_stride = c.cols * core::mem::size_of::<u16>();
            let origin = c.user_base + (12 * line_stride);
            let visible_origin = c.user_base + (7 * line_stride);

            for line in 0..=12 {
                let byte = b'0' + u8::try_from(line % 10).unwrap();
                write_cell(
                    c.user_base + (line * line_stride),
                    0,
                    styled_cell(byte, c.foreground_color, c.background_color),
                );
            }

            c.origin = origin;
            c.origin_end = origin + c.screen_size;
            c.visible_origin = visible_origin;

            c.scroll_view(super::super::VisualAction::ToTop);

            assert_eq!(c.visible_origin, c.user_base);
            assert_eq!(
                read_cell(c.visible_origin, 0),
                styled_cell(b'0', c.foreground_color, c.background_color)
            );
        }

        #[test_case]
        fn to_bottom_moves_to_output_origin()
        {
            use super::{make_console, read_cell, styled_cell, write_cell};

            let mut c = make_console();
            let line_stride = c.cols * core::mem::size_of::<u16>();
            let origin = c.user_base + (12 * line_stride);
            let visible_origin = c.user_base + (2 * line_stride);

            for line in 0..=12 {
                let byte = b'0' + u8::try_from(line % 10).unwrap();
                write_cell(
                    c.user_base + (line * line_stride),
                    0,
                    styled_cell(byte, c.foreground_color, c.background_color),
                );
            }

            c.origin = origin;
            c.origin_end = origin + c.screen_size;
            c.visible_origin = visible_origin;

            c.scroll_view(super::super::VisualAction::ToBottom);

            assert_eq!(c.visible_origin, origin);
            assert_eq!(
                read_cell(c.visible_origin, 0),
                styled_cell(b'2', c.foreground_color, c.background_color)
            );
        }

        #[test_case]
        fn follow_output_restores_terminal_mode_and_status_line()
        {
            use super::super::Mode;
            use super::{
                assert_status_line_text, make_console, read_cell, styled_cell, write_cell,
            };

            let mut c = make_console();
            let line_stride = c.cols * core::mem::size_of::<u16>();
            let origin = c.user_base + (30 * line_stride);
            let visible_origin = c.user_base + (5 * line_stride);
            let current_line_start = c.user_base + (28 * line_stride);

            for line in 0..=30 {
                let byte = b'A' + u8::try_from(line % 26).unwrap();
                write_cell(
                    c.user_base + (line * line_stride),
                    0,
                    styled_cell(byte, c.foreground_color, c.background_color),
                );
            }

            c.origin = origin;
            c.origin_end = origin + c.screen_size;
            c.visible_origin = visible_origin;
            c.index = current_line_start + (3 * core::mem::size_of::<u16>());

            c.scroll_view(super::super::VisualAction::FollowOutput);

            assert_eq!(c.visible_origin, c.user_base + (4 * line_stride));
            assert_eq!(c.visual_mode, Mode::Terminal);
            assert_eq!(
                read_cell(c.visible_origin, 0),
                styled_cell(b'E', c.foreground_color, c.background_color)
            );
            assert_status_line_text(
                &c,
                Mode::Terminal.as_status_text(),
                c.status_line_text_color,
                c.status_line_bg_color,
            );
        }
    }

    mod blank
    {
        fn assert_blank_state(c: &super::super::VgaConsole)
        {
            use super::super::Mode;
            use super::super::crtc;
            use super::{assert_status_line_text, assert_user_region_blank};

            assert_eq!(c.origin, c.user_base);
            assert_eq!(c.visible_origin, c.user_base);
            assert_eq!(c.index, c.user_base);
            assert_eq!(c.origin_end, c.user_base + c.screen_size);
            assert_eq!(c.visual_mode, Mode::Terminal);
            assert_user_region_blank(c);

            let start = (u16::from(crtc::read(crtc::Register::StartAddressHigh)) << 8)
                | u16::from(crtc::read(crtc::Register::StartAddressLow));
            let cursor = (u16::from(crtc::read(crtc::Register::CursorLocationHigh)) << 8)
                | u16::from(crtc::read(crtc::Register::CursorLocationLow));

            assert_eq!(start, ((c.user_base - c.phys_base) / 2) as u16);
            assert_eq!(cursor, ((c.user_base - c.phys_base) / 2) as u16);
            assert_status_line_text(
                c,
                Mode::Terminal.as_status_text(),
                c.status_line_text_color,
                c.status_line_bg_color,
            );
        }

        #[test_case]
        fn clears_a_fresh_buffer()
        {
            let mut c = super::make_console();

            c.blank();

            assert_blank_state(&c);
        }

        #[test_case]
        fn clears_buffer_after_plain_text_output()
        {
            let mut c = super::make_console();

            c.putstr("hello\nworld\nmore text");
            c.blank();

            assert_blank_state(&c);
        }

        #[test_case]
        fn clears_buffer_with_colored_cells_and_scrolled_state()
        {
            use super::super::{Mode, VGAColor};
            use super::{styled_cell, write_cell};

            let mut c = super::make_console();
            let line_stride = c.cols * core::mem::size_of::<u16>();

            write_cell(
                c.user_base,
                0,
                styled_cell(b'A', VGAColor::LightRed, VGAColor::Blue),
            );
            write_cell(
                c.user_base,
                1,
                styled_cell(b'B', VGAColor::Yellow, VGAColor::Red),
            );
            write_cell(
                c.user_base + (3 * line_stride),
                5,
                styled_cell(b'C', VGAColor::LightGreen, VGAColor::Magenta),
            );

            c.origin = c.user_base + (2 * line_stride);
            c.visible_origin = c.user_base + line_stride;
            c.origin_end = c.origin + c.screen_size;
            c.index = c.origin + (7 * core::mem::size_of::<u16>());
            c.visual_mode = Mode::Visual;

            c.blank();

            assert_blank_state(&c);
        }
    }

    mod cursor
    {
        #[test_case]
        fn cursor_size_programs_cursor_shape_registers()
        {
            use super::super::MAX_SCANLINE_MASK;
            use super::super::crtc;

            let c = super::make_console();
            let max_scanline = crtc::read(crtc::Register::MaximumScanLine) & MAX_SCANLINE_MASK;
            let cursor_start_high = crtc::read(crtc::Register::CursorStart) & 0xc0;
            let cursor_end_high = crtc::read(crtc::Register::CursorEnd) & 0xe0;
            let end_scanline = max_scanline.min(5);
            let start_scanline = end_scanline.min(2);

            unsafe {
                crtc::write(
                    crtc::Register::CursorStart,
                    cursor_start_high | 0x20 | max_scanline.min(7),
                );
                crtc::write(
                    crtc::Register::CursorEnd,
                    cursor_end_high | max_scanline.min(7),
                );
            }

            c.cursor_size(start_scanline, end_scanline);

            assert_eq!(
                crtc::read(crtc::Register::CursorStart),
                cursor_start_high | start_scanline
            );
            assert_eq!(
                crtc::read(crtc::Register::CursorEnd),
                cursor_end_high | end_scanline
            );
        }

        #[test_case]
        fn enable_cursor_clears_disable_bit()
        {
            use super::super::MAX_SCANLINE_MASK;
            use super::super::crtc;

            let mut c = super::make_console();
            let max_scanline = crtc::read(crtc::Register::MaximumScanLine) & MAX_SCANLINE_MASK;
            let cursor_start_high = crtc::read(crtc::Register::CursorStart) & 0xc0;

            unsafe {
                crtc::write(
                    crtc::Register::CursorStart,
                    cursor_start_high | 0x20 | max_scanline.min(5),
                );
            }
            c.enable_cursor();

            assert_eq!(
                crtc::read(crtc::Register::CursorStart),
                cursor_start_high | max_scanline.min(5)
            );
        }

        #[test_case]
        fn disable_cursor_sets_disable_bit()
        {
            use super::super::MAX_SCANLINE_MASK;
            use super::super::crtc;

            let mut c = super::make_console();
            let max_scanline = crtc::read(crtc::Register::MaximumScanLine) & MAX_SCANLINE_MASK;
            let cursor_start_high = crtc::read(crtc::Register::CursorStart) & 0xc0;

            unsafe {
                crtc::write(
                    crtc::Register::CursorStart,
                    cursor_start_high | max_scanline.min(5),
                );
            }
            c.disable_cursor();

            assert_eq!(
                crtc::read(crtc::Register::CursorStart),
                cursor_start_high | 0x20 | max_scanline.min(5)
            );
        }
    }

    mod status_line
    {
        #[test_case]
        fn update_status_line_writes_text_and_colors_to_reserved_line()
        {
            use super::super::{Mode, VGAColor};
            use super::{assert_status_line_text, make_console};

            let mut c = make_console();

            c.status_line_text_color = VGAColor::LightRed;
            c.status_line_bg_color = VGAColor::Blue;
            c.visual_mode = Mode::Visual;
            c.update_status_line();

            assert_status_line_text(
                &c,
                Mode::Visual.as_status_text(),
                VGAColor::LightRed,
                VGAColor::Blue,
            );
        }

        #[test_case]
        fn enable_status_line_programs_line_compare_registers()
        {
            use super::super::crtc;
            use super::make_console;

            let mut c = make_console();
            let vertical_retrace_end = crtc::read(crtc::Register::VerticalRetraceEnd);

            c.disable_status_line();
            c.enable_status_line();

            let line_compare = u16::from(crtc::read(crtc::Register::LineCompare))
                | if (crtc::read(crtc::Register::Overflow) & 0x10) != 0 {
                    0x0100
                } else {
                    0
                }
                | if (crtc::read(crtc::Register::MaximumScanLine) & 0x40) != 0 {
                    0x0200
                } else {
                    0
                };

            assert!(c.status_line_enabled);
            assert_eq!(line_compare, ((c.rows - 1) * c.scanlines_per_row()) as u16);
            assert_eq!(
                crtc::read(crtc::Register::VerticalRetraceEnd),
                vertical_retrace_end
            );
        }

        #[test_case]
        fn disable_status_line_programs_line_compare_registers()
        {
            use super::super::crtc;
            use super::make_console;

            let mut c = make_console();
            let vertical_retrace_end = crtc::read(crtc::Register::VerticalRetraceEnd);

            c.disable_status_line();

            let line_compare = u16::from(crtc::read(crtc::Register::LineCompare))
                | if (crtc::read(crtc::Register::Overflow) & 0x10) != 0 {
                    0x0100
                } else {
                    0
                }
                | if (crtc::read(crtc::Register::MaximumScanLine) & 0x40) != 0 {
                    0x0200
                } else {
                    0
                };

            assert!(!c.status_line_enabled);
            assert_eq!(line_compare, 0x03ff);
            assert_eq!(
                crtc::read(crtc::Register::VerticalRetraceEnd),
                vertical_retrace_end
            );
        }
    }

    mod resize
    {
        #[test_case]
        fn updates_console_state_and_crtc_registers()
        {
            use super::super::Mode;
            use super::super::crtc;
            use super::{assert_status_line_text, assert_user_region_blank, make_console};

            let mut c = make_console();
            let total_vram_size = c.vram_end - c.phys_base;
            let vertical_retrace_end = crtc::read(crtc::Register::VerticalRetraceEnd);

            c.putstr("resize me");
            c.resize(40, 10);

            let expected_line_stride = 40 * core::mem::size_of::<u16>();
            let expected_user_base = c.phys_base + expected_line_stride;
            let expected_vram_size = total_vram_size - expected_line_stride;
            let expected_screen_size = 40 * 10 * core::mem::size_of::<u16>();
            let expected_vertical_display_end = (10 * c.scanlines_per_row()) - 1;
            let expected_overflow_bits = (if (expected_vertical_display_end & 0x100) != 0 {
                0x02
            } else {
                0x00
            }) | if (expected_vertical_display_end & 0x200) != 0 {
                0x40
            } else {
                0x00
            };

            assert_eq!(c.cols, 40);
            assert_eq!(c.rows, 10);
            assert_eq!(c.user_base, expected_user_base);
            assert_eq!(c.vram_size, expected_vram_size);
            assert_eq!(c.screen_size, expected_screen_size);
            assert_eq!(c.origin, expected_user_base);
            assert_eq!(c.visible_origin, expected_user_base);
            assert_eq!(c.index, expected_user_base);
            assert_eq!(c.origin_end, expected_user_base + expected_screen_size);
            assert_eq!(c.visual_mode, Mode::Terminal);
            assert!(c.status_line_enabled);

            let line_compare = u16::from(crtc::read(crtc::Register::LineCompare))
                | if (crtc::read(crtc::Register::Overflow) & 0x10) != 0 {
                    0x0100
                } else {
                    0
                }
                | if (crtc::read(crtc::Register::MaximumScanLine) & 0x40) != 0 {
                    0x0200
                } else {
                    0
                };
            let start = (u16::from(crtc::read(crtc::Register::StartAddressHigh)) << 8)
                | u16::from(crtc::read(crtc::Register::StartAddressLow));
            let cursor = (u16::from(crtc::read(crtc::Register::CursorLocationHigh)) << 8)
                | u16::from(crtc::read(crtc::Register::CursorLocationLow));

            assert_eq!(crtc::read(crtc::Register::HorizontalDisplayEnd), 39);
            assert_eq!(
                crtc::read(crtc::Register::VerticalDisplayEnd),
                (expected_vertical_display_end & 0xff) as u8
            );
            assert_eq!(crtc::read(crtc::Register::Offset), 20);
            assert_eq!(
                crtc::read(crtc::Register::Overflow) & 0x42,
                expected_overflow_bits
            );
            assert_eq!(
                crtc::read(crtc::Register::VerticalRetraceEnd),
                vertical_retrace_end
            );
            assert_eq!(line_compare, ((c.rows - 1) * c.scanlines_per_row()) as u16);
            assert_eq!(start, ((expected_user_base - c.phys_base) / 2) as u16);
            assert_eq!(cursor, ((expected_user_base - c.phys_base) / 2) as u16);

            assert_user_region_blank(&c);
            assert_status_line_text(
                &c,
                Mode::Terminal.as_status_text(),
                c.status_line_text_color,
                c.status_line_bg_color,
            );
        }
    }
}
