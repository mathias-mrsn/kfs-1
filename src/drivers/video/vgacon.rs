/*
 * http://www.osdever.net/FreeVGA/home.htm
 * https://github.com/torvalds/linux/blob/master/drivers/video/console/vgacon.c
 * TODO: Change few types (usize to u32, etc...)
 * TODO: Create new tests
 */

use core::cmp;
use core::fmt;
use core::ptr;

use crate::io::{inb, outb};

pub const BLANK: u16 = 0x0000;
pub const VGA_VRAM_BASE: *mut u16 = 0xb8000 as _;
pub const VGA_INDEX_MARK: u16 = 0x0530;
pub const VGACON_C: usize = 80;
pub const VGACON_R: usize = 25;

pub const VGA_CRT_DR: u16 = 0x3D5;
pub const VGA_CRT_AR: u16 = 0x3D4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
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

// #[allow(dead_code)]
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

#[derive(Debug)]
pub struct VgaCon<const R: usize, const C: usize, const A: usize>
where
    [(); R * C * A]:,
{
    pub vc_num:              u8,
    pub vc_offset:           usize,
    pub vc_lock:             bool,
    pub vc_index:            usize,
    pub vc_screen_size:      usize,
    pub vc_foreground_color: Color,
    pub vc_background_color: Color,
    pub vc_screenbuf:        [u16; R * C * A],
    pub vc_screenbuf_size:   usize,
    pub vc_visible_origin:   usize,
    pub vc_rows:             usize,
    pub vc_cols:             usize,
}

impl<const R: usize, const C: usize, const A: usize> VgaCon<R, C, A>
where
    [(); R * C * A]:,
{
    pub const fn new(
        id: u8,
        offset: usize,
        foreground_color: Color,
        background_color: Color,
    ) -> Self
    {
        debug_assert!(id < 10, "VGA Index must be lower than 10");

        Self {
            vc_num:              id,
            vc_offset:           offset,
            vc_lock:             false,
            vc_index:            0,
            vc_screen_size:      R * C,
            vc_foreground_color: foreground_color,
            vc_background_color: background_color,
            vc_screenbuf:        [BLANK; R * C * A],
            vc_screenbuf_size:   R * C * A,
            vc_visible_origin:   0,
            vc_rows:             R,
            vc_cols:             C,
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
        if self.vc_lock {
            return;
        }

        let bg_color = background.unwrap_or(self.vc_background_color as u8) & 0xf;
        let fg_color = foreground.unwrap_or(self.vc_foreground_color as u8) & 0xf;
        let byte = (c as u16) | ((bg_color as u16) << 12) | ((fg_color as u16) << 8);

        if self.vc_visible_origin != 0 {
            self.scroll(ScrollDir::GoToBottom, None);
        }

        if self.vc_index >= self.vc_cols {
            self.scroll(ScrollDir::ScDown, Some(1));
        }

        self.vc_screenbuf[self.vc_index] = byte;
        unsafe {
            let offset = ((self.vc_rows - 1) * VGACON_C + self.vc_index) as isize;
            *VGA_VRAM_BASE.offset(self.vc_offset as isize + offset) = byte;
        }

        self.vc_index += 1;
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
            c &= 0xdf;
        } else {
            c |= 0x20;
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

    pub fn scroll(
        &mut self,
        dir: ScrollDir,
        lines: Option<usize>,
    )
    {
        let delta = lines.unwrap_or(0) * self.vc_cols;
        match dir {
            ScrollDir::ScUp if lines.is_some() => {
                self.vc_visible_origin = cmp::min(
                    self.vc_visible_origin + delta,
                    self.vc_screenbuf_size - self.vc_screen_size,
                );
                self.restore();
            }
            ScrollDir::ScDown if lines.is_some() => {
                if delta < self.vc_visible_origin {
                    self.vc_visible_origin -= delta;
                } else {
                    let adjusted_delta =
                        cmp::min(delta - self.vc_visible_origin, self.vc_screenbuf_size);
                    self.vc_screenbuf
                        .copy_within(..(self.vc_screenbuf_size - adjusted_delta), adjusted_delta);
                    self.vc_screenbuf[..adjusted_delta].fill(0);
                    self.vc_visible_origin = 0;
                    self.vc_index = 0;
                }
            }
            ScrollDir::GoToBottom => {
                self.vc_visible_origin = 0;
            }
            ScrollDir::GoToTop => {
                self.vc_visible_origin = self.vc_screenbuf_size - self.vc_screen_size
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
        if mode == BlankingMode::BlankScreen
            || mode == BlankingMode::BlankScreenVisibleBuffer
            || mode == BlankingMode::BlankAll
        {
            unsafe {
                ptr::write_bytes(VGA_VRAM_BASE.add(self.vc_offset), 0x00, self.vc_screen_size);
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
        for rows in 0..self.vc_rows {
            for cols in 0..self.vc_cols {
                unsafe {
                    *VGA_VRAM_BASE.offset(
                        ((((self.vc_rows - rows - 1) * VGACON_C) + cols) + self.vc_offset) as isize,
                    ) = self.vc_screenbuf
                        [(rows * self.vc_cols) + cols + self.vc_visible_origin as usize];
                }
            }
        }

        unsafe {
            *VGA_VRAM_BASE.offset(((self.vc_cols - 1) + self.vc_offset) as isize) =
                VGA_INDEX_MARK + self.vc_num as u16;
        }

        if self.vc_visible_origin != 0 {
            unsafe {
                *VGA_VRAM_BASE.offset(((self.vc_cols - 2) + self.vc_offset) as isize) =
                    VgaConIndicators::Visual as _;
            }
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
    let mut vga: VgaCon<25, 80, 1> = VgaCon::new(1u8, 0, Color::White, Color::Black);
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
    let mut vga: VgaCon<25, 80, 1> = VgaCon::new(1u8, 0, Color::White, Color::Black);
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
fn test_scroll1()
{
    let mut vga: VgaCon<25, 40, 3> = VgaCon::new(1u8, 20, Color::White, Color::Black);
    for _i in 0..24 {
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
fn test_scroll2() {}

#[test_case]
fn test_indicator1() {}
