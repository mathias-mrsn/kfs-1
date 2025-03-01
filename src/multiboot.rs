use core::cmp::Ordering;
use core::fmt::Debug;

use bitflags::bitflags;

use crate::memory::addr::PhysAddr;

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

#[repr(C)]
pub struct MultibootInfo
{
    flags:            u32,
    pub mem_lower:    u32,
    pub mem_upper:    u32,
    boot_device:      u32,
    cmdline:          u32,
    mods_count:       u32,
    mods_addr:        u32,
    symbols_1:        u32,
    symbols_2:        u32,
    symbols_3:        u32,
    symbols_4:        u32,
    pub mmap_length:  u32,
    pub mmap_addr:    PhysAddr,
    drives_length:    u32,
    drives_addr:      u32,
    _config_table:    u32,
    boot_loader_name: u32,
}

#[repr(u32)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum MultibootMmapEntryType
{
    Available = 1,
    Reserved,
    AcpiReclamable,
    Nvs,
    Badrram,
}

#[repr(C)]
#[derive(Eq, PartialEq, Clone, Copy)]
pub struct MultibootMmapEntry
{
    pub size:       u32,
    pub addr:       u64,
    pub len:        u64,
    pub entry_type: MultibootMmapEntryType,
}

impl PartialOrd for MultibootMmapEntry
{
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering>
    {
        Some(self.len.cmp(&other.len))
    }
}

impl Ord for MultibootMmapEntry
{
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering
    {
        self.len.cmp(&other.len)
    }
}

impl Debug for MultibootMmapEntry
{
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result
    {
        f.debug_struct("MultibootMmapEntry")
            .field("size", &self.size)
            .field("addr", &format_args!("{:#x}", self.addr))
            .field("len", &format_args!("{:#x}", self.len))
            .field("entry_type", &self.entry_type)
            .finish()
    }
}

impl Debug for MultibootInfo
{
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result
    {
        f.debug_struct("MultibootInfo")
            .field("flags", &format_args!("{:#b}", self.flags))
            .field(
                "mem_lower",
                &format_args!("{:#x} Bytes", self.mem_lower * 1000),
            )
            .field(
                "mem_upper",
                &format_args!("{:#x} Bytes", self.mem_upper * 1000),
            )
            .field("mmap_length", &format_args!("{}", self.mmap_length))
            .field("mmap_addr", &format_args!("{}", self.mmap_addr))
            .finish()
    }
}
