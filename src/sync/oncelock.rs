use core::{cell::UnsafeCell, mem::MaybeUninit};

use super::once::{Once, OnceState};

pub struct OnceLock<T>
{
    once:  Once,
    value: UnsafeCell<MaybeUninit<T>>,
}

impl<T> OnceLock<T>
{
    pub const fn new() -> Self
    {
        Self {
            once:  Once::new(),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    pub fn initialize(
        &mut self,
        v: T,
    )
    {
        if self.once.is_completed() {
            return;
        } else {
            unsafe { (&mut *self.value.get()).write(v) };
            self.once.set_state(OnceState::Complete);
        }
    }

    pub fn get(&self) -> &mut T { unsafe { (&mut *self.value.get()).assume_init_mut() } }
}

unsafe impl<T> Sync for OnceLock<T>
{
}
