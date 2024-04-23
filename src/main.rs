#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

mod drivers;
use crate::drivers::vga;
// use self::vga::putchar;

use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn kernel_main() -> ! {
    let mut b: vga::VGA = vga::VGA::new();

    b.refresh();

    // b.clean();
    b.putstr("/* ************************************************************************** */");
    b.putstr("/*                                                                            */");
    b.putstr("/*                                                        :::      ::::::::   */");
    b.putstr("/*                                                      :+:      :+:    :+:   */");
    b.putstr("/*                                                    +:+ +:+         +:+     */");
    b.putstr("/*                                                  +#+  +:+       +#+        */");
    b.putstr("/*                                                +#+#+#+#+#+   +#+           */");
    b.putstr("/*                                                     #+#    #+#             */");
    b.putstr("/*                                                    ###   ########.fr       */");
    b.putstr("/*                                                                            */");
    b.putstr("/* ************************************************************************** */");

    loop {}
}
