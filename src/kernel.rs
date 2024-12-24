use crate::drivers::video::vgacon;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> !
{
    let mut vga: vgacon::VgaCon<9, 80, 2> =
        vgacon::VgaCon::new(1u8, 0xb8000 as _, vgacon::Color::Pink, vgacon::Color::Black);
    vga.putstr("/* ************************************************************************** */");
    vga.putstr("/*                                                        :::      ::::::::   */");
    vga.putstr("/*                                                      :+:      :+:    :+:   */");
    vga.putstr("/*                                                    +:+ +:+         +:+     */");
    vga.putstr("/*                                                  +#+  +:+       +#+        */");
    vga.putstr("/*                                                +#+#+#+#+#+   +#+           */");
    vga.putstr("/*                                                     #+#    #+#             */");
    vga.putstr("/*                                                    ###   ########.fr       */");
    vga.putstr("/* ************************************************************************** */");

    let mut vga2: vgacon::VgaCon<17, 80, 2> = vgacon::VgaCon::new(
        2u8,
        (0xb8000 + (80 * 2 * 9)) as _,
        vgacon::Color::White,
        vgacon::Color::Pink,
    );
    vga2.putstr("Hello\n");
    vga2.putstr("Hello\n");

    loop {}
}
