/*
 * https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html
 */

#![no_std]
#![no_main]
#![feature(maybe_uninit_uninit_array)]
#![allow(unsafe_op_in_unsafe_fn)]
#![feature(naked_functions)]
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]
#![feature(format_args_nl)]
#![allow(dead_code)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "kernel_maintest"]
#![feature(abi_x86_interrupt)]

mod controllers;
mod cpu;
mod drivers;
mod instructions;
mod multiboot;
mod panic;
mod qemu;
mod test;
mod utils;

use core::arch::naked_asm;
use core::mem::MaybeUninit;

use drivers::video::LOGGER;

use crate::cpu::gdt;
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
            push eax
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
            kernel_main = sym kernel_main
        )
    }
}

// use crate::drivers::video;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(multiboot_magic: u32) -> !
{
    if multiboot_magic != multiboot::BOOTLOADER_MAGIC {
        panic!("hi")
    }

    lazy_static::initialize(&cpu::GDT);
    lazy_static::initialize(&cpu::IDT);
    let t = crate::cpu::apic::initialize();

    #[cfg(test)]
    kernel_maintest();

    // LOGGER.lock().blank();
    // println!("{}", include_str!(".assets/header.txt"));

    // let rsdp = apic::rsdp::search_on_bios();
    // match rsdp {
    //     Some(rsdp) => {
    //         let rsdt = unsafe { &*(rsdp.get_rsdt()) };
    //         let s = unsafe { rsdt.find_sdt(Signature::MADT) };
    //         writeln!(vga, "RSDP found: {:?}", &s)
    //     }
    //     None => writeln!(vga, "RSDP not found"),
    // };

    // for i in 0..50 {
    //     writeln!(vga, "{}", i).unwrap();
    // }

    // let i = 0;
    //
    // writeln!(vga, "{:#x}", ptr::addr_of!(i) as usize).unwrap();
    // writeln!(vga, "{:#x}", &i as *const _ as usize).unwrap();

    let cpuid;
    unsafe {
        cpuid = core::arch::x86::__cpuid(1);
    }

    loop {}
}
