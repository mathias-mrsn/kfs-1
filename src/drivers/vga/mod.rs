use lazy_static::lazy_static;
use core::fmt;
use spin::Mutex;

#[allow(dead_code)] // Remove warning about unused code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

lazy_static! {
    pub static ref VGADRIVER: Mutex<VGA> = Mutex::new(VGA::new());
}

pub const VGA_WIDTH: usize = 80;
pub const VGA_HEIGHT: usize = 25;
pub const VGA_PADDR: *mut u16 = 0xb8000 as _;

#[derive(Clone, Copy)]
#[repr(transparent)]
struct VGAChar(u16);

impl VGAChar {
    #[inline]
    fn new(c: u8, fg: u8, bg: u8) -> VGAChar {
        VGAChar(((c as u16) | ((fg | (bg << 4)) as u16) << 8) as _)
    }

    #[inline]
    fn get_vgac(c: VGAChar) -> (u8, u8, u8) {
        (
            (c.0 & 0xff) as u8,
            ((c.0 >> 8) & 0xf) as u8,
            ((c.0 >> 12) & 0xf) as u8,
        ) as _
    }
}

pub struct VGA {
    c_index: usize,
    buffer: [VGAChar; VGA_WIDTH * VGA_HEIGHT],
}

impl VGA {
    pub fn new() -> VGA {
        VGA {
            c_index: 0,
            buffer: [VGAChar::new(b' ', Color::White as u8, Color::Black as u8);
                VGA_WIDTH * VGA_HEIGHT],
        }
    }

    pub fn putchar(&mut self, c: char) {
        match c {
            '\n' => self.c_index += (((self.c_index / VGA_WIDTH) + 1) * VGA_WIDTH) - self.c_index,
            c => {
                let vga_character: VGAChar =
                    VGAChar::new(c as u8, Color::White as u8, Color::Black as u8);
                unsafe {
                    self.buffer[self.c_index as usize] = vga_character;
                    *VGA_PADDR.offset((self.c_index) as isize) = vga_character.0;
                }
                self.c_index += 1;
            }
        }
        if self.c_index == VGA_HEIGHT * VGA_WIDTH {
            self.c_index -= VGA_WIDTH;
            unsafe {
                self.scrolldown(1);
            }
        }
    }

    pub unsafe fn scrolldown(&mut self, i: u32) {
        let mut y = 0;

        assert!(
            (i as usize) < VGA_HEIGHT,
            "scrolldown(): parameter must be lower than {}",
            VGA_HEIGHT
        );
        for j in (i as usize * VGA_WIDTH)..(VGA_WIDTH * VGA_HEIGHT) {
            self.buffer[y] = self.buffer[j];
            y += 1;
        }
        self.refresh();
    }

    pub fn refresh(&self) {
        for i in 0..(VGA_HEIGHT * VGA_WIDTH) {
            unsafe {
                *VGA_PADDR.offset(i as isize) = self.buffer[i].0;
            }
        }
    }

    pub fn putstr(&mut self, s: &str) {
        for c in s.bytes() {
            self.putchar(c as char);
        }
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::vga::_print(format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::_print!("\n")
    };
    ($($arg:tt)*) => {{
        $crate::print!("{}\n", format_args!($($arg)*));
    }};
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    if let Some(s) = args.as_str() {
        VGADRIVER.lock().putstr(s);
    }
}

