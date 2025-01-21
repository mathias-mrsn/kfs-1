use super::SDTHeader;
use super::{SDT, SDTError};
use core::ptr;
use core::{mem, slice};

use crate::println;

#[repr(C, packed)]
pub struct RSDT
{
    pub hdr: SDTHeader,
}

impl SDT for RSDT
{
    const SIGNATURE: &'static [u8; 4] = b"RSDT";

    fn validate(&self) -> Result<(), SDTError>
    {
        if self.hdr.signature != *Self::SIGNATURE {
            return Err(SDTError::InvalidSignature);
        }

        self.hdr.validate()
    }
}

impl RSDT
{
    pub unsafe fn entries(&self) -> impl Iterator<Item = &SDTHeader>
    {
        let entry_count =
            (self.hdr.length - mem::size_of::<Self>() as u32) / mem::size_of::<u32>() as u32;
        let start_ptr = (self as *const Self).add(1) as *const u32;
        let entries_slice = slice::from_raw_parts(start_ptr, entry_count as usize);

        entries_slice
            .iter()
            .map(|&address| &*(address as *const SDTHeader))
    }

    pub fn find_sdt<T: SDT>(&self) -> Option<&T>
    {
        let mut entries = unsafe { self.entries() };
        let sdt = (entries.find(|sdt| sdt.signature == *T::SIGNATURE)).unwrap();
        return Some(unsafe { &*(sdt as *const _ as *const T) });
    }
}
