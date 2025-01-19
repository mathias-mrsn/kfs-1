use crate::vgac;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    use core::fmt::Write;
    let mut vga: vgac::VgaConsole = vgac::VgaConsole::new(
        vgac::VGAColor::White,
        vgac::VGAColor::Black,
        vgac::Resolution::R120_50,
        vgac::MemoryRanges::Small,
        Some(vgac::CursorTypes::Full),
    );

    writeln!(vga, "Fatal Error: {}", info.message()).unwrap();
    loop {}
}
