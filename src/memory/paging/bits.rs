// use crate::instructions::registers::{rdcr0, rdcr4, rdefer, wrcr0, wrcr4,
// wrefer}; use bitflags::bitflags;
//
// bitflags! {
//
//     #[repr(transparent)]
//     #[derive(Debug, Clone, Copy)]
//     pub struct CR4Flags: u32 {
//         const PSE = 1 << 4;
//         const PAE = 1 << 5;
//         const PGE = 1 << 7;
//         const LA57 = 1 << 12;
//         const PCIDE = 1 << 17;
//         const SMEP = 1 << 20;
//         const SMAP = 1 << 21;
//         const PEK = 1 << 22;
//         const CET = 1 << 23;
//         const PKS = 1 << 24;
//     }
//
//     #[repr(transparent)]
//     #[derive(Debug, Clone, Copy)]
//     pub struct Ia32EferMsrFlags: u64 {
//         const LME = 1 << 8;
//         const NXE = 1 << 11;
//     }
//
//     #[repr(transparent)]
//     #[derive(Debug, Clone, Copy)]
//     pub struct EFlagsFlags: u32 {
//         const AC = 1 << 18;
//     }
// }
//
// /// Bunch of functions to control and change the paging control bits.
//
// #[inline]
// pub fn rdpg() -> bool { (rdcr0() & CR0Flags::PG.bits()) != 0 }
//
// #[inline]
// pub unsafe fn wrpg(b: bool)
// {
//     wrcr0((rdcr0() ^ CR0Flags::PG.bits()) | if b == true {
// CR0Flags::PG.bits() } else { 0 }) }
//
// #[inline]
// pub fn rdpae() -> bool { (rdcr4() & CR4Flags::PAE.bits()) != 0 }
//
// #[inline]
// pub unsafe fn wrpae(b: bool)
// {
//     wrcr4((rdcr4() ^ CR4Flags::PAE.bits()) | if b == true {
// CR4Flags::PAE.bits() } else { 0 }) }
//
// #[inline]
// pub fn rdlme() -> bool { (unsafe { rdefer() } & Ia32EferMsrFlags::LME.bits())
// != 0 }
//
// #[inline]
// pub unsafe fn wrlme(b: bool)
// {
//     wrefer(
//         (rdefer() ^ Ia32EferMsrFlags::LME.bits())
//             | if b == true {
//                 Ia32EferMsrFlags::LME.bits()
//             } else {
//                 0
//             },
//     )
// }
//
// #[inline]
// pub fn rdla57() -> bool { (rdcr4() & CR4Flags::LA57.bits()) != 0 }
