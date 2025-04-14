use spin::Mutex;
use x86_64::{PhysAddr, structures::paging::PageTableFlags};

use crate::{logln, memory::MEMORY_MAPPER};

const REGISTER_SELECT: u64 = 0x00;
const REGISTER_WINDOW: u64 = 0x10;

const IO_APIC_VERSION_REG: u8 = 0x01;
const IO_APIC_REDIRECT_REG_START: u8 = 0x10;

pub struct IoApicInner {
    base_address: u64,
    id: u8,
    global_system_interrupt_base: u32,
    entry_count: u8,
}

impl IoApicInner {
    fn new(base_address: u64, id: u8, global_system_interrupt_base: u32, entry_count: u8) -> Self {
        Self {
            base_address,
            id,
            global_system_interrupt_base,
            entry_count,
        }
    }

    fn write_register(&self, register: u8, value: u32) {
        unsafe {
            write_io_apic_register(self.base_address, register, value);
        }
    }

    fn read_register(&self, register: u8) -> u32 {
        unsafe { read_io_apic_register(self.base_address, register) }
    }
}

pub static IO_APIC: IoApic = IoApic::new();

pub struct IoApic {
    inner: Mutex<Option<IoApicInner>>,
}

impl IoApic {
    const fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    pub fn init(&self, base_address: u64, id: u8, global_system_interrupt_base: u32) {
        logln!(
            "[info] IO APIC {}: Initializing at 0x{:08x}",
            id,
            base_address
        );

        {
            let mut inner = self.inner.lock();

            if inner.is_some() {
                panic!("Attempted to initialize IO APIC {} twice", id);
            }

            let virt_base_address = unsafe {
                MEMORY_MAPPER
                    .map_virt_page(
                        PhysAddr::new(base_address),
                        PageTableFlags::PRESENT
                            | PageTableFlags::WRITABLE
                            | PageTableFlags::NO_CACHE,
                    )
                    .unwrap()
            }
            .as_u64();

            let io_apic_version =
                unsafe { read_io_apic_register(virt_base_address, IO_APIC_VERSION_REG) };

            let entries_count = (io_apic_version >> 16) as u8 + 1;

            *inner = Some(IoApicInner::new(
                virt_base_address,
                id,
                global_system_interrupt_base,
                entries_count,
            ));
        }

        logln!("[info] IO APIC {}: Initialized", id);
    }

    pub fn is_redirect_set(&self, gsi: u32) -> bool {
        let lower_register = self.first_redirect_register_for_gsi(gsi);
        // let upper_register = lower_register + 1;

        if let Some(inner) = self.inner.lock().as_mut() {
            let low = inner.read_register(lower_register);
            // let high = inner.read_register(upper_register);

            let vector = (low & 0xFF) as u8;

            vector != 0
        } else {
            panic!("Attempted to read IO APIC redirect before IO APIC was initialized");
        }
    }

    pub fn set_redirect(
        &self,
        gsi: u32,
        vector: u8,
        delivery_mode: DeliveryMode,
        trigger: TriggerMode,
        polarity: PinPolarity,
        mask: bool,
        destination: Destination,
    ) {
        let mut lower = vector as u32;
        lower |= delivery_mode.as_u32() << 8;
        lower |= destination.mode_bit() << 11;
        lower |= polarity.as_u32() << 13;
        lower |= trigger.as_u32() << 15;
        lower |= if mask { 1 << 16 } else { 0 };

        let upper = destination.as_field();

        let lower_register = self.first_redirect_register_for_gsi(gsi);
        let upper_register = lower_register + 1;

        if let Some(inner) = self.inner.lock().as_mut() {
            inner.write_register(lower_register, lower);
            inner.write_register(upper_register, upper);
        } else {
            panic!("Attempted to set IO APIC redirect before IO APIC was initialized");
        }

        logln!(
            "[info] IO APIC 0: Redirect set for GSI {} to vector 0x{:02x}",
            gsi,
            vector
        );
    }

    fn first_redirect_register_for_gsi(&self, gsi: u32) -> u8 {
        IO_APIC_REDIRECT_REG_START + (gsi as u8) * 2
    }
}

unsafe fn write_io_apic_register(base_address: u64, register: u8, value: u32) {
    unsafe {
        write_io_apic_register_select(base_address, register);
        write_io_apic_register_window(base_address, value);
    }
}

unsafe fn read_io_apic_register(base_address: u64, register: u8) -> u32 {
    unsafe {
        write_io_apic_register_select(base_address, register);
        read_io_apic_register_window(base_address)
    }
}

unsafe fn write_io_apic_register_select(base_address: u64, register: u8) {
    let reg_select = (base_address + REGISTER_SELECT) as *mut u32;
    unsafe {
        reg_select.write_volatile(register as u32);
    }
}

unsafe fn write_io_apic_register_window(base_address: u64, value: u32) {
    let reg_window = (base_address + REGISTER_WINDOW) as *mut u32;
    unsafe {
        reg_window.write_volatile(value);
    }
}

unsafe fn read_io_apic_register_window(base_address: u64) -> u32 {
    let reg_window = (base_address + REGISTER_WINDOW) as *mut u32;
    unsafe { reg_window.read_volatile() }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryMode {
    Fixed,
    LowestPriority,
    Smi,
    Nmi,
    Init,
    ExtInt,
}

impl DeliveryMode {
    fn as_u32(&self) -> u32 {
        match self {
            Self::Fixed => 0b000,
            Self::LowestPriority => 0b001,
            Self::Smi => 0b010,
            Self::Nmi => 0b100,
            Self::Init => 0b101,
            Self::ExtInt => 0b111,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerMode {
    Edge,
    Level,
}

impl TriggerMode {
    fn as_u32(&self) -> u32 {
        match self {
            Self::Edge => 0,
            Self::Level => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinPolarity {
    ActiveHigh,
    ActiveLow,
}

impl PinPolarity {
    fn as_u32(&self) -> u32 {
        match self {
            Self::ActiveHigh => 0,
            Self::ActiveLow => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Destination {
    Physical(u8),
    Logical(u8),
}

impl Destination {
    fn mode_bit(&self) -> u32 {
        match self {
            Destination::Physical(_) => 0,
            Destination::Logical(_) => 1,
        }
    }

    fn as_field(&self) -> u32 {
        let value = match self {
            Destination::Physical(v) => *v,
            Destination::Logical(v) => *v,
        } as u32;

        value << 24
    }
}
