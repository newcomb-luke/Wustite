use spin::Mutex;

use super::LogicalIrq;

struct Inner {
    mappings: [Option<LogicalIrq>; 256]
}

impl Inner {
    const fn new() -> Self {
        Self {
            mappings: [None; 256]
        }
    }
}

pub static GSI_MAPPING_TABLE: GSIMappingTable = GSIMappingTable::new();

pub struct GSIMappingTable {
    inner: Mutex<Inner>
}

impl GSIMappingTable {
    const fn new() -> Self {
        Self {
            inner: Mutex::new(Inner::new())
        }
    }

    pub fn set_entry(&self, index: u8, entry: LogicalIrq) {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut inner = self.inner.lock();
            inner.mappings[index as usize] = Some(entry);
        });
    }

    pub fn get_entry(&self, index: u8) -> Option<LogicalIrq> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let inner = self.inner.lock();
            inner.mappings[index as usize]
        })
    }
}