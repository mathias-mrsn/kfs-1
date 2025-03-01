use addr::PhysAddr;

pub mod addr;
pub mod layout;
pub mod mmap;
pub mod paging;

pub const KS_PM_BEGIN: PhysAddr = PhysAddr(0x1000000);
pub const KS_PM_END: PhysAddr = PhysAddr(0x40000000);

// Re-export key functions from kmem module for easier access
//pub use _kmem::{PAGE_SIZE, allocate_pages, free_pages, memory_stats};
