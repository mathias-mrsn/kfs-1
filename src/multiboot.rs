use bitflags::bitflags;

/*
 * QEMU only supports Multiboot 1.
 * Using Multiboot 2 requires creating an image for each compilation.
 * On M1 systems, this involves compiling the kernel in a Docker image, which
 * significantly increases development time. For this reason, I continue to
 * use Multiboot 1.
 */
pub const MULTIBOOT_HEADER_MAGIC: u32 = 0x1BADB002;
pub const BOOTLOADER_MAGIC: u32 = 0x2BADB002;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MultibootHeader
{
    pub magic:         u32,
    pub flags:         u32,
    pub checksum:      u32,
    pub header_addr:   u32, //	if flags[16] is set
    pub load_addr:     u32, //	if flags[16] is set
    pub load_end_addr: u32, //	if flags[16] is set
    pub bss_end_addr:  u32, //	if flags[16] is set
    pub entry_addr:    u32, //	if flags[16] is set
    pub mode_type:     u32, //	if flags[2] is set
    pub width:         u32, //	if flags[2] is set
    pub height:        u32, //	if flags[2] is set
    pub depth:         u32, //	if flags[2] is set
}

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy)]
    pub struct MultibootHeaderFlags: u32 {
        const ALIGN_MODULES = 1 << 0;
        const MEMORY_INFO = 1 << 1;
        const VIDEO_MODE = 1 << 2;
        const LOAD_ADDRESS = 1 << 16;
    }
}
