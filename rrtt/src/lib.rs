#![no_std]
#![allow(unused)]

const NAME_MAX: usize = 8;
const PRIORITY_MAX: usize = 8;

macro_rules! container_of {
    ($ptr:expr, $ty:ty, $field:ident) => {
        ($ptr as *const _ as usize - memoffset::offset_of!($ty, $field)) as *const $ty
    };
}

mod cpu;
mod list;
mod object;
mod scheduler;
mod thread;

type TodoType = ();
