use std::mem::MaybeUninit;

pub struct BumpAllocator<T> {
    arena: Vec<MaybeUninit<T>>,
    size: usize,
}

impl<T> BumpAllocator<T> {
    pub fn with_capacity(cap: usize) -> Self {
        let mut bump = Vec::with_capacity(cap);
        unsafe {
            bump.set_len(cap);
        }
        BumpAllocator {
            arena: bump,
            size: 0
        }
    }
    pub fn is_full(self: &Self) -> bool {
        self.size == self.arena.len()
    }
    pub fn write(self: &mut Self, data: T) -> usize {
        self.arena[self.size].write(data);
        self.size += 1;
        self.size - 1
    }
    pub fn get(self: &mut Self, idx: usize) -> &mut T {
        unsafe {
            self.arena[idx].assume_init_mut()
        }
    }
    pub fn clear(self: &mut Self) {
        self.size = 0;
    }
}

