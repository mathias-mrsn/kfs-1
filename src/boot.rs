use bitflags::bitflags;

use core::mem::MaybeUninit;

pub const MULTIBOOT_HEADER_MAGIC: u32 = 0x1BADB002;

pub const STACK_SIZE: usize = 0x10000;

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy)]
    pub struct Flags: u32 {
    const ALIGN_MODULES = (1 << 0);
    const MEMORY_INFO = (1 << 1);
    const VIDEO_MODE = (1 << 2);
    const LOAD_ADDRESS = (1 << 16);
    }
}

#[repr(C)]
pub struct MultibootHeader {
    magic: u32,
    flags: Flags,
    checksum: u32,
    header_addr: u32,   //	if flags[16] is set
    load_addr: u32,     //	if flags[16] is set
    load_end_addr: u32, //	if flags[16] is set
    bss_end_addr: u32,  //	if flags[16] is set
    entry_addr: u32,    //	if flags[16] is set
    mode_type: u32,     //	if flags[2] is set
    width: u32,         //	if flags[2] is set
    height: u32,        //	if flags[2] is set
    depth: u32,         //	if flags[2] is set
}

#[used]
#[unsafe(link_section = ".multiboot")]
pub static MULTIBOOT_HEADER: MultibootHeader = MultibootHeader {
    magic: MULTIBOOT_HEADER_MAGIC,
    flags: Flags::ALIGN_MODULES,
    checksum: 0,
    header_addr: 0,
    load_addr: 0,
    load_end_addr: 0,
    bss_end_addr: 0,
    entry_addr: 0,
    mode_type: 0,
    width: 0,
    height: 0,
    depth: 0,
};

// #[used]
// #[unsafe(link_section = ".bss")]
// static mut STACK: [MaybeUninit<u8>; STACK_SIZE] = MaybeUninit::uninit_array();
