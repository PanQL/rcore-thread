use alloc::{collections::BinaryHeap, vec::Vec};
use core::cmp::Reverse;

use log::*;
use spin::Mutex;

mod naive;

pub use naive::NaiveTTScheduler;

type Tid = usize;

pub trait TTScheduler: 'static {
    fn push(&self, tid : Tid, cycle : usize, offset : usize, max_time : usize) -> bool;
    fn pop(&self) -> Option<Tid>;
    fn tick(&self, current_tid : Tid) -> bool;
}
