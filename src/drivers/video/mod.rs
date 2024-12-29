pub mod vgacon;

use core::fmt::{self, Write};
use spin::Mutex;

pub static LOGGER: Mutex<vgacon::VgaCon<25, 80, 3>> = Mutex::new(vgacon::VgaCon::new(
    2u8,
    0,
    vgacon::Color::White,
    vgacon::Color::Pink,
));

#[doc(hidden)]
pub fn _print(args: fmt::Arguments)
{
    LOGGER.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
	($($arg:tt)*) => {{
		$crate::drivers::video::_print(format_args!($($arg)*));
	}};
}

#[macro_export]
macro_rules! println {
	() => ($crate::print!("\n"));
	($($arg:tt)*) => {{
		$crate::drivers::video::_print(format_args_nl!($($arg)*));
	}};
}
