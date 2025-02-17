use crate::commun::{ConstFrom, ConstInto};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct VirtAddr(u32);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct PhysAddr(pub u32);

impl PhysAddr
{
    #[inline]
    pub const fn as_u64(&self) -> u64 { self.0 as _ }

    #[inline]
    pub const fn as_u32(&self) -> u32 { self.0 as _ }
}

impl From<u32> for PhysAddr
{
    fn from(value: u32) -> Self { Self(value) }
}

impl const ConstFrom<u32> for PhysAddr
{
    fn from_const(value: u32) -> PhysAddr { PhysAddr(value) }
}

impl Into<u32> for PhysAddr
{
    fn into(self) -> u32 { self.0 as _ }
}

impl const ConstInto<u32> for PhysAddr
{
    fn into_const(self) -> u32 { self.0 as _ }
}
