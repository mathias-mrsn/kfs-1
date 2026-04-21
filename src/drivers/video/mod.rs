use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use vgac::VgaConsole;

mod crtc;
mod gfxc;
mod vgac;

lazy_static! {
        static ref LOGGER: Mutex<VgaConsole> = Mutex::new(VgaConsole::new(
                vgac::VGAColor::White,
                vgac::VGAColor::Black,
                vgac::Resolution::R80_25,
                vgac::MemoryRanges::Small,
                Some(vgac::CursorTypes::Full),
        ));
}

#[doc(hidden)]
pub(crate) fn _print(args: fmt::Arguments)
{
        let mut logger = LOGGER.lock();
        fmt::write(&mut *logger, args).ok();
}

pub(crate) fn _panic_print(args: fmt::Arguments)
{
        if let Some(mut logger) = LOGGER.try_lock() {
                fmt::write(&mut *logger, args).ok();
        }
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
