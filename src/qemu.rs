const EXIT_IO_PORT: u16 = 0xf4;

use crate::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode
{
    Success = 0x10,
    Failed  = 0x11,
}

pub fn exit(code: QemuExitCode)
{
    unsafe {
        io::outdw(EXIT_IO_PORT, code as u32);
    }
}
