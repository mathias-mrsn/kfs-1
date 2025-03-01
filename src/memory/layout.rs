use crate::{
    commun::{ConstFrom, ConstInto},
    memory::PhysAddr,
    println,
};

pub const KERNEL_CODE_PA: PhysAddr = PhysAddr::from_const(0x10_0000);
pub const KERNEL_CODE_COMMUN_SIZE: usize = 0x400000 - KERNEL_CODE_PA.into_const();

unsafe extern "C" {
    static kernel_size: usize;
    #[link_name = "kernel_start"]
    static kernel_start: u8;
    #[link_name = "kernel_end"]
    static kernel_end: u8;
}
