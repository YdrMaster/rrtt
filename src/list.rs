use core::ptr::NonNull;

/// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/include/rtdef.h#L302).
#[derive(PartialEq, Eq, Debug)]
pub struct Node {
    /// point to next node.
    pub next: NonNull<Node>,
    /// point to prev node.
    pub prev: NonNull<Node>,
}

unsafe impl Sync for Node {}

impl Node {
    #[inline]
    pub const unsafe fn new_empty(this: *const Self) -> Self {
        Self {
            next: NonNull::new_unchecked(this.cast_mut()),
            prev: NonNull::new_unchecked(this.cast_mut()),
        }
    }
}
