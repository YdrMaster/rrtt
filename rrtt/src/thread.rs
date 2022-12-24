use crate::{cpu::InterruptFreeGuard, list, object::Object, scheduler::CURRENT_THREAD};
use core::ptr::NonNull;

/// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/include/rtdef.h#L479).
pub struct Thread {
    header: Object,
    list: list::Node,

    sp: NonNull<usize>,
    entry: NonNull<usize>,
    parameter: NonNull<usize>,
    stack_address: NonNull<usize>,
    stack_size: usize,
    error: usize,
    stat: u8,
    current_priority: u8,
    init_priority: u8,
    number_mask: usize,
    #[cfg(feature = "event")]
    event_size: u32,
    #[cfg(feature = "event")]
    event_info: u8,
    init_tick: usize,
    remain_tick: usize,
    // TODO TIMER
    cleanup: Option<fn(NonNull<Thread>)>,
    user_data: u32,
}

/// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/src/thread.c#L83).
fn thread_cleanup_execute(thread: &mut Thread) {
    let _guard = InterruptFreeGuard::new();
    if let Some(cleanup) = thread.cleanup {
        cleanup(NonNull::from(thread));
    }
}

pub fn exit() {
    let thread = unsafe { &mut *CURRENT_THREAD };
    let _guard = InterruptFreeGuard::new();
    thread_cleanup_execute(thread);
}

//     _thread_cleanup_execute(thread);

//     /* remove from schedule */
//     rt_schedule_remove_thread(thread);
//     /* change stat */
//     thread->stat = RT_THREAD_CLOSE;

//     /* remove it from timer list */
//     rt_timer_detach(&thread->thread_timer);

//     if (rt_object_is_systemobject((rt_object_t)thread) == RT_TRUE)
//     {
//         rt_object_detach((rt_object_t)thread);
//     }
//     else
//     {
//         /* insert to defunct thread list */
//         rt_list_insert_after(&rt_thread_defunct, &(thread->tlist));
//     }

//     /* switch to next task */
//     rt_schedule();
