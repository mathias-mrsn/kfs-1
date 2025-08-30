#![no_std]
#![no_main]
#![feature(maybe_uninit_uninit_array)]
#![allow(unsafe_op_in_unsafe_fn)]
#![feature(naked_functions)]
#![feature(generic_const_exprs)]
#![feature(const_trait_impl)]
#![allow(incomplete_features)]
#![feature(format_args_nl)]
#![allow(dead_code)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "kernel_maintest"]
#![feature(abi_x86_interrupt)]

mod controllers;
mod drivers;
mod instructions;
mod multiboot;
mod panic;
mod qemu;
mod test;

use core::arch::global_asm;
use core::mem::MaybeUninit;

use multiboot::MultibootInfo;

use crate::drivers::video::vgac;
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
                        MultibootHeaderFlags::ALIGN_MODULES.bits()
                                | MultibootHeaderFlags::MEMORY_INFO.bits(),
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

unsafe extern "C" {
        fn _start();
}

global_asm!(
r#"
.section .boot.text, "ax"
.global _start
_start:
    mov esp, offset {stack} + {stack_size}

    // Push multiboot informations
    push ebx
    push eax

    call {kernel_main}

    cli
    2:
    hlt
    jmp 2b
"#,
    stack = sym STACK,
    stack_size = const STACK_SIZE,
    kernel_main = sym kernel_main,
);

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(
        multiboot_magic: u32,
        _mbi: &'static MultibootInfo,
) -> !
{
        if multiboot_magic != multiboot::BOOTLOADER_MAGIC {
                panic!("invalid magic number at ")
        }

        #[cfg(test)]
        kernel_maintest();

        println!("sizeof");

        loop {}
}
