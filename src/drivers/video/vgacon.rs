/*
 * http://www.osdever.net/FreeVGA/home.htm
 * https://github.com/torvalds/linux/blob/master/drivers/video/console/vgacon.c
 */

use core::cmp;
use core::fmt;

use crate::io::{inb, outb};

pub const BLANK: u16 = 0x0000;
pub const VGA_VRAM_BASE: *mut u16 = 0xb8000 as _;
pub const VGA_INDEX_MARK: u16 = 0x0530;
pub const VGACON_C: usize = 80;
pub const VGACON_R: usize = 25;

/**
 * CRT Controller
 */

/* CRT Controller Registers */
pub const VGA_CRT_DR: u16 = 0x3D5;
pub const VGA_CRT_AR: u16 = 0x3D4;

const CURSOR_ENABLE_MASK: u8 = 0xdf;
const CURSOR_DISABLE_MASK: u8 = 0x20;

/* CRT Controller Indexes */
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CTRCRegistersIndexes
{
    VgaCrtcHTotal,
    VgaCrtcHDisp,
    VgaCrtcHBlankStart,
    VgaCrtcHBlankEnd,
    VgaCrtcHSyncStart,
    VgaCrtcHSyncEnd,
    VgaCrtcVTotal,
    VgaCrtcOverflow,
    VgaCrtcPresetRow,
    VgaCrtcMaxScan,
    VgaCrtcCursorStart,
    VgaCrtcCursorEnd,
    VgaCrtcStartHi,
    VgaCrtcStartLo,
    VgaCrtcCursorHi,
    VgaCrtcCursorLo,
    VgaCrtcVSyncStart,
    VgaCrtcVSyncEnd,
    VgaCrtcVDispEnd,
    VgaCrtcOffset,
    VgaCrtcUnderline,
    VgaCrtcVBlankStart,
    VgaCrtcVBlankEnd,
    VgaCrtcMode,
    VgaCrtcLineCompare,
}

#[inline(always)]
pub fn ctrc_write(
    index: u8,
    value: u8,
)
{
    unsafe {
        outb(VGA_CRT_AR, index);
        outb(VGA_CRT_DR, value);
    }
}

#[inline(always)]
fn ctrc_read(index: u8) -> u8
{
    unsafe {
        outb(VGA_CRT_AR, index);
        inb(VGA_CRT_DR)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDir
{
    ScUp,
    ScDown,
    GoToBottom,
    GoToTop,
}

#[derive(Debug)]
pub enum VgaConIndicators
{
    Visual = 0xe056,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BlankingMode
{
    BlankScreen,
    BlankScreenVisibleBuffer,
    BlankFullBuffer,
    BlankAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color
{
    Black      = 0,
    Blue       = 1,
    Green      = 2,
    Cyan       = 3,
    Red        = 4,
    Magenta    = 5,
    Brown      = 6,
    LightGray  = 7,
    DarkGray   = 8,
    LightBlue  = 9,
    LightGreen = 10,
    LightCyan  = 11,
    LightRed   = 12,
    Pink       = 13,
    Yellow     = 14,
    White      = 15,
}

/*
 * vc_screenbuf ------> +---------------+-.
 *                      | ------------> |  \
 *                      | | screen_size |   |
 *                      | |             |    > area 1
 *                      | |             |   |
 *                      | v             |  /
 *                      +---------------+-:
 *                      |               |  \
 * vc_visible_origin > ^| $> ls         |   |
 *                     || file          |    > area 2
 *           vc_rows  < | file2         |   |
 *                     || $> cat file   |  /
 * vc_origin --------> |+---------------+-:
 *                     || Hello         |  \
 *                     v| $> uname      |   |
 *                      | Darwin        |    > area 3
 * vc_index ------------|--------v      |   |
 *                      | $> echo       |  /
 *                      +---------------+-'
 *                      <--- vc_cols --->
 */
#[derive(Debug)]
pub struct VgaCon<const R: usize, const C: usize, const A: usize>
where
    [(); R * C * A]:,
{
    pub vc_num:              u8,
    pub vc_voffset:          usize,
    pub vc_hoffset:          usize,
    pub vc_index:            usize,
    pub vc_screen_size:      usize,
    pub vc_foreground_color: Color,
    pub vc_background_color: Color,
    pub vc_screenbuf:        [u16; R * C * A],
    pub vc_screenbuf_size:   usize,
    pub vc_visible_origin:   usize,
    pub vc_origin:           usize,
    pub vc_rows:             usize,
    pub vc_cols:             usize,
}

impl<const R: usize, const C: usize, const A: usize> VgaCon<R, C, A>
where
    [(); R * C * A]:,
{
    pub const fn new(
        id: u8,
        horizontal_offset: usize,
        vertical_offset: usize,
        foreground_color: Color,
        background_color: Color,
    ) -> Self
    {
        assert!(id < 10, "VGA Index must be lower than 10");

        Self {
            vc_num:              id,
            vc_voffset:          vertical_offset,
            vc_hoffset:          horizontal_offset,
            vc_index:            C * (R - 1),
            vc_screen_size:      R * C,
            vc_foreground_color: foreground_color,
            vc_background_color: background_color,
            vc_screenbuf:        [BLANK; R * C * A],
            vc_screenbuf_size:   R * C * A,
            vc_visible_origin:   C * R * (A - 1),
            vc_origin:           C * R * (A - 1),
            vc_rows:             R,
            vc_cols:             C,
        }
    }

    #[inline(always)]
    fn _write(
        &mut self,
        index: usize,
        word: u16,
    )
    {
        unsafe {
            *VGA_VRAM_BASE.offset(
                ((((index / self.vc_cols) + self.vc_voffset) * VGACON_C)
                    + self.vc_hoffset
                    + (index % self.vc_cols)) as isize,
            ) = word;
        }
    }

    #[inline(always)]
    pub fn putc(
        &mut self,
        c: u8,
    )
    {
        self.cputc(c, None, None);
    }

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

        if self.vc_visible_origin != self.vc_origin {
            self.scroll(ScrollDir::GoToBottom, None);
        }

        if self.vc_index == self.vc_screen_size {
            self.scroll(ScrollDir::ScDown, Some(1));
        }

        self.vc_screenbuf[self.vc_index + self.vc_origin] = word;
        self._write(self.vc_index, word);
        self.vc_index += 1;
        self.cursor_update();
    }

    #[inline(always)]
    pub fn putstr(
        &mut self,
        str: &str,
    )
    {
        self.cputstr(str, None, None);
    }

    pub fn cputstr(
        &mut self,
        str: &str,
        foreground: Option<u8>,
        background: Option<u8>,
    )
    {
        for byte in str.bytes() {
            match byte {
                b'\n' => self.scroll(ScrollDir::ScDown, Some(1)),
                0x20..=0x7e => self.cputc(byte, foreground, background),
                _ => self.cputc(0xfe, None, None),
            };
        }
    }

    pub fn cursor(
        &mut self,
        enable: bool,
    )
    {
        let mut c = ctrc_read(CTRCRegistersIndexes::VgaCrtcCursorStart as u8);

        if enable {
            c &= CURSOR_ENABLE_MASK;
        } else {
            c |= CURSOR_DISABLE_MASK;
        }

        ctrc_write(CTRCRegistersIndexes::VgaCrtcCursorStart as u8, c);
    }

    pub fn cursor_size(
        &mut self,
        from: u8,
        to: u8,
    )
    {
        if from > 16 || to > 16 {
            return;
        }

        let mut c_start: u8 = ctrc_read(CTRCRegistersIndexes::VgaCrtcCursorStart as u8);
        let mut c_end: u8 = ctrc_read(CTRCRegistersIndexes::VgaCrtcCursorEnd as u8);

        c_start = (c_start & 0xc0) | from;
        c_end = (c_end & 0xe0) | to;

        ctrc_write(CTRCRegistersIndexes::VgaCrtcCursorStart as u8, c_start);
        ctrc_write(CTRCRegistersIndexes::VgaCrtcCursorEnd as u8, c_end);
    }

    pub fn cursor_update(&mut self)
    {
        let pos: usize = (((self.vc_index / self.vc_cols) + self.vc_voffset) * VGACON_C)
            + self.vc_hoffset
            + (self.vc_index % self.vc_cols);

        ctrc_write(CTRCRegistersIndexes::VgaCrtcCursorLo as u8, pos as u8);
        ctrc_write(
            CTRCRegistersIndexes::VgaCrtcCursorHi as u8,
            (pos >> 8) as u8,
        );
    }

    pub fn scroll(
        &mut self,
        dir: ScrollDir,
        lines: Option<usize>,
    )
    {
        let delta = lines.unwrap_or(0) * self.vc_cols;
        match dir {
            ScrollDir::ScUp if lines.is_some() => {
                self.vc_visible_origin = self.vc_visible_origin.saturating_sub(delta);
            }
            ScrollDir::ScDown if lines.is_some() => {
                /* Number of new lines */
                let adjusted_delta = cmp::min(
                    self.vc_visible_origin
                        .saturating_add(delta)
                        .saturating_sub(self.vc_origin),
                    self.vc_screenbuf_size,
                );

                self.vc_visible_origin = cmp::min(self.vc_visible_origin + delta, self.vc_origin);
                self.vc_screenbuf.copy_within(adjusted_delta.., 0);
                self.vc_screenbuf[(self.vc_screenbuf_size - adjusted_delta)..].fill(0);
                self.vc_index = self.vc_cols * (self.vc_rows - 1)
            }
            ScrollDir::GoToBottom => {
                self.vc_visible_origin = self.vc_origin;
            }
            ScrollDir::GoToTop => {
                self.vc_visible_origin = 0;
            }
            _ => {}
        }
        self.restore();
    }

    pub fn blank(
        &mut self,
        mode: BlankingMode,
    )
    {
        if matches!(
            mode,
            BlankingMode::BlankScreen
                | BlankingMode::BlankScreenVisibleBuffer
                | BlankingMode::BlankAll
        ) {
            for i in 0..(self.vc_rows * self.vc_cols) {
                self._write(i, 0x0000);
            }
        }
        if matches!(
            mode,
            BlankingMode::BlankScreenVisibleBuffer | BlankingMode::BlankAll
        ) {
            self.vc_screenbuf[self.vc_visible_origin..self.vc_visible_origin + self.vc_screen_size]
                .fill(BLANK);
        }
        if matches!(mode, BlankingMode::BlankFullBuffer | BlankingMode::BlankAll) {
            self.vc_screenbuf.fill(BLANK);
        }
    }

    pub fn restore(&mut self)
    {
        for i in 0..(self.vc_rows * self.vc_cols) {
            self._write(i, self.vc_screenbuf[self.vc_visible_origin + i]);
        }
        self._write(self.vc_cols - 1, VGA_INDEX_MARK + self.vc_num as u16);

        if self.vc_visible_origin != self.vc_origin {
            self._write(self.vc_cols - 2, VgaConIndicators::Visual as _);
        }
    }
}

impl<const R: usize, const C: usize, const A: usize> fmt::Write for VgaCon<R, C, A>
where
    [(); R * C * A]:,
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
fn test_screen1()
{
    let mut vga: VgaCon<25, 80, 1> = VgaCon::new(1u8, 0, 0, Color::White, Color::Black);
    vga.putc('>' as u8);
    for _i in 0..24 {
        vga.putstr("\n");
    }
    unsafe {
        assert_eq!(*VGA_VRAM_BASE.offset(0), 0x0f3e);
    }
    vga.blank(BlankingMode::BlankAll);
}

#[test_case]
fn test_screen2()
{
    let mut vga: VgaCon<25, 80, 1> = VgaCon::new(1u8, 0, 0, Color::White, Color::Black);
    for _i in 0..((25 * 80) - 1) {
        vga.putc('>' as u8);
    }
    unsafe {
        assert_eq!(*VGA_VRAM_BASE.offset((25 * 80) - 2), 0x0f3e);
        assert_eq!(*VGA_VRAM_BASE.offset((25 * 80) - 1), 0x0000);
    }
    vga.blank(BlankingMode::BlankAll);
}

#[test_case]
fn test_scroll()
{
    let mut vga: VgaCon<25, 40, 3> = VgaCon::new(1u8, 20, 0, Color::White, Color::Black);
    for _i in 0..50 {
        vga.putstr(">\n");
    }
    vga.scroll(ScrollDir::ScUp, Some(500));
    unsafe {
        assert_eq!(*VGA_VRAM_BASE.offset(20), 0x0000);
    }
    vga.scroll(ScrollDir::ScDown, Some(26));
    unsafe {
        assert_eq!(*VGA_VRAM_BASE.offset((VGACON_C as isize * 24) + 20), 0x0f3e);
    }
    vga.scroll(ScrollDir::ScDown, Some(1000));
    unsafe {
        assert_eq!(*VGA_VRAM_BASE.offset(20), 0x0000);
    }
    vga.blank(BlankingMode::BlankAll);
}

#[test_case]
fn test_indicator()
{
    let mut vga: VgaCon<25, 80, 2> = VgaCon::new(1u8, 0, 0, Color::White, Color::Black);
    vga.scroll(ScrollDir::ScUp, Some(5));
    unsafe {
        assert_eq!(
            *VGA_VRAM_BASE.offset((VGACON_C - 2) as isize),
            VgaConIndicators::Visual as u16
        );
        assert_eq!(
            *VGA_VRAM_BASE.offset((VGACON_C - 1) as isize),
            VGA_INDEX_MARK + 1
        );
    }
    vga.scroll(ScrollDir::ScDown, Some(5));
    unsafe {
        assert_eq!(*VGA_VRAM_BASE.offset((VGACON_C - 2) as isize), 0x0000);
        assert_eq!(
            *VGA_VRAM_BASE.offset((VGACON_C - 1) as isize),
            VGA_INDEX_MARK + 1
        );
    }
    vga.blank(BlankingMode::BlankAll);
    let mut vga2: VgaCon<25, 80, 1> = VgaCon::new(2u8, 0, 0, Color::White, Color::Black);
    vga2.scroll(ScrollDir::ScUp, Some(5));
    unsafe {
        assert_ne!(
            *VGA_VRAM_BASE.offset((VGACON_C - 2) as isize),
            VgaConIndicators::Visual as u16
        );
        assert_eq!(
            *VGA_VRAM_BASE.offset((VGACON_C - 1) as isize),
            VGA_INDEX_MARK + 2
        );
    }
}
