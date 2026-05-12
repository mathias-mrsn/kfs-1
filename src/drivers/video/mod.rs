use core::fmt;
use spin::{Mutex, Once};
use vgac::VgaConsole;

use crate::instructions::cpu;

mod crtc;
mod gfxc;
mod vgac;

static LOGGER: Once<Mutex<VgaConsole>> = Once::new();

fn build_logger() -> Mutex<VgaConsole>
{
    Mutex::new(VgaConsole::new(
        vgac::VGAColor::White,
        vgac::VGAColor::Black,
        vgac::VGAColor::Black,
        vgac::VGAColor::Green,
        vgac::Resolution::R80_25,
        vgac::MemoryRanges::Small,
        vgac::CursorType::Full,
    ))
}

pub(crate) fn initialize() { let _ = LOGGER.call_once(build_logger); }

fn logger() -> &'static Mutex<VgaConsole> { LOGGER.call_once(build_logger) }

fn with_logger<R>(f: impl FnOnce(&mut VgaConsole) -> R) -> R
{
    cpu::without_interrupts(|| {
        let mut logger = logger().lock();
        f(&mut logger)
    })
}

#[doc(hidden)]
pub(crate) fn _print(args: fmt::Arguments)
{
    with_logger(|logger| {
        fmt::write(logger, args).ok();
    });
}

pub(crate) fn _panic_print(args: fmt::Arguments)
{
    cpu::without_interrupts(|| {
        if let Some(mut logger) = LOGGER.get().and_then(|logger| logger.try_lock()) {
            fmt::write(&mut *logger, args).ok();
        }
    });
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
