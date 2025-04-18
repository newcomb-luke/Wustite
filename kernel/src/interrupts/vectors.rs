use spin::Mutex;

use super::{Vector, VirtualIrq};

const NORMAL_VECTORS_START: u8 = 0x20;

pub static VECTOR_TO_VIRTUAL_MAP: VectorToVirtualMap = VectorToVirtualMap::new();
pub static VIRTUAL_TO_VECTOR_MAP: VirtualToVectorMap = VirtualToVectorMap::new();

struct InnerVectorToVirtual {
    mappings: [Option<VirtualIrq>; 256],
    next_vector: u8,
}

impl InnerVectorToVirtual {
    const fn new() -> Self {
        Self {
            mappings: [None; 256],
            next_vector: NORMAL_VECTORS_START,
        }
    }
}

pub struct VectorToVirtualMap {
    inner: Mutex<InnerVectorToVirtual>,
}

impl VectorToVirtualMap {
    const fn new() -> Self {
        Self {
            inner: Mutex::new(InnerVectorToVirtual::new()),
        }
    }

    pub fn next_free_vector(&self) -> Option<Vector> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut inner = self.inner.lock();

            if inner.next_vector >= u8::MAX {
                return None;
            }

            let vector = inner.next_vector;

            inner.next_vector += 1;

            Some(Vector::from_u8(vector))
        })
    }

    pub fn set_entry(&self, vector: Vector, irq: VirtualIrq) {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut inner = self.inner.lock();
            inner.mappings[vector.as_u8() as usize] = Some(irq);
        });
    }

    pub fn get_entry(&self, vector: Vector) -> Option<VirtualIrq> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let inner = self.inner.lock();
            inner.mappings[vector.as_u8() as usize]
        })
    }
}

struct InnerVirtualToVector {
    mappings: [Option<Vector>; 256],
}

impl InnerVirtualToVector {
    const fn new() -> Self {
        Self {
            mappings: [None; 256],
        }
    }
}

pub struct VirtualToVectorMap {
    inner: Mutex<InnerVirtualToVector>,
}

impl VirtualToVectorMap {
    const fn new() -> Self {
        Self {
            inner: Mutex::new(InnerVirtualToVector::new()),
        }
    }

    pub fn set_entry(&self, irq: VirtualIrq, vector: Vector) {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut inner = self.inner.lock();
            inner.mappings[irq.as_u8() as usize] = Some(vector);
        });
    }

    pub fn get_entry(&self, irq: VirtualIrq) -> Option<Vector> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let inner = self.inner.lock();
            inner.mappings[irq.as_u8() as usize]
        })
    }
}
