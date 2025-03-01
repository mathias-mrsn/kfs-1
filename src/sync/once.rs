use core::sync::atomic::{AtomicUsize, Ordering::Acquire};

type Primitive = usize;
type Futex = AtomicUsize;

const INCOMPLETE: Primitive = 0;
const COMPLETE: Primitive = 1;

const STATE_MASK: Primitive = 0b1;

pub struct Once
{
    state: Futex,
}

pub enum OnceState
{
    Incomplete,
    Complete,
}

impl Once
{
    #[inline]
    pub const fn new() -> Self
    {
        Self {
            state: Futex::new(INCOMPLETE),
        }
    }

    #[inline]
    pub fn is_completed(&self) -> bool { self.state.load(Acquire) == COMPLETE }

    #[inline]
    pub fn state(&mut self) -> OnceState
    {
        match *self.state.get_mut() {
            INCOMPLETE => OnceState::Incomplete,
            COMPLETE => OnceState::Complete,
            _ => panic!("error while loading Once state"),
        }
    }

    pub fn set_state(
        &mut self,
        s: OnceState,
    )
    {
        *self.state.get_mut() = match s {
            OnceState::Incomplete => INCOMPLETE,
            OnceState::Complete => COMPLETE,
        }
    }
}
