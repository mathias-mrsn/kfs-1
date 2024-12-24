use core::cmp;
// use core::fmt;
use core::ptr;

pub const BLANK: u16 = 0x0020;
pub const VGA_INDEX_MARK: u16 = 0x0530;
pub const VGACON_COLS: usize = 80;
pub const VGACON_ROWS: usize = 25;

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
pub struct VgaCon<const ROWS: usize, const COLS: usize, const NB_AREAS: usize>
where
    [(); ROWS * COLS * NB_AREAS]:,
{
    pub vc_num:              u8,
    pub vc_lock:             bool,
    pub vc_index:            usize,
    pub vc_screen_size:      usize,
    pub vc_vram_base:        *mut u16,
    pub vc_foreground_color: Color,
    pub vc_background_color: Color,
    pub vc_screenbuf:        [u16; ROWS * COLS * NB_AREAS],
    pub vc_screenbuf_size:   usize,
    pub vc_visible_origin:   usize,
    pub vc_rows:             usize,
    pub vc_cols:             usize,
}

impl<const ROWS: usize, const COLS: usize, const NB_AREAS: usize> VgaCon<ROWS, COLS, NB_AREAS>
where
    [(); ROWS * COLS * NB_AREAS]:,
{
    pub fn new(
        id: u8,
        vram_base: *mut u16,
        foreground_color: Color,
        background_color: Color,
    ) -> Self
    {
        debug_assert!(id < 10, "VGA Index must be lower than 10");

        Self {
            vc_num:              id,
            vc_lock:             false,
            vc_index:            0,
            vc_screen_size:      ROWS * COLS,
            vc_vram_base:        vram_base,
            vc_foreground_color: foreground_color,
            vc_background_color: background_color,
            vc_screenbuf:        [BLANK; ROWS * COLS * NB_AREAS],
            vc_screenbuf_size:   ROWS * COLS * NB_AREAS,
            vc_visible_origin:   0,
            vc_rows:             ROWS,
            vc_cols:             COLS,
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
            let offset = ((self.vc_rows - 1) * VGACON_COLS + self.vc_index) as isize;
            *self.vc_vram_base.offset(offset) = byte;
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
                // printable ASCII byte or newline
                b'\n' => self.scroll(ScrollDir::ScDown, Some(1)),
                0x20..=0x7e => self.cputc(byte, foreground, background),
                // not part of printable ASCII range
                _ => self.cputc(0xfe, None, None),
            };
        }
    }

    // pub fn cursor(
    //     &mut self,
    //     enable: bool,
    // )
    // {
    // }

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
                ptr::write_bytes(self.vc_vram_base, 0x00, self.vc_screen_size);
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
                    *self
                        .vc_vram_base
                        .offset((((self.vc_rows - rows - 1) * VGACON_COLS) + cols) as isize) = self
                        .vc_screenbuf
                        [(rows * self.vc_cols) + cols + self.vc_visible_origin as usize];
                }
            }
        }

        unsafe {
            *self.vc_vram_base.offset((self.vc_cols - 1) as _) =
                VGA_INDEX_MARK + self.vc_num as u16;
        }

        if self.vc_visible_origin != 0 {
            unsafe {
                *self.vc_vram_base.offset((self.vc_cols - 2) as _) = VgaConIndicators::Visual as _;
            }
        }
    }
}

// impl<const ROWS: usize, const COLS: usize, const NB_AREAS: usize> fmt::Write
//     for VgaCon<ROWS, COLS, NB_AREAS>
// where
//     [(); ROWS * COLS * NB_AREAS]:,
// {
//     fn write_str(
//         &mut self,
//         s: &str,
//     ) -> fmt::Result
//     {
//         self.putstr(s);
//         Ok(())
//     }
//
//     fn write_char(
//         &mut self,
//         c: char,
//     ) -> fmt::Result
//     {
//         self.putc(c as u8);
//         Ok(())
//     }
// }
//
// pub static VGABUFFER: VgaCon<25, 80, 3> =
//     VgaCon::new(1u8, 0xb8000 as _, Color::White, Color::Black);
//
// /*
//  * From: std/macros.rs
//  */
// #[macro_export]
// macro_rules! print {
//     ($($arg:tt)*) => {{
//         $crate::drivers::video::vgacon::_print($crate::format_args!($($arg)*
// ));     }};
// }
//
// #[macro_export]
// macro_rules! println {
//     () => {
//         $crate::print!("\n")
//     };
//     ($($arg:tt)*) => {{
//         $crate::drivers::video::vgacon::_print($crate::format_args_nl!
// ($($arg)*));     }};
// }
//
// #[doc(hidden)]
// pub fn _print(args: fmt::Arguments)
// {
//     WRITER.lock().write_fmt(args).unwrap();
// }
