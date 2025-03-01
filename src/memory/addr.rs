use crate::commun::{ConstFrom, ConstInto};
use core::{
    fmt::{Display, Formatter, LowerHex, Result},
    ops::Add,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct VirtAddr(usize);

impl VirtAddr
{
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Ord, PartialOrd)]
#[repr(transparent)]
pub struct PhysAddr(pub usize);

impl PhysAddr
{
    #[inline]
    pub const fn as_u64(&self) -> u64 { self.0 as _ }

    #[inline]
    pub const fn inner(&self) -> usize { self.0 }

    #[inline]
    pub const fn as_u32(&self) -> u32 { self.0 as _ }

    #[inline]
    pub const fn as_ptr<T>(self) -> *const T { self.as_u64() as *const T }
}

impl Default for PhysAddr
{
    fn default() -> Self { Self(0) }
}

impl Display for PhysAddr
{
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> Result
    {
        write!(f, "{:#x}", self.0)
    }
}

impl LowerHex for PhysAddr
{
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> Result
    {
        let val = self.0;

        LowerHex::fmt(&val, f)
    }
}

impl From<usize> for PhysAddr
{
    fn from(value: usize) -> Self { Self(value) }
}

impl const ConstFrom<usize> for PhysAddr
{
    fn from_const(value: usize) -> PhysAddr { PhysAddr(value) }
}

impl Into<usize> for PhysAddr
{
    fn into(self) -> usize { self.0 as _ }
}

impl const ConstInto<usize> for PhysAddr
{
    fn into_const(self) -> usize { self.0 as _ }
}

impl Add<usize> for PhysAddr
{
    type Output = PhysAddr;

    fn add(
        self,
        rhs: usize,
    ) -> Self::Output
    {
        PhysAddr(
            self.0
                .checked_add(rhs as _)
                .expect("PhysAddr addition overflow"),
        )
    }
}
