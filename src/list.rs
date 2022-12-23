use core::{
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

/// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/include/rtdef.h#L302).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C)]
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

    /// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/include/rtservice.h#L42).
    #[inline]
    pub fn init(&mut self) {
        let ptr = unsafe { NonNull::new_unchecked(self) };
        self.next = ptr;
        self.prev = ptr;
    }

    /// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/include/rtservice.h#L52).
    #[inline]
    pub fn insert(&mut self, n: &mut MaybeUninit<Self>) {
        let n = unsafe { n.assume_init_mut() };
        let n_ptr = unsafe { NonNull::new_unchecked(n) };
        unsafe {
            self.next.as_mut().prev = n_ptr;
            n.next = core::mem::replace(&mut self.next, n_ptr);
            n.prev = NonNull::from(self);
        }
    }
}

impl Deref for Node {
    type Target = MaybeUninit<Node>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self as *const _ as *const Self::Target) }
    }
}

impl DerefMut for Node {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self as *mut _ as *mut Self::Target) }
    }
}

impl<'a> IntoIterator for &'a Node {
    type Item = &'a Node;

    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let ptr = NonNull::from(self);
        Iter {
            pos: ptr,
            head: ptr,
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

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.pos = unsafe { self.pos.as_ref().next };
        if self.pos != self.head {
            unsafe { Some(self.pos.as_ref()) }
        } else {
            None
        }
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.pos = unsafe { self.pos.as_ref().prev };
        if self.pos != self.head {
            unsafe { Some(self.pos.as_ref()) }
        } else {
            None
        }
    }
}
