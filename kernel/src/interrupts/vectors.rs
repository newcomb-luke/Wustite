use spin::Mutex;

use super::LogicalIrq;

const NORMAL_VECTORS_START: u8 = 0x20;

struct Inner {
    mappings: [Option<LogicalIrq>; 256],
    next_vector: u8
}

impl Inner {
    const fn new() -> Self {
        Self {
            mappings: [None; 256],
            next_vector: NORMAL_VECTORS_START
        }
    }
}

pub static VECTOR_MAP: VectorMap = VectorMap::new();

pub struct VectorMap {
    inner: Mutex<Inner>
}

impl VectorMap {
    const fn new() -> Self {
        Self {
            inner: Mutex::new(Inner::new())
        }
    }

    pub fn next_free_vector(&self) -> Option<u8> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut inner = self.inner.lock();

            if inner.next_vector >= u8::MAX {
                return None;
            }

            let vector = inner.next_vector;

            inner.next_vector += 1;

            Some(vector)
        })
    }

    pub fn set_entry(&self, vector: u8, logical: LogicalIrq) {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut inner = self.inner.lock();
            inner.mappings[vector as usize] = Some(logical);
        });
    }

    pub fn get_entry(&self, vector: u8) -> Option<LogicalIrq> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let inner = self.inner.lock();
            inner.mappings[vector as usize]
        })
    }
}