use crate::instructions::tables::lgdt;
use bitflags::bitflags;
use core::mem;

const GDT_PHYS_ADDR: u32 = 0x800;
const GDT_MAX_DESCRIPTORS: usize = 8192;

#[derive(Debug)]
#[repr(C, packed)]
pub struct DescriptorTablePointer
{
    pub limit: u16,
    pub base:  *const (),
}

#[derive(Debug)]
pub struct DescriptorTable<const M: usize = 1>
{
    table: [u64; M],
    len:   usize,
}

struct AssertTableLimit<const M: usize>;

impl<const M: usize> AssertTableLimit<M>
{
    const OK: usize = {
        assert!(M > 0, "GDT need at least 1 entry");
        assert!(
            M <= 8192,
            "GDT can be up to 65536 bytes in length (8192 entries)"
        );
        0
    };
}

impl<const M: usize> DescriptorTable<M>
where
    [(); AssertTableLimit::<M>::OK]:,
{
    pub fn clear(&mut self)
    {
        self.table.fill(0);
        self.len = 1;
    }

    pub fn push(
        &mut self,
        descriptor: u64,
    )
    {
        assert!(self.len < M, "GDT Table is already full");
        self.table[self.len] = descriptor;
        self.len += 1;
    }

    pub fn fill(
        &mut self,
        descriptors: &[u64],
    )
    {
        for &descriptor in descriptors {
            self.push(descriptor);
        }
    }

    pub fn load(&self)
    {
        let ptr = self.as_ptr();
        unsafe { lgdt(&ptr) };
    }

    pub fn as_ptr(&self) -> DescriptorTablePointer
    {
        DescriptorTablePointer {
            limit: (self.len * mem::size_of::<u64>() - 1) as u16,
            base:  self.table.as_ptr() as _,
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct DescriptorBits: u64 {
        const A = 1 << 40;
        const RW = 1 << 41;
        const DC = 1 << 42;
        const E = 1 << 43;
        const S = 1 << 44;
        const DPL_RING_3 = 3 << 45;
        const DPL_RING_2 = 2 << 45;
        const DPL_RING_1 = 1 << 45;
        const P = 1 << 47;
        const L = 1 << 53;
        const DB = 1 << 54;
        const G = 1 << 55;
        const LIMIT_MAX = 0x000F_0000_0000_FFFF;
    }
}

impl DescriptorBits
{
    const _DATA: Self = Self::from_bits_truncate(
        Self::S.bits() | Self::P.bits() | Self::RW.bits() | Self::A.bits() | Self::LIMIT_MAX.bits(),
    );

    const _CODE: Self = Self::from_bits_truncate(Self::_DATA.bits() | Self::E.bits());

    const KERNEL_CODE: Self =
        Self::from_bits_truncate(Self::_CODE.bits() | Self::G.bits() | Self::DB.bits());

    const KERNEL_DATA: Self =
        Self::from_bits_truncate(Self::_DATA.bits() | Self::G.bits() | Self::DB.bits());

    const KERNEL_STACK: Self = Self::KERNEL_DATA;

    const USER_CODE: Self =
        Self::from_bits_truncate(Self::KERNEL_CODE.bits() | Self::DPL_RING_3.bits());

    const USER_DATA: Self =
        Self::from_bits_truncate(Self::KERNEL_DATA.bits() | Self::DPL_RING_3.bits());

    const USER_STACK: Self = Self::USER_DATA;
}

pub fn setup()
{
    pub static mut GDT: *mut DescriptorTable<7> = GDT_PHYS_ADDR as *mut DescriptorTable<7>;

    unsafe {
        (*GDT).clear();
        (*GDT).fill(&[
            DescriptorBits::KERNEL_CODE.bits(),
            DescriptorBits::KERNEL_DATA.bits(),
            DescriptorBits::KERNEL_DATA.bits(),
            DescriptorBits::USER_CODE.bits(),
            DescriptorBits::USER_DATA.bits(),
            DescriptorBits::USER_DATA.bits(),
        ]);
        (*GDT).load();
    }
}
