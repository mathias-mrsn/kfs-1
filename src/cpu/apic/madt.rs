use core::mem;

use super::{SDT, SDTError, SDTHeader};

#[repr(C)]
#[derive(Debug)]
pub struct MADT
{
    pub hdr:       SDTHeader,
    local_address: u32,
    flags:         u32,
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

trait Entry
{
    const TYPE: &'static u8;

    fn length(&self) -> Result<u8, ()>;
    fn entry_type(&self) -> Result<u8, ()>;
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct EntryHeader
{
    entry_type: u8,
    length:     u8,
}

impl Entry for EntryHeader
{
    const TYPE: &'static u8 = &0xff;

    fn length(&self) -> Result<u8, ()> { Ok(self.length) }

    fn entry_type(&self) -> Result<u8, ()> { Ok(self.entry_type) }
}

#[repr(C)]
pub struct MADTIterator<'a, T>
{
    t:        &'a MADT,
    c:        usize,
    _phantom: core::marker::PhantomData<&'a T>,
}

impl<'a, T: Entry> Iterator for MADTIterator<'a, T>
{
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T>
    {
        while self.c < (self.t.hdr.length + self.t as *const _ as u32) as usize {
            let entry = unsafe { &*(self.c as *const T) };
            if entry.entry_type().unwrap() != *T::TYPE && *T::TYPE != 0xff {
                self.c += entry.length().unwrap() as usize;
                continue;
            }
            self.c += entry.length().unwrap() as usize;
            return Some(entry);
        }
        None
    }
}

impl MADT
{
    pub fn iter<T>(&self) -> MADTIterator<T>
    {
        MADTIterator::<T> {
            t:        self,
            c:        (self as *const _ as *const u8).wrapping_add(mem::size_of::<MADT>()) as usize,
            _phantom: core::marker::PhantomData,
        }
    }
}

macro_rules! impl_entry {
    ($struct_name:ident, $type_const:expr) => {
        impl Entry for $struct_name
        {
            const TYPE: &'static u8 = &$type_const;

            fn length(&self) -> Result<u8, ()> { Ok(self.length) }

            fn entry_type(&self) -> Result<u8, ()> { Ok(self.entry_type) }
        }
    };
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct LocalApic
{
    pub entry_type:   u8,
    pub length:       u8,
    pub processor_id: u8,
    pub apic_id:      u8,
    pub flags:        u32,
}

impl_entry!(LocalApic, 0x00);

#[repr(C, packed)]
#[derive(Debug)]
pub struct IOApic
{
    pub entry_type:                   u8,
    pub length:                       u8,
    pub io_apic_id:                   u8,
    pub reserved:                     u8,
    pub io_apic_address:              u32,
    pub global_system_interrupt_base: u32,
}

// impl IOApic
// {
//
// }

impl_entry!(IOApic, 0x01);

#[repr(C, packed)]
#[derive(Debug)]
pub struct IOApicISO
{
    pub entry_type:              u8,
    pub length:                  u8,
    pub bus_source:              u8,
    pub irq_source:              u8,
    pub global_system_interrupt: u32,
    pub flags:                   u16,
}

impl_entry!(IOApicISO, 0x02);

#[repr(C, packed)]
#[derive(Debug)]
pub struct IOApicNMI
{
    pub entry_type:              u8,
    pub length:                  u8,
    pub nmi_source:              u8,
    pub reserved:                u8,
    pub flags:                   u16,
    pub global_system_interrupt: u32,
}

impl_entry!(IOApicNMI, 0x03);

#[repr(C, packed)]
#[derive(Debug)]
pub struct LocalApicNMI
{
    pub entry_type:   u8,
    pub length:       u8,
    pub processor_id: u8,
    pub flags:        u16,
    pub lint:         u8,
}

impl_entry!(LocalApicNMI, 0x04);

#[repr(C, packed)]
#[derive(Debug)]
pub struct LocalApicOverride
{
    pub entry_type:         u8,
    pub length:             u8,
    pub reserved:           u16,
    pub local_apic_address: u64,
}

impl_entry!(LocalApicOverride, 0x05);

#[repr(C, packed)]
#[derive(Debug)]
pub struct Local2Apic
{
    pub entry_type:               u8,
    pub length:                   u8,
    pub reserved:                 u16,
    pub processor_local_2apic_id: u32,
    pub flags:                    u32,
    pub acpi_id:                  u32,
}

impl_entry!(Local2Apic, 0x09);
