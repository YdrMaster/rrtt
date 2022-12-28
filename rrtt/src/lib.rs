#![no_std]
#![allow(unused)]

use konst::{primitive::parse_usize, unwrap_ctx};

const NAME_MAX: usize = 8;
const PRIORITY_MAX: usize = unwrap_ctx!(parse_usize(env!("PRIORITY_MAX")));

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
