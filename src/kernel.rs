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

mod commun;
mod controllers;
mod cpu;
mod drivers;
mod instructions;
mod memory;
mod multiboot;
mod panic;
mod qemu;
mod registers;
mod sync;
mod test;
mod utils;

use core::arch::asm;
use core::arch::global_asm;
use core::arch::naked_asm;
use core::mem;
use core::mem::MaybeUninit;
use core::slice;

use crate::commun::{ConstDefault, ConstFrom, ConstInto};

use drivers::video::LOGGER;
use memory::addr::PhysAddr;
use memory::paging::pdt::PDE;
use memory::paging::pdt::PDEFlags;
use memory::paging::pdt::PDT;
use multiboot::MultibootInfo;
use multiboot::{MultibootMmapEntry, MultibootMmapEntryType};
use registers::RegisterAccessor;
use registers::cr0::CR0Flags;

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

#[unsafe(no_mangle)]
#[unsafe(link_section = ".boot.pdt")]
static PDT: PDT = const {
    let mut table = PDT::default_const();
    let mut i: usize = 0;
    while i < 256 {
        table.user_space[i as usize] = PDE::new(
            PhysAddr::from_const(0x1000 * i),
            PDEFlags::PAGE_SIZE
                .union(PDEFlags::READ_WRITE)
                .union(PDEFlags::PRESENT),
        );
        i += 1;
    }
    i = 0;
    while i < 1 {
        table.kernel_space[i as usize] = PDE::new(
            PhysAddr::from_const(0x1000 * i),
            PDEFlags::PAGE_SIZE
                .union(PDEFlags::READ_WRITE)
                .union(PDEFlags::PRESENT),
        );
        i += 1;
    }
    table
};

unsafe extern "C" {
    fn _start();
}

global_asm!(
r#"
.section .boot.text, "ax"
.global _start
_start:
    mov esp, offset {stack} + {stack_size} - 0xc0000000

    // Push multiboot informations
    push ebx
    push eax


    // Set cr3 to the address of the page table.
    mov eax, offset {PDT}
    mov cr3, eax

    // Enable PSE (unknown why yet)
    mov eax, cr4
    or eax, 0x00000010
    mov cr4, eax

    // Enable pagging
    mov eax, cr0
    or eax, 0x80010000
    mov cr0, eax

    // Convert stack address from physical to virtual
    add esp, 0xc0000000

    call {kernel_main}
"#,
    stack = sym STACK,
    stack_size = const STACK_SIZE,
    kernel_main = sym kernel_main,
    PDT = sym PDT,
);

// use crate::drivers::video;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(
    multiboot_magic: u32,
    mbi: &'static MultibootInfo,
) -> !
{
    if multiboot_magic != multiboot::BOOTLOADER_MAGIC {
        panic!("invalid magic number at ")
    }

    lazy_static::initialize(&cpu::GDT);
    // let _t = crate::cpu::apic::initialize();
    lazy_static::initialize(&cpu::IDT);

    // Initialize memory subsystems
    let mmap = crate::memory::mmap::initialize(mbi);
    //crate::memory::_kmem::initialize(mbi);

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

    // unsafe {
    //     asm!("int 0x21");
    // }

    // let cpuid;
    // unsafe {
    //     cpuid = core::arch::x86::__cpuid(1);
    // }

    //println!(
    //    "cr3 pg addr -> {:?}",
    //    crate::registers::cr3::CR3::read_pdt()
    //);

    // unsafe {
    //     crate::registers::cr0::CR0::write(CR0Flags::PG);
    // }
    loop {
        // unsafe {
        //     asm!("hlt");
        // }
    }
}
