pub mod gdt;
pub mod handlers;
pub mod idt;

/// A structure representing a pointer to a descriptor table (GDT/IDT)
///
/// This structure matches the format required by the LGDT and LIDT instructions
/// in x86 architecture. It contains both the size limit and base address of the
/// descriptor table.
///
/// # Fields
///
/// * `limit` - The size of the descriptor table minus 1 (maximum size is 65535
///   ///   bytes)
/// * `base` - A pointer to the start of the descriptor table in memory
///
/// # Memory Layout
///
/// The structure is marked with `#[repr(C, packed)]` to ensure:
/// * No padding bytes are added between fields
/// * The memory layout matches the CPU's expected format
///
/// # Safety
///
/// This structure is primarily used in unsafe contexts when loading descriptor
/// tables via CPU instructions. The caller must ensure:
/// * The base pointer points to a valid descriptor table
/// * The limit accurately represents the table size minus 1
/// * The structure is properly aligned when used
#[derive(Debug)]
#[repr(C, packed)]
pub struct DescriptorTablePointer
{
    pub limit: u16,
    pub base:  *const (),
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum PrivilegeRings
{
    Ring0 = 0x0,
    Ring1,
    Ring2,
    Ring3,
}

impl PrivilegeRings
{
    fn from_u8(value: u8) -> Self
    {
        match value {
            0x0 => PrivilegeRings::Ring0,
            0x1 => PrivilegeRings::Ring1,
            0x2 => PrivilegeRings::Ring2,
            0x3 => PrivilegeRings::Ring3,
            _ => panic!(
                "given value: {}, doesnt match any kind of PrivilegeRings.",
                value
            ),
        }
    }
}

// TODO: Implement own fmt
#[derive(Debug)]
#[repr(C, packed)]
pub struct InterruptStackFrame
{
    eip:    u32,
    cs:     u16,
    cflags: u32,
    esp:    u32,
    ss:     u16,
}
