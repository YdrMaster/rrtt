cfg_if::cfg_if! {
    if #[cfg(target_arch = "arm")] {
        #[inline]
        pub fn interrupt_disable() -> usize {
            let x: usize;
            core::arch::asm!(
            "   MRS   {}, PRIMASK
                CPSID I
            ",
                out(reg) x
            );
            x
        }

        #[inline]
        pub fn interrupt_enable(reg: usize) {
            core::arch::asm!(
            "
                MSR   PRIMASK, {x}
                CPSID I
            ",
                in(reg) reg
            )
        }
    } else {
        #[inline]
        pub fn interrupt_disable() -> usize {
            0
        }

        #[inline]
        pub fn interrupt_enable(_reg: usize) {}
    }
}

pub struct InterruptFreeGuard(usize);

impl InterruptFreeGuard {
    #[inline]
    pub fn new() -> Self {
        Self(interrupt_disable())
    }
}

impl Drop for InterruptFreeGuard {
    #[inline]
    fn drop(&mut self) {
        interrupt_enable(self.0)
    }
}
