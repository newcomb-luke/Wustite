use spin::Mutex;

use super::{LogicalIrq, Vector};

const NORMAL_VECTORS_START: u8 = 0x20;

pub static VECTOR_TO_LOGICAL_MAP: VectorToLogicalMap = VectorToLogicalMap::new();
pub static LOGICAL_TO_VECTOR_MAP: LogicalToVectorMap = LogicalToVectorMap::new();

struct InnerVectorToLogical {
    mappings: [Option<LogicalIrq>; 256],
    next_vector: u8,
}

impl InnerVectorToLogical {
    const fn new() -> Self {
        Self {
            mappings: [None; 256],
            next_vector: NORMAL_VECTORS_START,
        }
    }
}

pub struct VectorToLogicalMap {
    inner: Mutex<InnerVectorToLogical>,
}

impl VectorToLogicalMap {
    const fn new() -> Self {
        Self {
            inner: Mutex::new(InnerVectorToLogical::new()),
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

    pub fn set_entry(&self, vector: Vector, logical: LogicalIrq) {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut inner = self.inner.lock();
            inner.mappings[vector.as_u8() as usize] = Some(logical);
        });
    }

    pub fn get_entry(&self, vector: Vector) -> Option<LogicalIrq> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let inner = self.inner.lock();
            inner.mappings[vector.as_u8() as usize]
        })
    }
}

struct InnerLogicalToVector {
    mappings: [Option<Vector>; 256],
}

impl InnerLogicalToVector {
    const fn new() -> Self {
        Self {
            mappings: [None; 256],
        }
    }
}

pub struct LogicalToVectorMap {
    inner: Mutex<InnerLogicalToVector>,
}

impl LogicalToVectorMap {
    const fn new() -> Self {
        Self {
            inner: Mutex::new(InnerLogicalToVector::new()),
        }
    }

    pub fn set_entry(&self, logical: LogicalIrq, vector: Vector) {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut inner = self.inner.lock();
            inner.mappings[logical.as_u8() as usize] = Some(vector);
        });
    }

    pub fn get_entry(&self, logical: LogicalIrq) -> Option<Vector> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let inner = self.inner.lock();
            inner.mappings[logical.as_u8() as usize]
        })
    }
}
