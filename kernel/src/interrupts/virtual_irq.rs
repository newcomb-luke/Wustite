use spin::Mutex;

use super::{ErasedIrqHandler, IrqResult};

pub static VIRTUAL_IRQ_MAPPING_TABLE: VirtualIrqMappingTable = VirtualIrqMappingTable::new();

#[derive(Debug, Clone, Copy)]
pub struct HandlerEntry {
    handler: ErasedIrqHandler,
    context: usize,
}

impl HandlerEntry {
    pub const fn new(handler: ErasedIrqHandler, context: usize) -> Self {
        Self { handler, context }
    }

    pub fn call(&self, irq: VirtualIrq) -> IrqResult {
        let context = self.context as *const ();

        (self.handler)(context, irq)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct VirtualIrq(u8);

impl VirtualIrq {
    pub fn as_u8(&self) -> u8 {
        self.0
    }
}

struct Inner {
    mappings: [Option<HandlerEntry>; 256],
    next_virtual: u8,
}

impl Inner {
    const fn new() -> Self {
        Self {
            mappings: [None; 256],
            next_virtual: 0,
        }
    }
}

pub struct VirtualIrqMappingTable {
    inner: Mutex<Inner>,
}

impl VirtualIrqMappingTable {
    const fn new() -> Self {
        Self {
            inner: Mutex::new(Inner::new()),
        }
    }

    pub fn next_free_irq(&self) -> Option<VirtualIrq> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut inner = self.inner.lock();

            if inner.next_virtual >= u8::MAX {
                return None;
            }

            let irq = inner.next_virtual;

            inner.next_virtual += 1;

            Some(VirtualIrq(irq))
        })
    }

    pub fn set_entry(&self, irq: VirtualIrq, entry: HandlerEntry) {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut inner = self.inner.lock();
            inner.mappings[irq.0 as usize] = Some(entry);
        });
    }

    pub fn get_entry(&self, irq: VirtualIrq) -> Option<HandlerEntry> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let inner = self.inner.lock();
            inner.mappings[irq.0 as usize]
        })
    }
}
