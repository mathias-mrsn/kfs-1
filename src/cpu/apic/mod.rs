use madt::MADT;

use crate::println;
use core::fmt;

use super::CPUIDFeatureEDX;

pub mod madt;
pub mod rsdp;
pub mod rsdt;

pub fn does_cpu_has_apic() -> bool
{
    let cpuid = unsafe { core::arch::x86::__cpuid(1) };
    (cpuid.edx & CPUIDFeatureEDX::APIC.bits()) != 0
}

pub trait SDT
{
    const SIGNATURE: &'static [u8; 4];

    fn validate(&self) -> Result<(), SDTError>;
}

pub enum SDTError
{
    InvalidSignature,
    InvalidChecksum,
}

#[repr(C, packed)]
pub struct SDTHeader
{
    signature:        [u8; 4],
    length:           u32,
    revision:         u8,
    checksum:         u8,
    oem_id:           [u8; 6],
    oem_table_id:     [u8; 8],
    oem_revision:     u32,
    creator_id:       u32,
    creator_revision: u32,
}

impl fmt::Debug for SDTHeader
{
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result
    {
        let signature = core::str::from_utf8(&self.signature).unwrap_or("Invalid UTF-8");
        let oem_id = core::str::from_utf8(&self.oem_id).unwrap_or("Invalid UTF-8");
        let oem_table_id = core::str::from_utf8(&self.oem_table_id).unwrap_or("Invalid UTF-8");
        let length = self.length;
        let oem_revision = self.oem_revision;
        let creator_id = self.creator_id;
        let creator_revision = self.creator_revision;

        f.debug_struct("RSDP")
            .field("signature", &signature)
            .field("length", &length)
            .field("revision", &self.revision)
            .field("checksum", &format_args!("{:#x}", self.checksum))
            .field("oem_id", &oem_id)
            .field("oem_table_id", &oem_table_id)
            .field("oem_revision", &oem_revision)
            .field("creator_id", &creator_id)
            .field("creator_revision", &creator_revision)
            .finish()
    }
}

impl SDTHeader
{
    pub fn validate(&self) -> Result<(), SDTError>
    {
        let bytes = unsafe {
            core::slice::from_raw_parts(self as *const SDTHeader as *const u8, self.length as _)
        };

        let checksum_valid = bytes.iter().fold(0u8, |sum, &byte| sum.wrapping_add(byte)) == 0;
        if !checksum_valid {
            return Err(SDTError::InvalidChecksum);
        }

        Ok(())
    }
}

pub fn initialize()
{
    let rsdp = match rsdp::search_on_bios() {
        Some(v) => v,
        None => {
            if let Some(v) = rsdp::search_on_ebda() {
                v
            } else {
                panic!("invalid");
            }
        }
    };

    let rsdt = unsafe { &*(rsdp.get_rsdt()) };

    let madt = rsdt.find_sdt::<MADT>();

    println!("{:?}", madt);
}
