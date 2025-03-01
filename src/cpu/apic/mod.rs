use madt::{IOApic, MADT};

use crate::println;
use core::fmt;

use super::CPUIDFeatureEDX;
use crate::controllers::{inb, outb};
use core::arch::asm;

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

use core::ptr::{read_volatile, write_volatile};

///// GPT
// === PIC definitions ===

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

/// Disable the PIC by masking all IRQs.
pub unsafe fn disable_pic()
{
    // Write 0xFF to both PIC data ports to mask all interrupts.
    outb(PIC1_DATA, 0xFF);
    outb(PIC2_DATA, 0xFF);
}

/// Remap the PIC so that its interrupts start at 0x20.
pub unsafe fn remap_pic()
{
    // Start initialization in cascade mode.
    outb(PIC1_COMMAND, 0x11);
    outb(PIC2_COMMAND, 0x11);

    // Set vector offsets: PIC1 -> 0x20, PIC2 -> 0x28.
    outb(PIC1_DATA, 0x20);
    outb(PIC2_DATA, 0x28);

    // Tell PIC1 that PIC2 is at IRQ2 and assign PIC2 its cascade identity.
    outb(PIC1_DATA, 0x04);
    outb(PIC2_DATA, 0x02);

    // Set PICs to operate in 8086/88 (MCS-80/85) mode.
    outb(PIC1_DATA, 0x01);
    outb(PIC2_DATA, 0x01);

    // Mask all interrupts again.
    outb(PIC1_DATA, 0xFF);
    outb(PIC2_DATA, 0xFF);
}

/// Disable PIC mode via the IMCR register if the system requires it.
/// (Write 0x70 to port 0x22, then 0x01 to port 0x23.)
pub unsafe fn disable_pic_mode()
{
    outb(0x22, 0x70);
    outb(0x23, 0x01);
}

// === Local APIC definitions ===

/// Common LAPIC base address (typically 0xFEE00000 on x86).
const LAPIC_BASE: usize = 0xFEE00000;
/// Spurious Interrupt Vector Register offset (from LAPIC_BASE).
const LAPIC_SVR_OFFSET: usize = 0xF0;

/// Configure the Local APIC's Spurious Interrupt Vector Register.
/// This sets the spurious vector to 0xFF and sets bit 8 to enable the APIC.
pub unsafe fn enable_local_apic()
{
    let lapic_svr = (LAPIC_BASE + LAPIC_SVR_OFFSET) as *mut u32;
    // Read the current SVR.
    let mut svr = read_volatile(lapic_svr);
    // Set the lower 8 bits to 0xFF (the spurious vector)
    svr = (svr & !0xFF) | 0xFF;
    // Set bit 8: APIC Software Enable.
    svr |= 0x100;
    write_volatile(lapic_svr, svr);
}

// === I/O APIC definitions ===

/// Offsets for the I/O APIC registers relative to its base.
const IOREGSEL_OFFSET: usize = 0x00;
const IOWIN_OFFSET: usize = 0x10;

/// Write a value to an I/O APIC register.
/// `io_apic_base`: base address of the I/O APIC (from ACPI MADT)
/// `reg`: register index
/// `value`: value to write
pub unsafe fn ioapic_write(
    io_apic_base: usize,
    reg: u32,
    value: u32,
)
{
    let ioregsel = io_apic_base as *mut u32;
    let iowin = (io_apic_base + IOWIN_OFFSET) as *mut u32;
    write_volatile(ioregsel, reg);
    write_volatile(iowin, value);
}

/// Read a value from an I/O APIC register.
pub unsafe fn ioapic_read(
    io_apic_base: usize,
    reg: u32,
) -> u32
{
    let ioregsel = io_apic_base as *mut u32;
    let iowin = (io_apic_base + IOWIN_OFFSET) as *mut u32;
    write_volatile(ioregsel, reg);
    read_volatile(iowin)
}

/// Configure an IOREDTBL entry in the I/O APIC.
/// The entry is split across two 32-bit registers (low then high).
///
/// - `entry`: the IOREDTBL entry index (for example, 1 for IRQ1 override).
/// - `vector`: the ISR vector to use.
/// - `local_apic_id`: the destination Local APIC ID.
pub unsafe fn configure_ioapic_entry(
    io_apic_base: usize,
    entry: u8,
    vector: u8,
    local_apic_id: u8,
)
{
    // Each IOREDTBL entry uses two registers.
    // The first register is at index 0x10 + (entry * 2) and the second at the next
    // index.
    let reg_low = 0x10 + (entry as u32 * 2);
    let reg_high = reg_low + 1;

    // Read current values to preserve reserved bits.
    let current_low = ioapic_read(io_apic_base, reg_low);
    let current_high = ioapic_read(io_apic_base, reg_high);

    // Build new low dword:
    // Bits 0-7: vector (set to our ISR vector)
    // Bits 8-10: delivery mode (000 for fixed)
    // Bit 11: destination mode (0 for physical)
    // Bit 13: polarity (0 for active high)
    // Bit 15: trigger mode (0 for edge)
    // Bit 16: mask (0 for enabled)
    let new_low = (current_low & 0xFFFF_FF00) | (vector as u32);

    // Build new high dword:
    // Bits 24-31: destination field (our Local APIC ID)
    let new_high = (current_high & 0x00FF_FFFF) | ((local_apic_id as u32) << 24);

    // Write the updated values back.
    ioapic_write(io_apic_base, reg_low, new_low);
    ioapic_write(io_apic_base, reg_high, new_high);
}

// === APIC MSR enabling ===

/// Enable the APIC by setting the 11th bit of the APIC base MSR (MSR 0x1B).
pub unsafe fn enable_apic_msr()
{
    const MSR_APIC_BASE: u32 = 0x1B;
    let (mut low, mut high): (u32, u32);
    asm!(
        "rdmsr",
        in("ecx") MSR_APIC_BASE,
        out("eax") low,
        out("edx") high,
    );
    let mut apic_base = ((high as u64) << 32) | (low as u64);

    // Set bit 11 (APIC Global Enable, value 0x800).
    apic_base |= 1 << 11;
    let new_low = apic_base as u32;
    let new_high = (apic_base >> 32) as u32;
    asm!(
        "wrmsr",
        in("ecx") MSR_APIC_BASE,
        in("eax") new_low,
        in("edx") new_high,
    );
}

// === ACPI MADT Parsing Stub ===

/// Stub function to parse ACPI's MADT and return the I/O APIC base address and
/// Local APIC ID. In a full implementation you would use an ACPI parser crate
/// and iterate through the MADT entries.
pub fn parse_acpi_madt() -> Option<(usize, u8)>
{
    // For demonstration purposes we return example values:
    let io_apic_address = 0xFEC00000usize; // Common I/O APIC base address.
    let local_apic_id = 0; // Example local APIC ID.
    Some((io_apic_address, local_apic_id))
}

// === Main Initialization ===

/// Initialize the interrupt controller by performing all the necessary steps.
pub unsafe fn init_interrupt_controller()
{
    // 1. Disable and remap the legacy PIC.
    disable_pic();
    remap_pic();

    // 2. Optionally disable PIC mode via the IMCR.
    disable_pic_mode();

    // 3. Enable the Local APIC (configure the spurious interrupt vector register).
    enable_local_apic();

    // 4. Parse the ACPI MADT to get the I/O APIC address and Local APIC ID.
    if let Some((io_apic_base, local_apic_id)) = parse_acpi_madt() {
        // 5. Configure an IOREDTBL entry.
        // For example, if an Interrupt Source Override remaps IRQ1,
        // choose entry 1 and set your desired ISR vector (here 0x30 is used as an
        // example).
        let io_redtbl_entry: u8 = 1;
        let isr_vector: u8 = 0x30;
        configure_ioapic_entry(io_apic_base, io_redtbl_entry, isr_vector, local_apic_id);
    }

    // 6. Enable the APIC by setting the proper bit in the APIC base MSR.
    enable_apic_msr();
}

pub fn initialize()
{
    unsafe {
        init_interrupt_controller();
        const MSR_APIC_BASE: u32 = 0x1B;
        let (mut low, mut high): (u32, u32);
        asm!(
            "rdmsr",
            in("ecx") MSR_APIC_BASE,
            out("eax") low,
            out("edx") high,
        );
        let mut apic_base = ((high as u64) << 32) | (low as u64);
        println!("msr: {:#31b}", apic_base)
    }

    let rsdp = match rsdp::search_on_bios() {
        Some(v) => v,
        None => {
            if let Some(v) = rsdp::search_on_ebda() {
                v
            } else {
                panic!("Could not find RSDP");
            }
        }
    };

    let rsdt = unsafe { &*(rsdp.get_rsdt()) };

    // println!("{:#x}", rsdt as *const _ as usize);

    let madt = rsdt.find_sdt::<MADT>();

    for it in madt.unwrap().iter::<IOApic>() {
        // println!("e: {:?}", (it.io_apic_address as *mut u32));
        unsafe {
            *(it.io_apic_address as *mut u32).offset(0) = 0x12;
            *(it.io_apic_address as *mut u32).offset(0x4) = 35;

            *(it.io_apic_address as *mut u32).offset(0) = 0x13;
            *(it.io_apic_address as *mut u32).offset(0x4) = 00;
        }
        // println!("e: {:?}", it);
    }

    println!("{:?}", madt);
}
