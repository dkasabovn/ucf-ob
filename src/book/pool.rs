use std::ops;
use std::mem::MaybeUninit;

const LEVEL_ALLOC : usize = 200;

pub struct BasicArena<T> {
    alloc: Vec<MaybeUninit<T>>,
    free: Vec<usize>,
    size: usize
}

// This shouldn't be called with anything that implements drop
impl<T> MemArena<T> for BasicArena<T> {
    fn with_capacity(capacity: usize) -> Self {
        BasicArena {
            alloc: Vec::with_capacity(capacity),
            free: Vec::new(),
            size: 0
        }
    }
    fn alloc(self: &mut Self) -> usize {
        self.size += 1;
        if self.free.is_empty() {
            let idx = self.alloc.len();
            unsafe {
                self.alloc.set_len(idx + 1);
            }
            return idx;
        } else {
            let idx = self.free.pop().unwrap();
            return idx;
        }
    }
    fn free(self: &mut Self, idx: usize) {
        self.size -= 1;
        self.free.push(idx);
    }
    fn set(self: &mut Self, idx: usize, data: T) {
        self.alloc[idx].write(data);
    }
    fn len(self: &Self) -> usize {
        self.size
    }
}


impl<T> ops::Index<usize> for BasicArena<T> {
    type Output = T;
    fn index<'a>(&'a self, i : usize) -> &'a T {
        unsafe {
            self.alloc[i].assume_init_ref()
        }
    }
}

impl <T> ops::IndexMut<usize> for BasicArena<T> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        unsafe {
            self.alloc[i].assume_init_mut()
        }
    }
}

pub trait MemArena<T>: ops::IndexMut<usize> + ops::Index<usize> {
    fn with_capacity(capacity: usize) -> Self;
    fn alloc(self: &mut Self) -> usize;
    fn free(self: &mut Self, idx: usize);
    fn set(self: &mut Self, idx: usize, data: T);
    fn len(self: &Self) -> usize;
}

