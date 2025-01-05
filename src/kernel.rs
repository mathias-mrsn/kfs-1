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

mod drivers;
mod io;
mod multiboot;
mod panic;
mod qemu;
mod test;
mod utils;

use core::arch::naked_asm;
use core::mem::MaybeUninit;

use drivers::video::vgac;
use drivers::video::vgac::crtc;
use drivers::video::vgacon;

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

    #[cfg(test)]
    kernel_maintest();

    // let width = 40;
    // vgacon::ctrc_write(vgacon::CTRCRegistersIndexes::VgaCrtcHDisp as u8, width -
    // 1); vgacon::ctrc_write(vgacon::CTRCRegistersIndexes::VgaCrtcVDispEnd as
    // u8, 0xff); vgacon::ctrc_write(
    //     vgacon::CTRCRegistersIndexes::VgaCrtcOffset as u8,
    //     width >> 1,
    // );

    fn resize(
        height: u8,
        width: u8,
    )
    {
        let mut scanlines: u32 = height as u32 * 16;
        let max_scan: u8 = vgacon::ctrc_read(vgacon::CTRCRegistersIndexes::VgaCrtcMaxScan as u8);
        if (max_scan & 0x80) != 0 {
            scanlines <<= 1;
        }
        let mode = vgacon::ctrc_read(vgacon::CTRCRegistersIndexes::VgaCrtcMode as u8);
        if (mode & 0x04) != 0 {
            scanlines >>= 1;
        }
        scanlines -= 1;
        let scanlines_lo = scanlines & 0xff;

        let mut r7 = vgacon::ctrc_read(vgacon::CTRCRegistersIndexes::VgaCrtcOverflow as u8) & !0x42;
        if (scanlines & 0x100) != 0 {
            r7 |= 0x02;
        }
        if (scanlines & 0x200) != 0 {
            r7 |= 0x40;
        }
        let vsync_end = vgacon::ctrc_read(vgacon::CTRCRegistersIndexes::VgaCrtcVSyncEnd as u8);

        vgacon::ctrc_write(
            vgacon::CTRCRegistersIndexes::VgaCrtcVSyncEnd as u8,
            vsync_end & !0x80,
        );

        /* Reduire the size of the window */
        vgacon::ctrc_write(vgacon::CTRCRegistersIndexes::VgaCrtcHDisp as u8, width - 1);
        // vgacon::ctrc_write(vgacon::CTRCRegistersIndexes::VgaCrtcVDispEnd
        // // as u8, 0xff);
        vgacon::ctrc_write(
            vgacon::CTRCRegistersIndexes::VgaCrtcOffset as u8,
            width >> 1,
        );
        vgacon::ctrc_write(
            vgacon::CTRCRegistersIndexes::VgaCrtcVSyncEnd as u8,
            scanlines_lo as u8,
        );
        vgacon::ctrc_write(vgacon::CTRCRegistersIndexes::VgaCrtcOverflow as u8, r7);
        vgacon::ctrc_write(
            vgacon::CTRCRegistersIndexes::VgaCrtcVSyncEnd as u8,
            vsync_end,
        );
    }

    // resize(15, 60);
    //
    // let mut out;
    // unsafe {
    //     io::outb(0x3CE, 6);
    //     io::outb(0x3CF, 2);
    //     io::outb(0x3CE, 6);
    //     out = io::inb(0x3CF);
    // }
    // print!("--> {:08b}", out);
    // // for i in 0..(15 * 40) {
    // //     unsafe {
    // //         *vgacon::VGA_VRAM_BASE.offset(i) = 0x0f3e;
    // //     }
    // // }
    // print!(
    //     "horizontal total: {}",
    //     vgacon::ctrc_read(vgacon::CTRCRegistersIndexes::VgaCrtcHTotal as u8),
    // );
    // print!(
    //     "end horizontal display: {}",
    //     vgacon::ctrc_read(vgacon::CTRCRegistersIndexes::VgaCrtcHDisp as u8),
    // );
    // print!(
    //     "start horizontal blanking: {}",
    //     vgacon::ctrc_read(vgacon::CTRCRegistersIndexes::VgaCrtcHBlankStart as
    // u8), );
    // print!(
    //     "end horizontal blanking: {:08b}",
    //     vgacon::ctrc_read(vgacon::CTRCRegistersIndexes::VgaCrtcHBlankEnd as u8),
    // );
    // print!(
    //     "start horizontal retrace: {}",
    //     vgacon::ctrc_read(0x04 as u8),
    // );
    // print!(
    //     "end horizontal retrace: {:08b}",
    //     vgacon::ctrc_read(0x05 as u8),
    // );
    // print!("vertical total register: {}", vgacon::ctrc_read(0x06 as u8),);
    // print!("overflow: {:08b}", vgacon::ctrc_read(0x07 as u8),);
    // print!("preset row scan: {:08b}", vgacon::ctrc_read(0x08 as u8),);
    // print!("max scan line: {:08b}", vgacon::ctrc_read(0x09 as u8),);
    // print!("cursor start: {:08b}", vgacon::ctrc_read(0x0a as u8),);
    // print!("cursor end: {:08b}", vgacon::ctrc_read(0x0b as u8),);
    // print!("addr high register: {:08b}", vgacon::ctrc_read(0x0c as u8),);
    // print!("addr low register: {:08b}", vgacon::ctrc_read(0x0d as u8),);
    // print!("offset: {}", vgacon::ctrc_read(0x13 as u8),);
    //
    // vgacon::ctrc_write(vgacon::CTRCRegistersIndexes::VgaCrtcStartLo as u8, 60);

    use core::fmt::Write;
    let mut vga: vgac::VgaConsole = vgac::VgaConsole::new(
        vgac::VGAColor::White,
        vgac::VGAColor::Black,
        25,
        80,
        vgac::MemoryRanges::Small,
        Some(vgac::CursorTypes::Full),
    );

    write!(vga, "Hello {}", 2).unwrap();
    write!(vga, "Hello {:08b}", crtc::read(crtc::Indexes::MaxScan)).unwrap();

    // for _i in 0..1 {
    //     vga.putstr(
    //         "\n\
    // nHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHello\
    // n",     );
    // }

    vga.resize(50, 100);
    vga.resize(5, 10);

    // for _j in 0..255 {
    //     vga.putstr("Hello");
    // }

    // vga.blank();

    // vga.scroll(vgac::ScrollDir::Down, Some(3));
    // vgac::crtc::write(vgac::crtc::Indexes::StartLo, 80 as u8);
    // vgac::crtc::write(vgac::crtc::Indexes::StartHi, (80 << 8) as u8);

    // unsafe {
    //     print!("salut {}", io::inb(0x3c7));
    // }

    loop {}
}
