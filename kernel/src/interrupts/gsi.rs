use spin::Mutex;

use super::{
    GSI, VirtualIrq,
    io_apic::{PinPolarity, TriggerMode},
};

struct Inner {
    mappings: [Option<VirtualIrq>; 256],
}

impl Inner {
    const fn new() -> Self {
        Self {
            mappings: [None; 256],
        }
    }
}

pub static GSI_MAPPING_TABLE: GSIMappingTable = GSIMappingTable::new();
pub static GSI_OVERRIDE_TABLE: GSIOverrideTable = GSIOverrideTable::new();

pub struct GSIMappingTable {
    inner: Mutex<Inner>,
}

impl GSIMappingTable {
    const fn new() -> Self {
        Self {
            inner: Mutex::new(Inner::new()),
        }
    }

    pub fn set_entry(&self, gsi: GSI, irq: VirtualIrq) {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut inner = self.inner.lock();
            inner.mappings[gsi.as_u8() as usize] = Some(irq);
        });
    }

    pub fn get_entry(&self, gsi: GSI) -> Option<VirtualIrq> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let inner = self.inner.lock();
            inner.mappings[gsi.as_u8() as usize]
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GSIOverrideEntry {
    gsi: GSI,
    trigger: TriggerMode,
    polarity: PinPolarity,
}

impl GSIOverrideEntry {
    pub fn new(gsi: GSI, trigger: TriggerMode, polarity: PinPolarity) -> Self {
        Self {
            gsi,
            trigger,
            polarity,
        }
    }

    pub fn gsi(&self) -> GSI {
        self.gsi
    }

    pub fn trigger(&self) -> TriggerMode {
        self.trigger
    }

    pub fn polarity(&self) -> PinPolarity {
        self.polarity
    }
}

pub struct GSIOverrideTable {
    inner: Mutex<[Option<GSIOverrideEntry>; 256]>,
}

impl GSIOverrideTable {
    const fn new() -> Self {
        Self {
            inner: Mutex::new([None; 256]),
        }
    }

    pub fn add_override(&self, isa: GSI, gsi_override: GSIOverrideEntry) {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut inner = self.inner.lock();
            inner[isa.as_u8() as usize] = Some(gsi_override);
        });
    }

    pub fn check_override(&self, isa: GSI) -> Option<GSIOverrideEntry> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let inner = self.inner.lock();
            inner[isa.as_u8() as usize]
        })
    }
}
