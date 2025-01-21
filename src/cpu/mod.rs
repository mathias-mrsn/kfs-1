use bitflags::bitflags;
use core::arch::asm;
use core::fmt;
use gdt::{Entry, GlobalDescriptorTable};
use idt::InterruptDescriptorTable;
use lazy_static::lazy_static;

use crate::instructions::cpu::{cli, sti};

pub mod apic;
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

bitflags! {
    pub struct CPUIDFeatureECX: u32 {
        const SSE3         = 1 << 0;
        const PCLMUL       = 1 << 1;
        const DTES64       = 1 << 2;
        const MONITOR      = 1 << 3;
        const DS_CPL       = 1 << 4;
        const VMX          = 1 << 5;
        const SMX          = 1 << 6;
        const EST          = 1 << 7;
        const TM2          = 1 << 8;
        const SSSE3        = 1 << 9;
        const CID          = 1 << 10;
        const SDBG         = 1 << 11;
        const FMA          = 1 << 12;
        const CX16         = 1 << 13;
        const XTPR         = 1 << 14;
        const PDCM         = 1 << 15;
        const PCID         = 1 << 17;
        const DCA          = 1 << 18;
        const SSE4_1       = 1 << 19;
        const SSE4_2       = 1 << 20;
        const X2APIC       = 1 << 21;
        const MOVBE        = 1 << 22;
        const POPCNT       = 1 << 23;
        const TSC          = 1 << 24;
        const AES          = 1 << 25;
        const XSAVE        = 1 << 26;
        const OSXSAVE      = 1 << 27;
        const AVX          = 1 << 28;
        const F16C         = 1 << 29;
        const RDRAND       = 1 << 30;
        const HYPERVISOR   = 1 << 31;
    }
}

bitflags! {
    pub struct CPUIDFeatureEDX: u32 {
        const FPU          = 1 << 0;
        const VME          = 1 << 1;
        const DE           = 1 << 2;
        const PSE          = 1 << 3;
        const TSC          = 1 << 4;
        const MSR          = 1 << 5;
        const PAE          = 1 << 6;
        const MCE          = 1 << 7;
        const CX8          = 1 << 8;
        const APIC         = 1 << 9;
        const SEP          = 1 << 11;
        const MTRR         = 1 << 12;
        const PGE          = 1 << 13;
        const MCA          = 1 << 14;
        const CMOV         = 1 << 15;
        const PAT          = 1 << 16;
        const PSE36        = 1 << 17;
        const PSN          = 1 << 18;
        const CLFLUSH      = 1 << 19;
        const DS           = 1 << 21;
        const ACPI         = 1 << 22;
        const MMX          = 1 << 23;
        const FXSR         = 1 << 24;
        const SSE          = 1 << 25;
        const SSE2         = 1 << 26;
        const SS           = 1 << 27;
        const HTT          = 1 << 28;
        const TM           = 1 << 29;
        const IA64         = 1 << 30;
        const PBE          = 1 << 31;
    }
}

lazy_static! {
    pub static ref GDT: GlobalDescriptorTable = {
        let mut m: GlobalDescriptorTable = GlobalDescriptorTable::default();

        // cli();

        unsafe {
            m.kernel_code = Entry::FM_COMMUN;
            m.kernel_code.access.wr_executable(true);
            m.kernel_code.access.wr_dpl(PrivilegeRings::Ring0);

            m.kernel_data = Entry::FM_COMMUN;
            m.kernel_data.access.wr_dpl(PrivilegeRings::Ring0);

            m.kernel_stack = Entry::FM_COMMUN;
            m.kernel_stack.access.wr_dpl(PrivilegeRings::Ring0);

            m.user_code = Entry::FM_COMMUN;
            m.user_code.access.wr_executable(true);
            m.user_code.access.wr_dpl(PrivilegeRings::Ring3);

            m.user_data = Entry::FM_COMMUN;
            m.user_data.access.wr_dpl(PrivilegeRings::Ring3);

            m.user_stack = Entry::FM_COMMUN;
            m.user_stack.access.wr_dpl(PrivilegeRings::Ring3);

            m.external_load(0x800);

            asm!(
                r#"
            jmp ${kexec_offset}, $2f;
            2:
            mov {0:x}, {kcode_offset}
            mov %ds, {0:x}
            mov %es, {0:x}
            mov %fs, {0:x}
            mov %gs, {0:x}
            mov %ss, {0:x}
        "#,
                out(reg) _,
                kexec_offset = const 0x8,
                kcode_offset = const 0x10,
                options(nostack, nomem, att_syntax)
            );
        }

        m
    };
    pub static ref IDT: InterruptDescriptorTable = {
        let mut m: InterruptDescriptorTable = InterruptDescriptorTable::default();
        unsafe {
            m.divide_error
                .set_handler(handlers::divide_error_handler as _);
            m.debug.set_handler(handlers::debug_handler as _);

            m[34].set_handler(handlers::keyboard_handler as _);
            m[35].set_handler(handlers::keyboard_handler as _);
            m[36].set_handler(handlers::keyboard_handler as _);
            m[37].set_handler(handlers::keyboard_handler as _);
            m[38].set_handler(handlers::keyboard_handler as _);
            m[39].set_handler(handlers::keyboard_handler as _);
            m[40].set_handler(handlers::keyboard_handler as _);
            m[41].set_handler(handlers::keyboard_handler as _);
            m[42].set_handler(handlers::keyboard_handler as _);
            m[43].set_handler(handlers::keyboard_handler as _);

            m.external_load(0x1000);
            // sti();
        }
        m
    };
}
