pub mod gdt;

pub static mut GDT: *mut gdt::GlobalDescriptorTable<8> =
    0x800 as *mut gdt::GlobalDescriptorTable<8>;
