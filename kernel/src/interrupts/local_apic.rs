#![allow(dead_code)]

use spin::Once;
use x86_64::{PhysAddr, structures::paging::PageTableFlags};

use crate::{kprintln, memory::MEMORY_MAPPER};

// Register offset definitions
const EOI_REG: u16 = 0x0B0;
const LOGICAL_DESTINATION_REG: u16 = 0x0D0;
const DESTINATION_FORMAT_REG: u16 = 0x0E0;
const SPURIOUS_INTERRUPT_VECTOR_REG: u16 = 0x0F0;
const LVT_TIMER_REG: u16 = 0x320;
const LVT_LINT0_REG: u16 = 0x350;
const LVT_LINT1_REG: u16 = 0x360;
const TIMER_INITIAL_COUNT_REG: u16 = 0x380;
const TIMER_CURRENT_COUNT_REG: u16 = 0x390;
const TIMER_DIVIDE_CONFIG_REG: u16 = 0x3E0;

const SIV_ENABLE_BIT: u32 = 0x100;

pub enum LocalInterrupt {
    LInt0,
    LInt1,
}

impl LocalInterrupt {
    fn as_u8(&self) -> u8 {
        match self {
            Self::LInt0 => 0,
            Self::LInt1 => 1,
        }
    }
}

pub struct LocalApic {
    base_address: u64,
}

impl LocalApic {
    fn new(base_address: u64) -> Self {
        Self { base_address }
    }

    unsafe fn write_register(&self, register: u16, value: u32) {
        let ptr = (self.base_address + register as u64) as *mut u32;
        unsafe {
            ptr.write_volatile(value);
        }
    }

    unsafe fn read_register(&self, register: u16) -> u32 {
        let ptr = (self.base_address + register as u64) as *const u32;
        unsafe { ptr.read_volatile() }
    }
}

pub static LOCAL_APIC: Once<LocalApic> = Once::new();

/// The caller must guarantee that this is in fact the proper physical base address for the LAPIC
pub unsafe fn initialize_local_apic(base_address: u64) {
    if LOCAL_APIC.is_completed() {
        panic!("Attempted to initialize LAPIC twice");
    }

    // Map the first page and get the virtual address of the base
    let virt_base_address = unsafe {
        MEMORY_MAPPER
            .map_virt_page(
                PhysAddr::new(base_address),
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE,
            )
            .unwrap()
    };

    // Map a second page too
    unsafe {
        MEMORY_MAPPER
            .map_virt_page(
                PhysAddr::new(base_address + 0x1000),
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE,
            )
            .unwrap();
    }

    LOCAL_APIC.call_once(|| LocalApic::new(virt_base_address.as_u64()));

    kprintln!("LAPIC: Initialized at 0x{:08x}", base_address);
}

/// SAFETY: The local ACPI must have been initialized previously
/// Interrupts must not be enabled
pub unsafe fn enable_local_apic(spurious_interrupt_vector: u8) {
    let value = SIV_ENABLE_BIT | (spurious_interrupt_vector as u32);

    // This is safe because LOCAL_APIC is guaranteed to be initialized according to the caller
    unsafe {
        LOCAL_APIC
            .get()
            .unwrap()
            .write_register(SPURIOUS_INTERRUPT_VECTOR_REG, value);
    }

    kprintln!(
        "LAPIC: Enabled with spurious interrupt vector of 0x{:02x}",
        spurious_interrupt_vector
    );
}

/// SAFETY: The local ACPI must have been initialized previously
pub unsafe fn disable_local_apic() {
    let enable_bit_mask = !SIV_ENABLE_BIT;

    // This is safe because LOCAL_APIC is guaranteed to be initialized according to the caller
    unsafe {
        let local_apic = LOCAL_APIC.get().unwrap();

        let current_value = local_apic.read_register(SPURIOUS_INTERRUPT_VECTOR_REG);
        let new_value = current_value & enable_bit_mask;

        local_apic.write_register(SPURIOUS_INTERRUPT_VECTOR_REG, new_value);
    }

    kprintln!("LAPIC: Disabled");
}

/// SAFETY: The local ACPI must have been initialized previously
/// Interrupts must not be enabled
pub unsafe fn configure_nmi(local_interrupt: LocalInterrupt) {
    const NMI_DELIVERY: u32 = 0b100 << 8;

    let reg = match local_interrupt {
        LocalInterrupt::LInt0 => LVT_LINT0_REG,
        LocalInterrupt::LInt1 => LVT_LINT1_REG,
    };

    // This is safe because reg is a controlled variable, and LOCAL_APIC is guaranteed to be initialized according to the caller
    unsafe {
        LOCAL_APIC.wait().write_register(reg, NMI_DELIVERY);
    }

    kprintln!(
        "LAPIC: Local interrupt {} set as NMI",
        local_interrupt.as_u8()
    );
}

pub fn acknowledge_interrupt() {
    unsafe {
        LOCAL_APIC.get().unwrap().write_register(EOI_REG, 0);
    }
}
