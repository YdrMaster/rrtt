use crate::{list, TodoType, NAME_MAX};
use core::{
    mem::{size_of, transmute},
    ptr::NonNull,
};

macro_rules! obj_info {
    ($ident:ident) => {
        ObjectInformation {
            r#type: ObjectClassType::$ident,
            object_list: unsafe {
                list::Node::new_empty(&OBJECT_CONTAINER[ObjIdx::$ident as usize].object_list)
            },
            object_size: size_of::<TodoType>(),
        }
    };
}

static mut OBJECT_CONTAINER: [ObjectInformation; ObjIdx::Unknown as usize] = [
    obj_info!(Thread),
    #[cfg(feature = "semaphore")]
    obj_info!(Semaphore),
    #[cfg(feature = "mutex")]
    obj_info!(Mutex),
    #[cfg(feature = "event")]
    obj_info!(Event),
    #[cfg(feature = "mailbox")]
    obj_info!(MailBox),
    #[cfg(feature = "message-queue")]
    obj_info!(MessageQueue),
    #[cfg(feature = "mem-heap")]
    obj_info!(MemHeap),
    #[cfg(feature = "mem-pool")]
    obj_info!(MemPool),
    obj_info!(Timer),
];

/// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/include/rtdef.h#L334).
pub struct Object {
    /// name of kernel object
    name: [u8; NAME_MAX],
    /// type of kernel object
    r#type: u8,
    /// flag of kernel object
    flag: u8,
    /// list node of kernel object
    list: list::Node,
}

/// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/include/rtdef.h#L344).
#[repr(usize)]
pub enum ObjectClassType {
    /// The object is not used.
    Null = 0x00,
    /// The object is a thread.
    Thread = 0x01,
    /// The object is a semaphore.
    Semaphore = 0x02,
    /// The object is a mutex.
    Mutex = 0x03,
    /// The object is a event.
    Event = 0x04,
    /// The object is a mail box.
    MailBox = 0x05,
    /// The object is a message queue.
    MessageQueue = 0x06,
    /// The object is a memory heap.
    MemHeap = 0x07,
    /// The object is a memory pool.
    MemPool = 0x08,
    /// The object is a device.
    Device = 0x09,
    /// The object is a timer.
    Timer = 0x0a,
    /// The object is unknown.
    Unknown = 0x0c,
    /// The object is a static object.
    Static = 0x80,
}

/// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/include/rtdef.h#L377).
pub struct ObjectInformation {
    /// object class type
    r#type: ObjectClassType,
    /// object list
    object_list: list::Node,
    /// object size
    object_size: usize,
}

/// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/src/object.c#L24).
#[repr(usize)]
pub enum ObjIdx {
    /// The object is a thread.
    Thread = 0,
    #[cfg(feature = "semaphore")]
    /// The object is a semaphore.
    Semaphore,
    #[cfg(feature = "mutex")]
    /// The object is a mutex.
    Mutex,
    #[cfg(feature = "event")]
    /// The object is a event.
    Event,
    #[cfg(feature = "mailbox")]
    /// The object is a mail box.
    MailBox,
    #[cfg(feature = "message-queue")]
    /// The object is a message queue.
    MessageQueue,
    #[cfg(feature = "mem-heap")]
    /// The object is a memory heap.
    MemHeap,
    #[cfg(feature = "mem-pool")]
    /// The object is a memory pool.
    MemPool,
    #[cfg(feature = "device")]
    /// The object is a device.
    Device,
    /// The object is a timer.
    Timer,
    /// The object is unknown.
    Unknown,
}
