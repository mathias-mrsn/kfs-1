#![feature(maybe_uninit_uninit_array)]
#![feature(naked_functions)]
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]
#![allow(dead_code)]
#![no_std]
#![no_main]

mod drivers;
mod kernel;
mod multiboot;
mod panic;

use core::arch::naked_asm;
use core::mem::MaybeUninit;

use crate::multiboot::{MULTIBOOT_HEADER_MAGIC, MultibootHeader, MultibootHeaderFlags};

const STACK_SIZE: usize = 0x10000;

#[used]
#[unsafe(link_section = ".multiboot")]
pub static MULTIBOOT_HEADER: MultibootHeader = MultibootHeader {
    magic:         MULTIBOOT_HEADER_MAGIC,
    flags:         MultibootHeaderFlags::ALIGN_MODULES.bits()
        | MultibootHeaderFlags::MEMORY_INFO.bits(),
    checksum:      MULTIBOOT_HEADER_MAGIC
        .wrapping_add(
            MultibootHeaderFlags::ALIGN_MODULES.bits() | MultibootHeaderFlags::MEMORY_INFO.bits(),
        )
        .wrapping_neg(),
    header_addr:   0,
    load_addr:     0,
    load_end_addr: 0,
    bss_end_addr:  0,
    entry_addr:    0,
    mode_type:     0,
    width:         0,
    height:        0,
    depth:         0,
};

#[used]
#[unsafe(link_section = ".bss")]
static mut STACK: [MaybeUninit<u8>; STACK_SIZE] = MaybeUninit::uninit_array();

#[naked]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".boot")]
pub extern "C" fn _start()
{
    unsafe {
        naked_asm!(
            "mov esp, offset {stack} + {stack_size}",
            "
            push ebx
            call {kernel_main}
            ",
            "
            cli
            2:
            hlt
            jmp 2b
            ",
            stack = sym STACK,
            stack_size = const STACK_SIZE,
            kernel_main = sym kernel::kernel_main
        )
    }
}
