use lazy_static::lazy_static;

use core::cmp::{max, min};

use crate::mem;
use crate::multiboot::{MultibootInfo, MultibootMmapEntry, MultibootMmapEntryType};
use crate::slice;
use crate::sync::oncelock::OnceLock;
use crate::{commun::ConstFrom, memory::PhysAddr, println};

use super::paging::pdt::PAGES_TABLES_SIZE;

/// Kernel Code Physical Address
pub const KERNEL_CODE_PHYS: PhysAddr = PhysAddr::from_const(0x10_0000);
pub const KERNEL_SPACE_END: PhysAddr = PhysAddr::from_const(0x4000_0000);

pub static MMAP: OnceLock<MemoryMap> = OnceLock::new();
pub const PAGE_SIZE: usize = 4096;

unsafe extern "C" {
    static kernel_size: usize;
    #[link_name = "kernel_start"]
    static kernel_start: u8;
    #[link_name = "kernel_end"]
    static kernel_end: u8;
}

pub struct MemoryMap
{
    kernel_space: PhysAddr,
}

pub const KERNEL_SPACE_MAX_END: PhysAddr = PhysAddr::from_const(0x40000000);

/// Initialize the memory map from multiboot information
pub fn initialize(mbi: &'static MultibootInfo)
{
    unsafe {
        let entries_slice = slice::from_raw_parts::<MultibootMmapEntry>(
            mbi.mmap_addr.as_ptr::<MultibootMmapEntry>(),
            mbi.mmap_length as usize / mem::size_of::<MultibootMmapEntry>(),
        );

        for i in entries_slice {
            println!("{:?}", i);
        }

        let available_entry = entries_slice
            .iter()
            .filter(|entry| entry.entry_type == MultibootMmapEntryType::Available)
            .max();

        if let Some(largest) = available_entry {
            let kernel_code_length: usize =
                unsafe { &kernel_end as *const _ as usize - &kernel_start as *const _ as usize };

            let kernel_space: PhysAddr =
                KERNEL_CODE_PHYS + max(kernel_code_length, 0x300000) + PAGES_TABLES_SIZE;

            let kernel_space_end = min(
                PhysAddr::from((largest.addr + largest.len) as usize),
                KERNEL_SPACE_MAX_END,
            );
            println!("start: {}, end: {}", kernel_space, kernel_space_end);

            //let total_memory = (largest.addr + largest.len) -
            // kernel_space.into() as _; let total_frames =
            // total_memory / PAGE_SIZE; println!("total_frames:
            // {}", total_frames);

            // Simple memory layout: lower 1GB for kernel, rest for user
            //let kernel_space = memory_start;
            //let kernel_space_end = PhysAddr::from(0x40000000); // 1GB
            //
            //let user_space = kernel_space_end;
            //let user_space_end = memory_end;

            // Create and store memory map
            //let memory_map = MemoryMap {};

            //MMAP.initialize(memory_map)
        }
    }
}
