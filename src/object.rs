use crate::{cpu, list, scheduler, TodoType, NAME_MAX};
use core::{
    ffi::CStr,
    mem::{size_of, MaybeUninit},
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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C)]
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

impl Object {
    /// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/src/object.c#L201).
    pub unsafe fn get_information(
        r#type: ObjectClassType,
    ) -> Option<&'static mut ObjectInformation> {
        OBJECT_CONTAINER.iter_mut().find(|r| r.r#type == r#type)
    }

    /// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/src/object.c#L220).
    pub fn get_length(r#type: ObjectClassType) -> usize {
        let Some(info) = (unsafe { Self::get_information(r#type) }) else { return 0; };
        let _guard = cpu::InterruptFreeGuard::new();
        info.object_list.into_iter().count()
    }

    /// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/src/object.c#L248).
    pub fn get_pointers(r#type: ObjectClassType, buf: &mut [MaybeUninit<Object>]) -> usize {
        let Some(info) = (unsafe { Self::get_information(r#type) }) else { return 0; };
        let _guard = cpu::InterruptFreeGuard::new();
        buf.iter_mut()
            .zip(info.object_list.into_iter().rev())
            .map(|(obj, node)| obj.write(unsafe { *(container_of!(node, Object, list)) }))
            .count()
    }

    /// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/src/object.c#L289).
    pub fn init(object: &mut MaybeUninit<Self>, r#type: ObjectClassType, name: &str) {
        let info = unsafe { Self::get_information(r#type) }.unwrap();
        {
            let _guard = scheduler::LockNestedGuard::new();
            let _guard = cpu::InterruptFreeGuard::new();
            for member in info
                .object_list
                .into_iter()
                .map(|node| container_of!(node, Object, list))
            {
                assert_ne!(member, object.as_ptr());
            }
        }
        let object = unsafe { object.assume_init_mut() };
        let name = name.as_bytes();
        let len = (object.name.len() - 1).min(name.len());
        object.name[..len].copy_from_slice(&name[..len]);
        object.name[len] = 0;
        object.r#type = r#type as u8 | ObjectClassType::Static as u8;
        object.flag = 0;
        // TODO HOOK
        let _guard = cpu::InterruptFreeGuard::new();
        info.object_list.insert(&mut object.list);
    }

    /// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/src/object.c#L347).
    pub fn detach(&mut self) {
        // TODO HOOK

        self.r#type = 0;

        let _guard = cpu::InterruptFreeGuard::new();
        self.list.remove();
    }

    /// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/src/object.c#L464).
    #[inline]
    pub fn is_system_object(&self) -> bool {
        self.r#type & (ObjectClassType::Static as u8) != 0
    }

    /// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/src/object.c#L484).
    #[inline]
    pub fn get_type(&self) -> u8 {
        self.r#type & !(ObjectClassType::Static as u8)
    }

    /// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/src/object.c#L500).
    pub fn find(name: &str, r#type: ObjectClassType) -> Option<&'static mut Object> {
        let Some(info) = (unsafe { Self::get_information(r#type) }) else { return None; };

        let _guard = scheduler::LockNestedGuard::new();
        let _guard = cpu::InterruptFreeGuard::new();
        info.object_list
            .into_iter()
            .map(|node| unsafe { &mut *container_of!(node, Object, list).cast_mut() })
            .find(|obj| unsafe {
                CStr::from_ptr(obj.name.as_ptr().cast()).to_bytes() == name.as_bytes()
            })
    }
}

/// See [the c code](https://github.com/RT-Thread/rtthread-nano/blob/9177e3e2f61794205565b2c53b0cb4ed2abcc43b/rt-thread/include/rtdef.h#L344).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
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
enum ObjIdx {
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
                Object::get_information(ObjectClassType::Null),
                Object::get_information(ObjectClassType::Thread),
                #[cfg(feature = "semaphore")]
                Object::get_information(ObjectClassType::Semaphore),
                #[cfg(feature = "mutex")]
                Object::get_information(ObjectClassType::Mutex),
                #[cfg(feature = "event")]
                Object::get_information(ObjectClassType::Event),
                #[cfg(feature = "mailbox")]
                Object::get_information(ObjectClassType::MailBox),
                #[cfg(feature = "message-queue")]
                Object::get_information(ObjectClassType::MessageQueue),
                #[cfg(feature = "mem-heap")]
                Object::get_information(ObjectClassType::MemHeap),
                #[cfg(feature = "mem-pool")]
                Object::get_information(ObjectClassType::MemPool),
                #[cfg(feature = "device")]
                Object::get_information(ObjectClassType::Device),
                Object::get_information(ObjectClassType::Timer),
                Object::get_information(ObjectClassType::Unknown),
                Object::get_information(ObjectClassType::Static),
            ]
        );
    }
}

#[test]
fn test_modify() {
    use ObjectClassType as Ty;

    assert_eq!(0, Object::get_length(Ty::Null));
    assert_eq!(0, Object::get_length(Ty::Thread));

    let mut threads = unsafe { MaybeUninit::<[MaybeUninit<Object>; 2]>::uninit().assume_init() };
    Object::init(&mut threads[0], Ty::Thread, "thread0");
    Object::init(&mut threads[1], Ty::Thread, "thread1");

    assert_eq!(2, Object::get_length(Ty::Thread));

    let mut pointers = unsafe { MaybeUninit::<[MaybeUninit<Object>; 8]>::uninit().assume_init() };
    assert_eq!(2, Object::get_pointers(Ty::Thread, &mut pointers));
    for (a, b) in threads.into_iter().zip(pointers) {
        unsafe { assert_eq!(a.assume_init_read(), b.assume_init_read()) };
    }

    unsafe { threads[0].assume_init_mut() }.detach();

    assert_eq!(1, Object::get_length(Ty::Thread));
    assert_eq!(1, Object::get_pointers(Ty::Thread, &mut pointers));
    unsafe {
        assert_eq!(
            threads[1].assume_init_read(),
            pointers[0].assume_init_read(),
        )
    };

    let thread1 = Object::find("thread1", Ty::Thread);
    unsafe { assert_eq!(Some(threads[1].assume_init_mut()), thread1) };
}
