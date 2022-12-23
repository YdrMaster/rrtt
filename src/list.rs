use core::{marker::PhantomData, ptr::NonNull};

/// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/include/rtdef.h#L302).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

impl<'a> IntoIterator for &'a Node {
    type Item = &'a Node;

    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            pos: unsafe { self.next.as_ref().next },
            head: NonNull::from(self),
            _lt: PhantomData,
        }
    }
}

pub struct Iter<'a> {
    pos: NonNull<Node>,
    head: NonNull<Node>,
    _lt: PhantomData<&'a ()>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos = unsafe { self.pos.as_ref().next };
        if self.pos != self.head {
            unsafe { Some(self.pos.as_ref()) }
        } else {
            None
        }
    }
}
