use crate::drivers::video;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
        video::_panic_print(format_args_nl!("Fatal Error: {}", info.message()));
        video::_panic_print(format_args_nl!("Location: {:?}", info.location()));
        loop {}
}
