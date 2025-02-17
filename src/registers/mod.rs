pub mod cr0;
pub mod cr3;
pub mod cr4;
pub mod cs;
pub mod ia32_efer;

pub trait RegisterAccessor<T>
{
    type Flags;

    fn read() -> Self::Flags;
    fn read_raw() -> T;
    fn read_bit(f: Self::Flags) -> bool;
    unsafe fn write(f: Self::Flags);
    unsafe fn write_raw(v: T);
    unsafe fn write_bit(
        f: Self::Flags,
        b: bool,
    );
}
