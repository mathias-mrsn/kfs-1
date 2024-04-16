#[allow(dead_code)] // Remove warning about unused code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]

// TODO: Remove few type conversion.

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

pub type VGAChar = u16;

/// VGA screen width
pub const VGA_WIDTH: u8 = 80;
/// VGA screen heigth
pub const VGA_HEIGHT: u8 = 25;
// VGA buffer physical address.
pub const VGA_PADDR: *mut u8 = 0xb8000 as *mut u8;

#[inline]
pub fn createVGAC(c: u8, fg: u8, bg: u8) -> VGAChar {
    (c as VGAChar) << 8 | (fg | bg << 4) as VGAChar
}

#[inline]
pub fn getVGAC(c: VGAChar) -> (u8, u8, u8) {
    ((c >> 8) as u8, (c & 0xf) as u8, ((c >> 4) & 0xf) as u8) as _
}

// pub static VGABUFFER: VGA = VGA::new();

/// Structure used to store information about the current state
/// of the VGA buffer (rows, columns, index...).
pub struct VGA {
    pub c_rows: u8,
    pub c_columns: u8,
}

impl VGA {
    pub fn new() -> VGA {
        VGA {
            c_rows: 0,
            c_columns: 0,
        }
    }

    pub fn clean(&mut self) {
        for i in 0..(VGA_WIDTH as u16 * VGA_HEIGHT as u16) {
            unsafe {
                self.putchar(createVGAC(
                    ' ' as u8,
                    Color::Black as u8,
                    Color::Black as u8,
                ));
            }
        }
        self.c_rows = 0;
        self.c_columns = 0;
    }

    pub unsafe fn putchar(&mut self, c: VGAChar) {
        *VGA_PADDR.offset(
            (self.c_rows as isize * VGA_WIDTH as isize * 2) + (self.c_columns as isize * 2),
        ) = (c >> 8 & 0x00ff) as u8;
        *VGA_PADDR.offset(
            (self.c_rows as isize * VGA_WIDTH as isize * 2) + (self.c_columns as isize * 2) + 1,
        ) = (c & 0x00ff) as u8;

        self.c_columns = (self.c_columns + 1) % VGA_WIDTH;
        if self.c_columns == 0 {
            self.c_rows = (self.c_rows + 1) % VGA_HEIGHT;
        }
    }

    pub fn putstr(&mut self, s: &str) {
        for c in s.chars() {
            unsafe {
                self.putchar(createVGAC(c as u8, Color::White as u8, Color::Black as u8));
            }
        }
    }
}
