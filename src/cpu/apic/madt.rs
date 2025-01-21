use super::{SDT, SDTError, SDTHeader};

#[derive(Debug)]
pub struct MADT
{
    pub hdr: SDTHeader,
}

impl SDT for MADT
{
    const SIGNATURE: &'static [u8; 4] = b"APIC";

    #[inline]
    fn validate(&self) -> Result<(), SDTError>
    {
        if self.hdr.signature != *Self::SIGNATURE {
            return Err(SDTError::InvalidSignature);
        }

        self.hdr.validate()
    }
}
