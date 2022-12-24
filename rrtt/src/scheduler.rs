use crate::thread::Thread;
use core::ptr::null_mut;

static mut LOCK_NEST: usize = 0;
pub(crate) static mut CURRENT_THREAD: *mut Thread = null_mut();

pub struct LockNestedGuard;

impl LockNestedGuard {
    #[inline]
    pub fn new() -> Self {
        unsafe { LOCK_NEST += 1 };
        Self
    }
}

impl Drop for LockNestedGuard {
    #[inline]
    fn drop(&mut self) {
        unsafe { LOCK_NEST -= 1 };
    }
}
