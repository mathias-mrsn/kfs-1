use core::fmt;
use core::{ops::Range, ptr};

use super::rsdt::RSDT;

const RSDP_SIGNATURE: [u8; 8] = *b"RSD PTR ";
const EBDA_SEG_PTR: u16 = 0x40E;
const EBDA_END: usize = 0x9ffff;
const EBDA_AREA: Range<u32> = 0x00080000..0x0009FFFF;
const BIOS_AREA: Range<u32> = 0x000E0000..0x000FFFFF;

pub enum Error
{
    InvalidSignature,
    InvalidChecksum,
}

/// A representation of the RSDP structure
#[repr(C, packed)]
pub struct RSDP
{
    pub signature:     [u8; 8],
    checksum:          u8,
    oem_id:            [u8; 6],
    revision:          u8,
    rsdt_address:      u32,
    // Fields for ACPI 2.0+
    length:            u32,
    xsdt_address:      u64,
    extended_checksum: u8,
    reserved:          [u8; 3],
}

impl fmt::Debug for RSDP
{
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result
    {
        let signature = core::str::from_utf8(&self.signature).unwrap_or("Invalid UTF-8");
        let oem_id = core::str::from_utf8(&self.oem_id).unwrap_or("Invalid UTF-8");
        let rsdt_address = self.rsdt_address;
        let length = self.length;
        let xsdt_address = self.xsdt_address;

        f.debug_struct("RSDP")
            .field("signature", &signature)
            .field("checksum", &format_args!("{:#x}", self.checksum))
            .field("oem_id", &oem_id)
            .field("revision", &self.revision)
            .field("rsdt_address", &format_args!("{:#x}", &rsdt_address))
            .field("length", &length)
            .field("xsdt_address", &format_args!("{:#x}", &xsdt_address))
            .field(
                "extended_checksum",
                &format_args!("{:#x}", self.extended_checksum),
            )
            .finish()
    }
}

impl RSDP
{
    pub fn validate(&self) -> Result<(), Error>
    {
        if self.signature != RSDP_SIGNATURE {
            return Err(Error::InvalidSignature);
        }

        let size: usize = if self.revision == 0 {
            20usize
        } else {
            self.length as _
        };

        let bytes = unsafe { core::slice::from_raw_parts(self as *const RSDP as *const u8, size) };
        let checksum_valid = bytes.iter().fold(0u8, |sum, &byte| sum.wrapping_add(byte)) == 0;
        if !checksum_valid {
            return Err(Error::InvalidChecksum);
        }

        Ok(())
    }

    pub fn get_signature(&self) -> &str
    {
        core::str::from_utf8(&self.signature).unwrap_or("Invalid UTF-8")
    }

    pub fn get_oem_id(&self) -> &str
    {
        core::str::from_utf8(&self.oem_id).unwrap_or("Invalid UTF-8")
    }

    pub fn get_rsdt(&self) -> *const RSDT { self.rsdt_address as *const RSDT }
}

pub fn search_on_bios() -> Option<&'static RSDP>
{
    for address in BIOS_AREA.step_by(16) {
        let ptr = address as *const RSDP;
        let rsdp = unsafe { &*ptr };

        if let Ok(_) = rsdp.validate() {
            return Some(rsdp);
        }
    }

    None
}

pub fn search_on_ebda() -> Option<&'static RSDP>
{
    let mut ebda_start = unsafe { ptr::read_unaligned(EBDA_SEG_PTR as *const _) };
    ebda_start <<= 4;

    let range = if EBDA_AREA.contains(&ebda_start) {
        EBDA_AREA
    } else {
        ebda_start..ebda_start + 1024
    };

    for address in range.step_by(16) {
        let ptr = address as *const RSDP;
        let rsdp = unsafe { &*ptr };

        if let Ok(_) = rsdp.validate() {
            return Some(rsdp);
        }
    }

    None
}
