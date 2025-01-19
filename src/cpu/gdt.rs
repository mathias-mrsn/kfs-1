use super::DescriptorTablePointer;
use super::PrivilegeRings;
use crate::instructions::tables::lgdt;
use core::fmt;
use core::mem;
use core::ops::{Index, IndexMut};
use core::ptr;

/// Warning: This variable is tmp stored in stack so it must not be higher than
/// STACK_SIZE
const GDT_LIMIT: usize = 7;

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Entry
{
    limit_lower: u16,
    base_lower:  u16,
    base_mid:    u8,
    access:      EntryAccess,
    flags:       EntryFlags,
    base_upper:  u8,
}

impl Default for Entry
{
    fn default() -> Self
    {
        Self {
            limit_lower: 0x00,
            base_lower:  0x00,
            base_mid:    0x00,
            access:      EntryAccess::default(),
            flags:       EntryFlags::default(),
            base_upper:  0x00,
        }
    }
}

impl fmt::Debug for Entry
{
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result
    {
        f.debug_struct("Entry")
            .field("Base", &format_args!("{:#x}", self.rd_base()))
            .field("Limit", &format_args!("{:#x}", self.rd_limit()))
            .field("Flags", &format_args!("{:?}", self.flags))
            .field("Access", &format_args!("{:?}", self.access))
            .finish()
    }
}

impl Entry
{
    /// Commun flags between Flat Model Descriptors
    const FM_COMMUN: Self = Self {
        limit_lower: 0xFFFF,
        base_lower:  0x0000,
        base_mid:    0x00,
        access:      EntryAccess(0x92),
        flags:       EntryFlags(0xCF),
        base_upper:  0x00,
    };

    #[inline]
    fn wr_limit(
        &mut self,
        limit: u32,
    )
    {
        self.limit_lower = (limit & 0xFFFF) as u16;
        self.flags.wr_limit(((limit >> 16) & 0xF) as u8);
    }

    #[inline]
    fn rd_limit(&self) -> u32
    {
        let mut limit: u32;
        limit = self.flags.rd_limit() as u32;
        limit <<= 4;
        limit += self.limit_lower as u32;

        limit
    }

    #[inline]
    fn wr_base(
        &mut self,
        base: u32,
    )
    {
        self.base_lower = (base & 0xFFFF) as u16;
        self.base_mid = ((base >> 16) & 0xFF) as u8;
        self.base_upper = ((base >> 24) & 0xFF) as u8;
    }

    #[inline]
    fn rd_base(&self) -> u32
    {
        let mut base: u32;
        base = self.base_upper as u32;
        base <<= 8;
        base += self.base_mid as u32;
        base <<= 8;
        base += self.base_lower as u32;

        base
    }
}

#[derive(Copy, Clone)]
pub struct EntryFlags(u8);

impl Default for EntryFlags
{
    fn default() -> Self { Self(0) }
}

impl fmt::Debug for EntryFlags
{
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result
    {
        f.debug_struct("EntryOptions")
            .field("Granuality", &self.rd_granuality())
            .field("Size Flag", &self.rd_sizeflag())
            .field("Long Mode", &self.rd_longmode())
            .finish()
    }
}

impl EntryFlags
{
    #[inline]
    fn wr_limit(
        &mut self,
        v: u8,
    )
    {
        self.0 = (self.0 & 0xF0) | ((v as u8) & 0xF);
    }

    #[inline]
    fn rd_limit(&self) -> u8 { (self.0 & 0xF) as _ }

    #[inline]
    fn wr_granuality(
        &mut self,
        b: bool,
    )
    {
        self.0 = (self.0 & 0x7F) | ((b as u8) << 7);
    }

    #[inline]
    fn rd_granuality(&self) -> bool { ((self.0 >> 7) & 0x1) != 0 }

    #[inline]
    fn wr_sizeflag(
        &mut self,
        b: bool,
    )
    {
        self.0 = (self.0 & 0xBF) | ((b as u8) << 6);
    }

    #[inline]
    fn rd_sizeflag(&self) -> bool { ((self.0 >> 6) & 0x1) != 0 }

    #[inline]
    fn wr_longmode(
        &mut self,
        b: bool,
    )
    {
        self.0 = (self.0 & 0xDF) | ((b as u8) << 5);
    }

    #[inline]
    fn rd_longmode(&self) -> bool { ((self.0 >> 5) & 0x1) != 0 }
}

#[derive(Copy, Clone)]
pub struct EntryAccess(u8);

impl Default for EntryAccess
{
    fn default() -> Self { Self(0) }
}

impl fmt::Debug for EntryAccess
{
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result
    {
        f.debug_struct("EntryOptions")
            .field("Present Bit", &self.rd_present())
            .field("DPL", &format_args!("{:?}", self.rd_dpl()))
            .field("Segment Type", &self.rd_segtype())
            .field("Executable", &self.rd_executable())
            .field("Direction", &self.rd_direction())
            .field("Readable/Writable", &self.rd_readable())
            .field("Access", &self.rd_access())
            .finish()
    }
}

impl EntryAccess
{
    #[inline]
    fn wr_present(
        &mut self,
        present: bool,
    )
    {
        self.0 = (self.0 & 0x7F) | ((present as u8) << 7);
    }

    #[inline]
    fn rd_present(&self) -> bool { ((self.0 >> 7) & 0x1) != 0 }

    #[inline]
    fn wr_dpl(
        &mut self,
        ring: PrivilegeRings,
    )
    {
        self.0 = (self.0 & 0x9F) | ((ring as u8) << 5);
    }

    #[inline]
    fn rd_dpl(&self) -> PrivilegeRings { PrivilegeRings::from_u8((self.0 >> 5) as u8 & 0x3) }

    #[inline]
    fn wr_segtype(
        &mut self,
        b: bool,
    )
    {
        self.0 = (self.0 & 0xEF) | ((b as u8) << 4);
    }

    #[inline]
    fn rd_segtype(&self) -> bool { ((self.0 >> 4) & 0x1) != 0 }

    #[inline]
    fn wr_executable(
        &mut self,
        b: bool,
    )
    {
        self.0 = (self.0 & 0xF7) | ((b as u8) << 3);
    }

    #[inline]
    fn rd_executable(&self) -> bool { ((self.0 >> 3) & 0x1) != 0 }

    #[inline]
    fn wr_direction(
        &mut self,
        b: bool,
    )
    {
        self.0 = (self.0 & 0xFB) | ((b as u8) << 2);
    }

    #[inline]
    fn rd_direction(&self) -> bool { ((self.0 >> 2) & 0x1) != 0 }

    #[inline]
    fn wr_readable(
        &mut self,
        b: bool,
    )
    {
        self.0 = (self.0 & 0xFD) | ((b as u8) << 1);
    }

    #[inline]
    fn rd_readable(&self) -> bool { ((self.0 >> 1) & 0x1) != 0 }

    #[inline]
    fn wr_access(
        &mut self,
        b: bool,
    )
    {
        self.0 = (self.0 & 0xFE) | (b as u8);
    }

    #[inline]
    fn rd_access(&self) -> bool { (self.0 & 0x1) != 0 }
}

pub struct GlobalDescriptorTable
{
    null:         Entry,
    kernel_code:  Entry,
    kernel_data:  Entry,
    kernel_stack: Entry,
    user_code:    Entry,
    user_data:    Entry,
    user_stack:   Entry,
    descriptors:  [Entry; GDT_LIMIT - 7],
}

impl Default for GlobalDescriptorTable
{
    fn default() -> Self
    {
        Self {
            null:         Entry::default(),
            kernel_code:  Entry::default(),
            kernel_data:  Entry::default(),
            kernel_stack: Entry::default(),
            user_code:    Entry::default(),
            user_data:    Entry::default(),
            user_stack:   Entry::default(),
            descriptors:  [Entry::default(); GDT_LIMIT - 7],
        }
    }
}

impl GlobalDescriptorTable
{
    pub fn clear(&mut self) { *self = Self::default(); }

    pub unsafe fn load(&self)
    {
        let ptr = self.as_ptr();
        lgdt(&ptr);
    }

    pub fn as_ptr(&self) -> DescriptorTablePointer
    {
        DescriptorTablePointer {
            limit: (mem::size_of::<Self>() - 1) as u16,
            base:  self as *const Self as *const (),
        }
    }

    pub unsafe fn external_load(
        &self,
        address: u32,
    )
    {
        let size: usize = mem::size_of::<Self>();

        unsafe {
            ptr::copy::<u8>(ptr::addr_of!(*self) as _, address as _, size);
        }

        let ptr: DescriptorTablePointer = DescriptorTablePointer {
            limit: (size - 1) as u16,
            base:  address as _,
        };

        unsafe { lgdt(&ptr) };
    }
}

impl Index<u16> for GlobalDescriptorTable
{
    type Output = Entry;

    #[inline]
    fn index(
        &self,
        index: u16,
    ) -> &Self::Output
    {
        match index {
            0 => panic!("cannot change the null entry"),
            1 => &self.kernel_code,
            2 => &self.kernel_data,
            3 => &self.kernel_stack,
            4 => &self.user_code,
            5 => &self.user_data,
            6 => &self.user_stack,
            i @ 7..=8197 => &self.descriptors[usize::from(i) - 32],
            _ => panic!("out of bounds"),
        }
    }
}

impl IndexMut<u16> for GlobalDescriptorTable
{
    #[inline]
    fn index_mut(
        &mut self,
        index: u16,
    ) -> &mut Self::Output
    {
        match index {
            0 => panic!("cannot change the null entry"),
            1 => &mut self.kernel_code,
            2 => &mut self.kernel_data,
            3 => &mut self.kernel_stack,
            4 => &mut self.user_code,
            5 => &mut self.user_data,
            6 => &mut self.user_stack,
            i @ 7..=8197 => &mut self.descriptors[usize::from(i) - 32],
            _ => panic!("out of bounds"),
        }
    }
}

#[unsafe(no_mangle)]
pub fn setup()
{
    let mut gdt: GlobalDescriptorTable = GlobalDescriptorTable::default();
    unsafe {
        gdt.kernel_code = Entry::FM_COMMUN;
        gdt.kernel_code.access.wr_executable(true);
        gdt.kernel_code.access.wr_dpl(PrivilegeRings::Ring0);

        gdt.kernel_data = Entry::FM_COMMUN;
        gdt.kernel_data.access.wr_dpl(PrivilegeRings::Ring0);

        gdt.kernel_stack = Entry::FM_COMMUN;
        gdt.kernel_stack.access.wr_dpl(PrivilegeRings::Ring0);

        gdt.user_code = Entry::FM_COMMUN;
        gdt.user_code.access.wr_executable(true);
        gdt.user_code.access.wr_dpl(PrivilegeRings::Ring3);

        gdt.user_data = Entry::FM_COMMUN;
        gdt.user_data.access.wr_dpl(PrivilegeRings::Ring3);

        gdt.user_stack = Entry::FM_COMMUN;
        gdt.user_stack.access.wr_dpl(PrivilegeRings::Ring3);

        gdt.external_load(0x800);
    }
}

#[test_case]
fn gdt_test()
{
    setup();

    unsafe {
        assert_eq!(*(0x800 as *mut u64).offset(0), 0x00u64);
        assert_eq!(*(0x800 as *mut u64).offset(1), 0x00cf9a000000ffffu64);
        assert_eq!(*(0x800 as *mut u64).offset(2), 0x00cf92000000ffffu64);
        assert_eq!(*(0x800 as *mut u64).offset(3), 0x00cf92000000ffffu64);
        assert_eq!(*(0x800 as *mut u64).offset(4), 0x00cffa000000ffffu64);
        assert_eq!(*(0x800 as *mut u64).offset(5), 0x00cff2000000ffffu64);
        assert_eq!(*(0x800 as *mut u64).offset(6), 0x00cff2000000ffffu64);
    }
}
