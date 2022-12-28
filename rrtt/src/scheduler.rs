use crate::{list, thread::Thread, PRIORITY_MAX};
use core::{mem::MaybeUninit, ptr::null_mut};

static mut LOCK_NEST: u16 = 0;
static mut CURRENT_PRIORITY: u8 = 0;
static mut READY_PRIORITY_GROUP: u32 = 0;
static mut PRIORITY_TABLE: [MaybeUninit<list::Node>; PRIORITY_MAX] =
    unsafe { MaybeUninit::uninit().assume_init() };
static mut THREAD_DEFUNCT: MaybeUninit<list::Node> = MaybeUninit::uninit();

#[cfg(large_priority)]
static mut READY_TABLE: [u8; 32] = [0; 32];

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

/// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/src/scheduler.c#L123).
pub fn init() {
    unsafe {
        LOCK_NEST = 0;
        CURRENT_PRIORITY = 0;
        CURRENT_THREAD = null_mut();
        READY_PRIORITY_GROUP = 0;
        for node in &mut PRIORITY_TABLE {
            node.assume_init().init();
        }

        #[cfg(large_priority)]
        {
            READY_TABLE.fill(0)
        };

        THREAD_DEFUNCT.assume_init().init();
    }
}

/// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/src/scheduler.c#L152).
pub fn start() -> ! {
    let highest_ready_priority = {
        #[cfg(large_priority)]
        {
            let number = unsafe { READY_PRIORITY_GROUP }.trailing_zeros() as usize;
            (number << 3) + unsafe { READY_TABLE[number] }.trailing_zeros();
        }

        #[cfg(small_priority)]
        {
            unsafe { READY_PRIORITY_GROUP }.trailing_zeros() as usize
        }
    };

    let to_thread = container_of!(
        unsafe { PRIORITY_TABLE[highest_ready_priority].next },
        Thread,
        list
    );
    unsafe { CURRENT_THREAD = to_thread.cast_mut() };

    //     /* switch to new thread */
    //     rt_hw_context_switch_to((rt_uint32_t)&to_thread->sp);

    unreachable!()
}
