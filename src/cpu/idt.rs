use super::DescriptorTablePointer;
use super::InterruptStackFrame;
use super::PrivilegeRings;
use crate::instructions::registers::rdcs;
use crate::instructions::tables::lidt;
use core::fmt;
use core::marker::PhantomData;
use core::mem;
use core::ops::{Index, IndexMut};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum GateTypes
{
    TaskGate        = 0x5,
    InterruptGate16 = 0x6,
    TrapGate16      = 0x7,
    InterruptGate32 = 0xE,
    TrapGate32      = 0xF,
}

impl GateTypes
{
    fn from_u8(value: u8) -> Self
    {
        match value {
            0x5 => GateTypes::TaskGate,
            0x6 => GateTypes::InterruptGate16,
            0x7 => GateTypes::TrapGate16,
            0xE => GateTypes::InterruptGate32,
            0xF => GateTypes::TrapGate32,
            _ => panic!("given value: {}, doesnt match any kind of GateType.", value),
        }
    }
}

#[derive(Clone, Copy)]
pub struct EntryOptions(u8);

impl Default for EntryOptions
{
    fn default() -> Self { Self(0) }
}

impl fmt::Debug for EntryOptions
{
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result
    {
        f.debug_struct("Entry")
            .field("Gate Type", &format_args!("{:?}", self.rd_gate_type()))
            .field("DPL", &format_args!("{:?}", self.rd_dpl()))
            .field("Present", &format_args!("{:?}", self.rd_present()))
            .finish()
    }
}

impl EntryOptions
{
    #[inline]
    fn wr_dpl(
        &mut self,
        ring: PrivilegeRings,
    )
    {
        self.0 |= (ring as u8) << 5;
    }

    #[inline]
    fn rd_dpl(&self) -> PrivilegeRings { PrivilegeRings::from_u8((self.0 >> 5) & 0x3) }

    #[inline]
    fn wr_gate_type(
        &mut self,
        gate: GateTypes,
    )
    {
        self.0 |= match gate {
            GateTypes::TaskGate
            | GateTypes::TrapGate16
            | GateTypes::TrapGate32
            | GateTypes::InterruptGate16
            | GateTypes::InterruptGate32 => gate as u8,
        };
    }

    #[inline]
    fn rd_gate_type(&self) -> GateTypes { GateTypes::from_u8(self.0 & 0xF) }

    #[inline]
    fn wr_present(
        &mut self,
        present: bool,
    )
    {
        self.0 = (self.0 & 0x7F) | ((present as u8) << 7);
    }

    #[inline]
    fn rd_present(&self) -> bool { ((self.0 >> 7) & 0x1) != 0 }
}

pub type Handler = extern "x86-interrupt" fn(stack_frame: InterruptStackFrame);
pub type HandlerWithCode =
    extern "x86-interrupt" fn(stack_frame: InterruptStackFrame, error_code: u32);

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct Entry<T>
{
    offset_lower:     u16,
    segment_selector: u16,
    _reserved:        u8,
    options:          EntryOptions,
    offset_high:      u16,
    _phantom:         PhantomData<T>,
}

impl<T> Default for Entry<T>
{
    fn default() -> Self
    {
        Self {
            offset_lower:     0,
            segment_selector: 0,
            _reserved:        0,
            options:          EntryOptions::default(),
            offset_high:      0,
            _phantom:         PhantomData,
        }
    }
}

impl<T> Entry<T>
{
    #[inline]
    pub unsafe fn set_handler(
        &mut self,
        handler: *const (),
    )
    {
        self.offset_lower = (handler as u32 & 0xFFFF) as u16;
        self.segment_selector = rdcs();
        self._reserved = 0;
        self.options.wr_present(true);
        self.options.wr_gate_type(GateTypes::InterruptGate32);
        self.offset_high = ((handler as u32 >> 16) & 0xFFFF) as u16;
        self._phantom = PhantomData;
    }
}

impl<T> fmt::Debug for Entry<T>
{
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result
    {
        let mut offset: u32 = (self.offset_high as u32) << 16;
        offset += self.offset_lower as u32;

        f.debug_struct("Entry")
            .field("offset", &format_args!("{:#x}", offset))
            .field("options", &format_args!("{:?}", self.options))
            .finish()
    }
}

pub struct InterruptDescriptorTable
{
    pub divide_error:                Entry<Handler>,
    pub debug:                       Entry<Handler>,
    pub nmi_interrupt:               Entry<Handler>,
    pub breakpoint:                  Entry<Handler>,
    pub overflow:                    Entry<Handler>,
    pub bound:                       Entry<Handler>,
    pub invalid_opcode:              Entry<Handler>,
    pub device_not_available:        Entry<Handler>,
    pub double_fault:                Entry<HandlerWithCode>,
    pub coprocessor_segment_overrun: Entry<Handler>,
    pub invalid_tss:                 Entry<HandlerWithCode>,
    pub segment_not_present:         Entry<HandlerWithCode>,
    pub stack_segment_fault:         Entry<HandlerWithCode>,
    pub general_protection:          Entry<HandlerWithCode>,
    pub page_fault:                  Entry<HandlerWithCode>,
    _intel_reserved1:                Entry<Handler>,
    pub fpu:                         Entry<Handler>,
    pub alignment_check:             Entry<HandlerWithCode>,
    pub machine_check:               Entry<Handler>,
    pub simd:                        Entry<Handler>,
    pub virtualization:              Entry<Handler>,
    pub control_protection:          Entry<HandlerWithCode>,
    _intel_reserved2:                [Entry<Handler>; 31 - 22],
    user_interrupts:                 [Entry<Handler>; 256 - 32],
}

impl Default for InterruptDescriptorTable
{
    fn default() -> Self
    {
        InterruptDescriptorTable {
            divide_error:                Entry::default(),
            debug:                       Entry::default(),
            nmi_interrupt:               Entry::default(),
            breakpoint:                  Entry::default(),
            overflow:                    Entry::default(),
            bound:                       Entry::default(),
            invalid_opcode:              Entry::default(),
            device_not_available:        Entry::default(),
            double_fault:                Entry::default(),
            coprocessor_segment_overrun: Entry::default(),
            invalid_tss:                 Entry::default(),
            segment_not_present:         Entry::default(),
            stack_segment_fault:         Entry::default(),
            general_protection:          Entry::default(),
            page_fault:                  Entry::default(),
            _intel_reserved1:            Entry::default(),
            fpu:                         Entry::default(),
            alignment_check:             Entry::default(),
            machine_check:               Entry::default(),
            simd:                        Entry::default(),
            virtualization:              Entry::default(),
            control_protection:          Entry::default(),
            _intel_reserved2:            [Entry::default(); 31 - 22],
            user_interrupts:             [Entry::default(); 256 - 32],
        }
    }
}

impl InterruptDescriptorTable
{
    pub fn clear(&mut self) { *self = Self::default(); }

    pub unsafe fn load(&self)
    {
        let ptr = self.as_ptr();
        lidt(&ptr);
    }

    pub fn as_ptr(&self) -> DescriptorTablePointer
    {
        DescriptorTablePointer {
            limit: (mem::size_of::<Self>() - 1) as u16,
            base:  self as *const Self as *const (),
        }
    }
}

impl Index<u8> for InterruptDescriptorTable
{
    type Output = Entry<Handler>;

    #[inline]
    fn index(
        &self,
        index: u8,
    ) -> &Self::Output
    {
        match index {
            0..=14 | 16..=21 => {
                panic!(
                    "don't use an index to refer to kernel interrupts and exceptions; instead, \
                     directly use the variable assigned to this interruption in the IDT struct"
                )
            }
            i @ 32..=255 => &self.user_interrupts[usize::from(i) - 32],
            i @ 15 | i @ 22..=31 => panic!("entry {} is reserved", i),
        }
    }
}

impl IndexMut<u8> for InterruptDescriptorTable
{
    #[inline]
    fn index_mut(
        &mut self,
        index: u8,
    ) -> &mut Self::Output
    {
        match index {
            0..=14 | 16..=21 => {
                panic!(
                    "don't use an index to refer to kernel interrupts and exceptions; instead, \
                     directly use the variable assigned to this interruption in the IDT struct"
                )
            }
            i @ 32..=255 => &mut self.user_interrupts[usize::from(i) - 32],
            i @ 15 | i @ 22..=31 => panic!("entry {} is reserved", i),
        }
    }
}

#[repr(u8)]
enum IRQ
{
    Timer            = 32,
    Keyboard         = 33,
    PICCascading     = 34,
    SecondSerialPort = 35,
    FirstSerialPort  = 36,
    FloppyDisk       = 38,
    SystemClock      = 40,
    NetworkInterface = 42,
    USBPort          = 43,
    PS2Mouse         = 44,
}
