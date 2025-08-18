use crate::println;
use crate::utils::math::{find_pow2_bits, next_pow2};
use core::{char::MAX, marker::PhantomData, mem, ptr::NonNull};
use core::{fmt, fmt::Debug};

use super::addr::PhysAddr;

use usize as OrderType;
pub const MAX_ORDER: usize = 11;

#[derive(Debug)]
pub enum BuddyError
{
    OutOfMemory,
    InvalidSize,
}

pub trait PageSize
{
    const PAGE_SIZE: usize;
    const PAGE_SIZE_STR: &'static str;
    const PAGE_SHIFT: usize = Self::PAGE_SIZE.trailing_zeros() as usize;
}

pub struct PageSize4Mb;

impl PageSize for PageSize4Mb
{
    const PAGE_SIZE: usize = 0x40_0000;
    const PAGE_SIZE_STR: &'static str = "4Mb";
}

pub struct PageSize4Kb;

impl PageSize for PageSize4Kb
{
    const PAGE_SIZE: usize = 0x1000;
    const PAGE_SIZE_STR: &'static str = "4Mb";
}

pub struct Allocator<T: PageSize>
{
    md_begin:      NonNull<Block>,
    full_memory:   usize,
    pub free_list: [Option<NonNull<Block>>; MAX_ORDER],
    _phantom:      PhantomData<T>,
}

impl<T: PageSize> Debug for Allocator<T>
{
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result
    {
        write!(
            f,
            "Allocator {{ md_begin: {:?}, full_memory: {}, free_list: {:?} }}",
            self.md_begin, self.full_memory, self.free_list
        )
    }
}

/// sizeof 8bytes
pub struct Block
{
    pub next: Option<NonNull<Block>>,
    pub prev: Option<NonNull<Block>>,
}

impl Block
{
    fn is_free(&self) -> bool { self.next == None && self.prev == None }

    const BLOCK_SHIFT: usize = mem::size_of::<Block>().trailing_zeros() as usize;
}

/// May cause a inf loop
impl Debug for Block
{
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result
    {
        write!(
            f,
            "Block {{ next: {:?}, prev: {:?} }}",
            self.next, self.prev
        )
    }
}

impl<T: PageSize> Allocator<T>
{
    pub fn new(
        md_begin: usize,
        available_memory: usize,
    ) -> Self
    {
        let block = NonNull::new(md_begin as *mut Block).expect("invalid metadata address");

        let mut free_list = [None; MAX_ORDER];
        free_list[MAX_ORDER - 1] = Some(block);

        Self {
            md_begin: block,
            full_memory: available_memory,
            free_list,
            _phantom: PhantomData,
        }
    }

    pub const fn size_of(&self) -> usize
    {
        (self.full_memory / T::PAGE_SIZE) * mem::size_of::<Block>()
    }

    pub const fn get_size_order(
        &self,
        order: usize,
    ) -> usize
    {
        T::PAGE_SIZE << order
    }

    pub const fn max_size() -> usize { T::PAGE_SIZE << MAX_ORDER }

    // TEST
    fn get_order(
        &self,
        size: usize,
    ) -> Result<usize, BuddyError>
    {
        if size == 0 || size > Self::max_size() {
            return Err(BuddyError::InvalidSize);
        }

        if size <= T::PAGE_SIZE {
            return Ok(0);
        }

        let order = next_pow2(size).trailing_zeros() as usize - T::PAGE_SHIFT;
        Ok(order)
    }

    //unsafe fn merge_buddies(
    //    &mut self,
    //    order: OrderType,
    //    &block: &NonNull<Block>,
    //) -> Result<(), BuddyError>
    //{
    //    block.as_mut().next = block.as_ref().next.next;
    //    Ok(())
    //}

    unsafe fn request_block(
        &mut self,
        order: OrderType,
    ) -> Result<NonNull<Block>, BuddyError>
    {
        if order >= MAX_ORDER {
            return Err(BuddyError::OutOfMemory);
        }

        if let Some(mut block) = self.free_list[order] {
            self.free_list[order] = block.as_ref().next;
            unsafe {
                block.as_mut().prev = None;
            }
            return Ok(block);
        } else {
            match self.request_block(order + 1) {
                Ok(mut block) => {
                    println!("PTR = {:?}", block.as_ptr());
                    //println!("ORDER = {:?}", order);
                    let node = block.as_ref();

                    match node.prev {
                        Some(mut prev) => prev.as_mut().next = node.next,
                        None => self.free_list[order + 1] = node.next,
                    }

                    match node.next {
                        Some(mut next) => next.as_mut().prev = node.prev,
                        None => (),
                    }

                    //let size_order = self.get_size_order(order);

                    //let mut splited = NonNull::new_unchecked(
                    //    block.as_ptr().wrapping_add(Block::BLOCK_SHIFT << order),
                    //);
                    //println!("SPPLITED = {:?}", splited);
                    //
                    //block.as_mut().next = Some(splited);
                    //block.as_mut().prev = None;
                    //splited.as_mut().prev = Some(block);
                    //splited.as_mut().next = self.free_list[order];
                    //self.free_list[order] = Some(block);

                    return Ok(block);
                }
                Err(e) => return Err(e),
            }
        }
    }

    pub fn allocate(
        &mut self,
        v: usize,
    ) -> Result<PhysAddr, BuddyError>
    {
        let order = self.get_order(v)?;
        println!("SIZE = {}, ORDER = {} ", self.get_size_order(order), order);
        let block = unsafe { self.request_block(4)? };

        Ok(PhysAddr::from(block.as_ptr() as usize))
    }

    pub fn desallocate(
        &self,
        _p: PhysAddr,
    ) -> Result<(), BuddyError>
    {
        Ok(())
    }
}
