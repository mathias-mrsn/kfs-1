use core::fmt;

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
/// # Safety
///
/// This structure is primarily used in unsafe contexts when loading descriptor
/// tables via CPU instructions.
#[derive(Debug)]
#[repr(C, packed)]
pub struct DescriptorTablePointer
{
    pub limit: u16,
    pub base:  *const (),
}

/// Represents x86 CPU privilege levels (rings)
///
/// Each ring represents a different privilege level, with Ring0 being the most
/// privileged (kernel mode) and Ring3 being the least privileged (user mode).
///
/// # Values
/// - `Ring0` (0x0): Kernel mode, highest privilege
/// - `Ring1` (0x1): Reserved/unused in most systems
/// - `Ring2` (0x2): Reserved/unused in most systems
/// - `Ring3` (0x3): User mode, lowest privilege
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
    /// Converts a raw u8 value into a `PrivilegeRings` enum
    ///
    /// # Arguments
    /// * `value` - A u8 value between 0 and 3
    ///
    /// # Panics
    /// Panics if the value is not between 0 and 3
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

/// Represents the CPU state automatically pushed to the stack during an
/// interrupt
///
/// # Fields
/// * `eip` - Instruction pointer at interrupt
/// * `cs` - Code segment at interrupt
/// * `cflags` - CPU flags at interrupt
/// * `esp` - Stack pointer at interrupt
/// * `ss` - Stack segment at interrupt
#[repr(C, packed)]
pub struct InterruptStackFrame
{
    eip:    u32,
    cs:     u16,
    cflags: u32,
    esp:    u32,
    ss:     u16,
}

impl fmt::Debug for InterruptStackFrame
{
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result
    {
        let eip = self.eip;
        let cs = self.cs;
        let cflags = self.cflags;
        let esp = self.esp;
        let ss = self.ss;

        f.debug_struct("InterruptStackFrame")
            .field("eip", &format_args!("0x{:08x}", eip))
            .field("cs", &format_args!("0x{:04x}", cs))
            .field("cflags", &format_args!("0x{:08x}", cflags))
            .field("esp", &format_args!("0x{:08x}", esp))
            .field("ss", &format_args!("0x{:04x}", ss))
            .finish()
    }
}
