use crate::{cpu, list, TodoType, NAME_MAX};
use core::{mem::size_of, ptr::NonNull};

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

/// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/src/object.c#L57).
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
    #[cfg(feature = "device")]
    obj_info!(Device),
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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
#[derive(PartialEq, Eq, Debug)]
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

/// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/src/object.c#L201).
#[inline]
pub unsafe fn get_information(r#type: ObjectClassType) -> Option<&'static mut ObjectInformation> {
    OBJECT_CONTAINER.iter_mut().find(|r| r.r#type == r#type)
}

/// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/src/object.c#L220).
pub fn get_length(r#type: ObjectClassType) -> usize {
    let Some(info) = (unsafe { get_information(r#type) }) else { return 0; };
    let reg = cpu::interrupt_disable();
    let ans = info.object_list.into_iter().count();
    cpu::interrupt_enable(reg);
    ans
}

#[test]
fn test_get_information() {
    unsafe {
        assert_eq!(
            [
                None,
                Some(&mut OBJECT_CONTAINER[ObjIdx::Thread as usize]),
                #[cfg(feature = "semaphore")]
                Some(&mut OBJECT_CONTAINER[ObjIdx::Semaphore as usize]),
                #[cfg(feature = "mutex")]
                Some(&mut OBJECT_CONTAINER[ObjIdx::Mutex as usize]),
                #[cfg(feature = "event")]
                Some(&mut OBJECT_CONTAINER[ObjIdx::Event as usize]),
                #[cfg(feature = "mailbox")]
                Some(&mut OBJECT_CONTAINER[ObjIdx::MailBox as usize]),
                #[cfg(feature = "message-queue")]
                Some(&mut OBJECT_CONTAINER[ObjIdx::MessageQueue as usize]),
                #[cfg(feature = "mem-heap")]
                Some(&mut OBJECT_CONTAINER[ObjIdx::MemHeap as usize]),
                #[cfg(feature = "mem-pool")]
                Some(&mut OBJECT_CONTAINER[ObjIdx::MemPool as usize]),
                #[cfg(feature = "device")]
                Some(&mut OBJECT_CONTAINER[ObjIdx::Device as usize]),
                Some(&mut OBJECT_CONTAINER[ObjIdx::Timer as usize]),
                None,
                None
            ],
            [
                get_information(ObjectClassType::Null),
                get_information(ObjectClassType::Thread),
                #[cfg(feature = "semaphore")]
                get_information(ObjectClassType::Semaphore),
                #[cfg(feature = "mutex")]
                get_information(ObjectClassType::Mutex),
                #[cfg(feature = "event")]
                get_information(ObjectClassType::Event),
                #[cfg(feature = "mailbox")]
                get_information(ObjectClassType::MailBox),
                #[cfg(feature = "message-queue")]
                get_information(ObjectClassType::MessageQueue),
                #[cfg(feature = "mem-heap")]
                get_information(ObjectClassType::MemHeap),
                #[cfg(feature = "mem-pool")]
                get_information(ObjectClassType::MemPool),
                #[cfg(feature = "device")]
                get_information(ObjectClassType::Device),
                get_information(ObjectClassType::Timer),
                get_information(ObjectClassType::Unknown),
                get_information(ObjectClassType::Static),
            ]
        );
    }
}

#[test]
fn test_get_length() {
    assert_eq!(0, get_length(ObjectClassType::Null));
    assert_eq!(0, get_length(ObjectClassType::Thread));
}
