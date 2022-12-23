pub(crate) static mut LOCK_NEST: usize = 0;

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
