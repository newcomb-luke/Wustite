#![allow(non_snake_case)]

use handlers::{
    breakpoint_handler, double_fault_handler, general_protection_handler, nmi_handler,
    page_fault_handler, spurious_interrupt_handler,
};
use lazy_static::lazy_static;
use paste::paste;
use vectors::{LOGICAL_TO_VECTOR_MAP, VECTOR_TO_LOGICAL_MAP};
use x86_64::structures::idt::InterruptDescriptorTable;

mod gsi;
mod handlers;
pub mod io_apic;
mod legacy;
pub mod local_apic;
mod logical;
mod vectors;

pub use gsi::*;
pub use logical::*;

use crate::{kprintln, logln};

macro_rules! normal_interrupt {
    ($idt: ident, $vector: literal) => {
        paste! {
            $idt[$vector].set_handler_fn([<handle_normal_interrupt_ $vector>]);
        }
    };
}

macro_rules! normal_interrupt_handler {
    ($vector: literal) => {
        paste! {
            extern "x86-interrupt" fn [<handle_normal_interrupt_ $vector>](_stack_frame: x86_64::structures::idt::InterruptStackFrame) {
                normal_interrupt($vector);
                crate::interrupts::local_apic::acknowledge_interrupt();
            }
        }
    };
}

macro_rules! register_normal_interrupts {
    ($idt: ident, $($vector: literal),*) => {
        $(normal_interrupt!($idt, $vector);)*
    };
}

macro_rules! define_normal_interrupt_handlers {
    ($($vector: literal),*) => {
        $(normal_interrupt_handler!($vector);)*
    };
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.non_maskable_interrupt.set_handler_fn(nmi_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.general_protection_fault
            .set_handler_fn(general_protection_handler);

        register_normal_interrupts!(
            idt, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C,
            0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A,
            0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48,
            0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56,
            0x57, 0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F
        );

        idt[0xFF].set_handler_fn(spurious_interrupt_handler);
        idt
    };
}

define_normal_interrupt_handlers!(
    0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F,
    0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F,
    0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F,
    0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum IrqResult {
    NotHandled,
    Handled,
}

fn normal_interrupt(vector: u8) {
    if let Some(logical) = VECTOR_TO_LOGICAL_MAP.get_entry(Vector::from_u8(vector)) {
        if let Some(entry) = LOGICAL_IRQ_MAPPING_TABLE.get_entry(logical) {
            match entry.call(logical) {
                IrqResult::NotHandled => {
                    unimplemented!();
                }
                IrqResult::Handled => {
                    return;
                }
            }
        }
    } else {
        logln!(
            "[warning] Spurious interrupt on LAPIC vector 0x{:02x} detected!",
            vector
        )
    }
}

pub type IrqHandler<T> = extern "C" fn(&'static T, irq: LogicalIrq) -> IrqResult;
pub type ErasedIrqHandler = extern "C" fn(*const (), irq: LogicalIrq) -> IrqResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vector(u8);

impl Vector {
    pub fn from_u8(vector: u8) -> Self {
        Self(vector)
    }

    pub fn as_u8(&self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GSI(u8);

impl GSI {
    pub fn from_u8(gsi: u8) -> Self {
        Self(gsi)
    }

    pub fn as_u8(&self) -> u8 {
        self.0
    }
}

pub fn init() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        legacy::initialize_legacy_pics();
    });
    IDT.load();
}

pub fn create_irq_mapping(gsi: GSI) -> Result<LogicalIrq, ()> {
    if let Some(logical_irq) = GSI_MAPPING_TABLE.get_entry(gsi) {
        return Ok(logical_irq);
    }

    let logical_irq = LOGICAL_IRQ_MAPPING_TABLE.next_free_irq().ok_or(())?;

    kprintln!(
        "Created new interrupt mapping from GSI {} to logical IRQ {}",
        gsi.as_u8(),
        logical_irq.as_u8()
    );

    GSI_MAPPING_TABLE.set_entry(gsi, logical_irq);

    Ok(logical_irq)
}

pub fn assign_irq_vector(irq: LogicalIrq) -> Result<Vector, ()> {
    if let Some(vector) = LOGICAL_TO_VECTOR_MAP.get_entry(irq) {
        return Ok(vector);
    }

    let vector = VECTOR_TO_LOGICAL_MAP.next_free_vector().ok_or(())?;

    kprintln!(
        "Assigned logical IRQ {} to LAPIC interrupt vector 0x{:02x}",
        irq.as_u8(),
        vector.as_u8()
    );

    VECTOR_TO_LOGICAL_MAP.set_entry(vector, irq);
    LOGICAL_TO_VECTOR_MAP.set_entry(irq, vector);

    Ok(vector)
}
