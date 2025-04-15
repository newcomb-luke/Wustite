use spin::Mutex;

use super::{ErasedIrqHandler, IrqResult};

pub static LOGICAL_IRQ_MAPPING_TABLE: LogicalIrqMappingTable = LogicalIrqMappingTable::new();

#[derive(Debug, Clone, Copy)]
pub struct HandlerEntry {
    handler: ErasedIrqHandler,
    context: usize
}

impl HandlerEntry {
    pub const fn new(handler: ErasedIrqHandler, context: usize) -> Self {
        Self {
            handler,
            context
        }
    }

    pub fn call(&self, irq: LogicalIrq) -> IrqResult {
        let context = self.context as *const ();

        (self.handler)(context, irq)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct LogicalIrq(u8);

struct Inner {
    mappings: [Option<HandlerEntry>; 256],
    next_logical: u8,
}

impl Inner {
    const fn new() -> Self {
        Self {
            mappings: [None; 256],
            next_logical: 0,
        }
    }
}

pub struct LogicalIrqMappingTable {
    inner: Mutex<Inner>,
}

impl LogicalIrqMappingTable {
    const fn new() -> Self {
        Self {
            inner: Mutex::new(Inner::new()),
        }
    }

    pub fn next_free_irq(&self) -> Option<LogicalIrq> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut inner = self.inner.lock();

            if inner.next_logical >= u8::MAX {
                return None;
            }

            let irq = inner.next_logical;

            inner.next_logical += 1;

            Some(LogicalIrq(irq))
        })
    }

    pub fn set_entry(&self, index: LogicalIrq, entry: HandlerEntry) {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut inner = self.inner.lock();
            inner.mappings[index.0 as usize] = Some(entry);
        });
    }

    pub fn get_entry(&self, index: LogicalIrq) -> Option<HandlerEntry> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let inner = self.inner.lock();
            inner.mappings[index.0 as usize]
        })
    }
}
