use core::ptr;

pub mod mem;

#[inline(always)]
pub unsafe fn writec<T>(
    dest: *mut T,
    value: T,
    count: usize,
) where
    T: Clone + Copy,
{
    for i in 0..count {
        ptr::write::<T>(dest.add(i), value);
    }
}
