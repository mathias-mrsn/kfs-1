use core::cmp::min;
use core::mem::size_of;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use bitflags::bitflags;

use crate::mem;
use crate::multiboot::{MultibootInfo, MultibootMmapEntry, MultibootMmapEntryType};
use crate::println;
use crate::slice;
use crate::sync::oncelock::OnceLock;

use super::addr::PhysAddr;

/// The size of a page in bytes (4KiB)
pub const PAGE_SIZE: usize = 4096;

/// The maximum order for our buddy allocator (2^MAX_ORDER pages)
const MAX_ORDER: usize = 11; // Up to 2^11 * 4KiB = 8MiB blocks

/// Global allocator instance
pub static PHYSICAL_MEMORY_ALLOCATOR: OnceLock<BuddyAllocator> = OnceLock::new();

/// Represents a free block in the buddy system
#[repr(C)]
struct FreeBlock
{
    next: *mut FreeBlock,
}

/// The buddy allocator system
pub struct BuddyAllocator
{
    /// Lists of free blocks for each order
    free_lists:      [*mut FreeBlock; MAX_ORDER + 1],
    /// Total memory managed by the allocator
    total_memory:    usize,
    /// Memory map information
    memory_start:    PhysAddr,
    memory_end:      PhysAddr,
    /// Initialization status
    initialized:     AtomicBool,
    /// Bitmap to track allocated pages
    /// Each bit represents a page (1 = allocated, 0 = free)
    allocated_pages: &'static mut [AtomicUsize],
    /// Number of currently allocated pages
    allocated_count: AtomicUsize,
    /// Total number of pages managed by the allocator
    total_pages:     usize,
}

/// Error type for memory allocation operations
#[derive(Debug)]
pub enum AllocError
{
    OutOfMemory,
    InvalidSize,
    NotInitialized,
}

impl BuddyAllocator
{
    /// Create a new buddy allocator with the given memory range
    pub fn new(
        memory_start: PhysAddr,
        memory_end: PhysAddr,
        bitmap_region: &'static mut [AtomicUsize],
    ) -> Self
    {
        let free_lists = [null_mut(); MAX_ORDER + 1];
        let total_memory = (memory_end.as_u32() - memory_start.as_u32()) as usize;
        let total_pages = total_memory / PAGE_SIZE;

        Self {
            free_lists,
            total_memory,
            memory_start,
            memory_end,
            initialized: AtomicBool::new(false),
            allocated_pages: bitmap_region,
            allocated_count: AtomicUsize::new(0),
            total_pages,
        }
    }

    /// Initialize the buddy allocator with the given memory map entries
    /// This sets up the free lists with the available memory regions
    pub fn initialize(
        &mut self,
        entries: &[MultibootMmapEntry],
    )
    {
        if self.initialized.load(Ordering::SeqCst) {
            return;
        }

        // Clear all free lists
        for list in &mut self.free_lists {
            *list = null_mut();
        }

        // Initialize the bitmap (all pages marked as allocated initially)
        let bitmap_entries = (self.total_pages + 31) / 32;
        for i in 0..bitmap_entries {
            if i < self.allocated_pages.len() {
                self.allocated_pages[i].store(!0, Ordering::SeqCst);
            }
        }

        // Process each available memory region
        for entry in entries
            .iter()
            .filter(|e| e.entry_type == MultibootMmapEntryType::Available)
        {
            let region_start = entry.addr as usize;
            let region_end = region_start + entry.len as usize;

            // Align the start address up to page boundary
            let aligned_start = align_up(region_start, PAGE_SIZE);
            // Align the end address down to page boundary
            let aligned_end = align_down(region_end, PAGE_SIZE);

            if aligned_start >= aligned_end {
                continue; // Skip regions that are too small after alignment
            }

            // Add all pages in this region to the free lists
            let mut addr = aligned_start;
            while addr + PAGE_SIZE <= aligned_end {
                // Find the maximum block size that fits at this address
                let max_block_size = self.max_block_size(addr, aligned_end);
                let order = self.size_to_order(max_block_size);

                // Free this block
                unsafe {
                    self.free_region(addr, order);
                }

                // Move to the next block
                addr += max_block_size;
            }
        }

        // Mark the allocator as initialized
        self.initialized.store(true, Ordering::SeqCst);
        println!(
            "Buddy allocator initialized: {} pages managed",
            self.total_pages
        );
    }

    /// Calculate the maximum contiguous block size (in bytes) that can be
    /// allocated at `addr`
    fn max_block_size(
        &self,
        addr: usize,
        end_addr: usize,
    ) -> usize
    {
        let mut size = PAGE_SIZE;
        let mut order = 0;

        // Try to find the largest block size that:
        // 1. Fits within the available memory
        // 2. Is properly aligned for its size
        while order < MAX_ORDER {
            let next_size = size * 2;
            if addr % next_size != 0 || addr + next_size > end_addr {
                break;
            }
            size = next_size;
            order += 1;
        }

        size
    }

    /// Convert a size in bytes to a buddy order
    fn size_to_order(
        &self,
        size: usize,
    ) -> usize
    {
        let pages = size / PAGE_SIZE;
        let mut order = 0;
        let mut order_size = 1;

        while order_size < pages && order < MAX_ORDER {
            order += 1;
            order_size *= 2;
        }

        order
    }

    /// Convert an order to size in bytes
    fn order_to_size(
        &self,
        order: usize,
    ) -> usize
    {
        PAGE_SIZE * (1 << order)
    }

    /// Mark a memory region as free and add it to the appropriate free list
    unsafe fn free_region(
        &mut self,
        addr: usize,
        order: usize,
    )
    {
        if order > MAX_ORDER {
            return;
        }

        // Create a free block at this address
        let block = addr as *mut FreeBlock;

        // Mark pages as free in the bitmap
        let page_index = (addr - self.memory_start.as_u32() as usize) / PAGE_SIZE;
        let num_pages = 1 << order;
        self.mark_pages_as_free(page_index, num_pages);

        // Add to the free list for this order
        (*block).next = self.free_lists[order];
        self.free_lists[order] = block;
    }

    /// Mark a range of pages as free in the bitmap
    fn mark_pages_as_free(
        &mut self,
        start_idx: usize,
        count: usize,
    )
    {
        for i in 0..count {
            let idx = start_idx + i;
            if idx < self.total_pages {
                let bitmap_idx = idx / 32;
                let bit_idx = idx % 32;

                if bitmap_idx < self.allocated_pages.len() {
                    let old_value = self.allocated_pages[bitmap_idx]
                        .fetch_and(!(1 << bit_idx), Ordering::SeqCst);

                    // If the page was previously allocated, decrement the count
                    if (old_value & (1 << bit_idx)) != 0 {
                        self.allocated_count.fetch_sub(1, Ordering::SeqCst);
                    }
                }
            }
        }
    }

    /// Allocate memory of the specified order
    pub fn allocate(
        &mut self,
        order: usize,
    ) -> Result<PhysAddr, AllocError>
    {
        if !self.initialized.load(Ordering::SeqCst) {
            return Err(AllocError::NotInitialized);
        }

        if order > MAX_ORDER {
            return Err(AllocError::InvalidSize);
        }

        // Find a suitable free block
        let block_opt = self.find_free_block(order);
        match block_opt {
            Some(block_addr) => {
                // Mark the pages as allocated
                let page_index =
                    (block_addr as usize - self.memory_start.as_u32() as usize) / PAGE_SIZE;
                let num_pages = 1 << order;
                self.mark_pages_as_allocated(page_index, num_pages);

                Ok(PhysAddr::from(block_addr as u32))
            }
            None => Err(AllocError::OutOfMemory),
        }
    }

    /// Find a free block of the required order, splitting larger blocks if
    /// necessary
    fn find_free_block(
        &mut self,
        order: usize,
    ) -> Option<*mut u8>
    {
        // Try to find a block of the requested size
        let mut current_order = order;

        // Look for a suitable block, starting from the requested order
        // and moving up to larger blocks if necessary
        while current_order <= MAX_ORDER {
            if !self.free_lists[current_order].is_null() {
                // Found a block, remove it from the free list
                let block = self.free_lists[current_order];
                unsafe {
                    self.free_lists[current_order] = (*block).next;
                }

                // If the block is larger than requested, split it
                let mut block_addr = block as usize;
                while current_order > order {
                    current_order -= 1;
                    let buddy_addr = block_addr + self.order_to_size(current_order);

                    // Put the buddy in the free list
                    unsafe {
                        self.free_region(buddy_addr, current_order);
                    }
                }

                return Some(block_addr as *mut u8);
            }

            current_order += 1;
        }

        None
    }

    /// Mark a range of pages as allocated in the bitmap
    fn mark_pages_as_allocated(
        &mut self,
        start_idx: usize,
        count: usize,
    )
    {
        for i in 0..count {
            let idx = start_idx + i;
            if idx < self.total_pages {
                let bitmap_idx = idx / 32;
                let bit_idx = idx % 32;

                if bitmap_idx < self.allocated_pages.len() {
                    let old_value =
                        self.allocated_pages[bitmap_idx].fetch_or(1 << bit_idx, Ordering::SeqCst);

                    // If the page was previously free, increment the count
                    if (old_value & (1 << bit_idx)) == 0 {
                        self.allocated_count.fetch_add(1, Ordering::SeqCst);
                    }
                }
            }
        }
    }

    /// Free a previously allocated block
    pub fn free(
        &mut self,
        addr: PhysAddr,
        order: usize,
    )
    {
        if !self.initialized.load(Ordering::SeqCst) || order > MAX_ORDER {
            return;
        }

        let addr_val = addr.as_u32() as usize;

        // Check if the address is properly aligned for its order
        if addr_val % self.order_to_size(order) != 0 {
            println!(
                "Warning: Attempted to free misaligned address: {:x}",
                addr_val
            );
            return;
        }

        // Mark the block as free
        unsafe {
            self.free_block(addr_val, order);
        }
    }

    /// Free a block and attempt to merge with its buddy if also free
    unsafe fn free_block(
        &mut self,
        addr: usize,
        order: usize,
    )
    {
        if order > MAX_ORDER {
            return;
        }

        // Calculate the buddy address
        let buddy_addr = addr ^ self.order_to_size(order);

        // Check if the buddy is free
        if self.is_buddy_free(buddy_addr, order) {
            // Remove the buddy from its free list
            self.remove_from_free_list(buddy_addr, order);

            // Merge with buddy and move up one order
            let merged_addr = min(addr, buddy_addr);
            self.free_block(merged_addr, order + 1);
        } else {
            // No buddy or buddy is not free, just add this block to its free list
            self.free_region(addr, order);
        }
    }

    /// Check if a buddy block is free
    fn is_buddy_free(
        &self,
        buddy_addr: usize,
        order: usize,
    ) -> bool
    {
        // Check if the buddy address is valid
        if buddy_addr < self.memory_start.as_u32() as usize
            || buddy_addr + self.order_to_size(order) > self.memory_end.as_u32() as usize
        {
            return false;
        }

        // Check if the buddy is in the free list
        let mut current = self.free_lists[order];
        while !current.is_null() {
            if current as usize == buddy_addr {
                return true;
            }
            unsafe {
                current = (*current).next;
            }
        }

        false
    }

    /// Remove a block from its free list
    fn remove_from_free_list(
        &mut self,
        addr: usize,
        order: usize,
    )
    {
        let addr_ptr = addr as *mut FreeBlock;

        if self.free_lists[order] == addr_ptr {
            // Block is at the head of the list
            unsafe {
                self.free_lists[order] = (*addr_ptr).next;
            }
            return;
        }

        // Search for the block in the list
        let mut current = self.free_lists[order];
        while !current.is_null() {
            unsafe {
                if (*current).next == addr_ptr {
                    // Found the block, remove it
                    (*current).next = (*addr_ptr).next;
                    return;
                }
                current = (*current).next;
            }
        }
    }

    /// Get the total number of managed pages
    pub fn total_pages(&self) -> usize { self.total_pages }

    /// Get the number of currently allocated pages
    pub fn allocated_pages(&self) -> usize { self.allocated_count.load(Ordering::SeqCst) }

    /// Get the number of free pages
    pub fn free_pages(&self) -> usize { self.total_pages - self.allocated_pages() }
}

/// Align `value` up to the next multiple of `align`.
fn align_up(
    value: usize,
    align: usize,
) -> usize
{
    (value + align - 1) & !(align - 1)
}

/// Align `value` down to the previous multiple of `align`.
fn align_down(
    value: usize,
    align: usize,
) -> usize
{
    value & !(align - 1)
}

// Public interface for physical memory allocation
pub fn initialize(mbi: &'static MultibootInfo)
{
    unsafe {
        // Get memory map entries from multiboot info
        let entries_slice = slice::from_raw_parts::<MultibootMmapEntry>(
            mbi.mmap_addr.as_ptr::<MultibootMmapEntry>(),
            mbi.mmap_length as usize / mem::size_of::<MultibootMmapEntry>(),
        );

        // Find largest available memory region for our bitmap
        let largest_available = entries_slice
            .iter()
            .filter(|entry| entry.entry_type == MultibootMmapEntryType::Available)
            .max();

        if let Some(largest) = largest_available {
            // Calculate physical memory limits
            let memory_start = PhysAddr::from(1 * 1024 * 1024 as u32); // Start at 1MB to avoid BIOS/bootloader area
            let memory_end = PhysAddr::from((largest.addr + largest.len) as u32);
            let total_memory = (memory_end.as_u32() - memory_start.as_u32()) as usize;
            let total_pages = total_memory / PAGE_SIZE;

            // Calculate bitmap size (1 bit per page, rounded up to usize)
            let bitmap_size_bytes = (total_pages + 7) / 8;
            let bitmap_size_words =
                (bitmap_size_bytes + size_of::<AtomicUsize>() - 1) / size_of::<AtomicUsize>();

            // Reserve space for bitmap at the start of the largest region
            let bitmap_addr = largest.addr as usize;
            let bitmap_end_addr = bitmap_addr + bitmap_size_words * size_of::<AtomicUsize>();
            let bitmap_aligned_end = align_up(bitmap_end_addr, PAGE_SIZE);

            // Create bitmap in place
            let bitmap_ptr = bitmap_addr as *mut AtomicUsize;
            let bitmap_slice = slice::from_raw_parts_mut(bitmap_ptr, bitmap_size_words);

            // Create and initialize the buddy allocator
            let mut allocator = BuddyAllocator::new(memory_start, memory_end, bitmap_slice);

            // Initialize the allocator with the memory map entries
            allocator.initialize(entries_slice);

            // Store the allocator in the global static
            if PHYSICAL_MEMORY_ALLOCATOR.set(allocator).is_err() {
                println!("Error: Failed to initialize physical memory allocator");
            }

            println!(
                "Physical memory allocator initialized with {} MB of RAM",
                total_memory / (1024 * 1024)
            );
        } else {
            println!("Error: No available memory regions found in multiboot info");
        }
    }
}

// Allocate physical memory pages
pub fn allocate_pages(count: usize) -> Result<PhysAddr, AllocError>
{
    if count == 0 {
        return Err(AllocError::InvalidSize);
    }

    // Find the smallest order that can fit the requested number of pages
    let mut order = 0;
    let mut order_size = 1;

    while order_size < count {
        order += 1;
        if order > MAX_ORDER {
            return Err(AllocError::InvalidSize);
        }
        order_size *= 2;
    }

    // Get the allocator and allocate the pages
    if let Some(allocator) = PHYSICAL_MEMORY_ALLOCATOR.get_mut() {
        allocator.allocate(order)
    } else {
        Err(AllocError::NotInitialized)
    }
}

// Free previously allocated physical memory
pub fn free_pages(
    addr: PhysAddr,
    count: usize,
)
{
    if count == 0 {
        return;
    }

    // Find the order that was used for this allocation
    let mut order = 0;
    let mut order_size = 1;

    while order_size < count {
        order += 1;
        if order > MAX_ORDER {
            println!("Warning: Invalid page count in free_pages: {}", count);
            return;
        }
        order_size *= 2;
    }

    // Get the allocator and free the pages
    if let Some(allocator) = PHYSICAL_MEMORY_ALLOCATOR.get_mut() {
        allocator.free(addr, order);
    }
}

// Get memory statistics
pub fn memory_stats() -> Option<(usize, usize, usize)>
{
    if let Some(allocator) = PHYSICAL_MEMORY_ALLOCATOR.get() {
        Some((
            allocator.total_pages(),
            allocator.allocated_pages(),
            allocator.free_pages(),
        ))
    } else {
        None
    }
}
