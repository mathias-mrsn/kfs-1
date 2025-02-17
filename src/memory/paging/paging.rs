use crate::registers::RegisterAccessor;
use crate::registers::cr0::{CR0, CR0Flags};
use crate::registers::cr4::{CR4, CR4Flags};
use crate::registers::ia32_efer::{IA32EFER, IA32EFERFlags};

#[derive(Debug)]
pub enum PagingModes
{
    None,
    X86Bits,
    PAE,
    FourLevel,
    FiveLevel,
}

// Intel Manual v3.c4.1.1
pub fn current_paging_mode() -> PagingModes
{
    let pg = CR0::read_bit(CR0Flags::PG) as u8;
    let pae = CR4::read_bit(CR4Flags::PAE) as u8;
    let lme = IA32EFER::read_bit(IA32EFERFlags::LME) as u8;
    let la57 = CR4::read_bit(CR4Flags::LA57) as u8;

    match (pg, pae, lme, la57) {
        (1, 0, 0, 0) => PagingModes::X86Bits,
        (1, 1, 0, 0) => PagingModes::PAE,
        (1, 1, 1, 0) => PagingModes::FourLevel,
        (1, 1, 1, 1) => PagingModes::FiveLevel,
        _ => PagingModes::None,
    }
}

pub fn enable_pagging(m: PagingModes)
{
    match m {
        PagingModes::None => unsafe { CR0::write(CR0Flags::PG) },
        PagingModes::X86Bits => unsafe {
            CR0::write_bit(CR0Flags::PG, false);
            CR4::write_bit(CR4Flags::PAE, false);
            IA32EFER::write_bit(IA32EFERFlags::LME, false);
            CR4::write_bit(CR4Flags::LA57, false);
            CR0::write_bit(CR0Flags::PG, true);
        },
        PagingModes::PAE => unsafe {
            CR0::write_bit(CR0Flags::PG, false);
            CR4::write_bit(CR4Flags::PAE, true);
            IA32EFER::write_bit(IA32EFERFlags::LME, false);
            CR4::write_bit(CR4Flags::LA57, false);
            CR0::write_bit(CR0Flags::PG, true);
        },
        PagingModes::FourLevel => unsafe {
            CR0::write_bit(CR0Flags::PG, false);
            CR4::write_bit(CR4Flags::PAE, true);
            IA32EFER::write_bit(IA32EFERFlags::LME, true);
            CR4::write_bit(CR4Flags::LA57, false);
            CR0::write_bit(CR0Flags::PG, true);
        },
        PagingModes::FiveLevel => unsafe {
            CR0::write_bit(CR0Flags::PG, false);
            CR4::write_bit(CR4Flags::PAE, true);
            IA32EFER::write_bit(IA32EFERFlags::LME, true);
            CR4::write_bit(CR4Flags::LA57, true);
            CR0::write_bit(CR0Flags::PG, true);
        },
    }
}
