use handlers::{
    breakpoint_handler, double_fault_handler, general_protection_handler, nmi_handler, page_fault_handler, ps2_keyboard_handler, ps2_mouse_handler, spurious_interrupt_handler
};
use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;
use core::arch::asm;

mod handlers;
pub mod io_apic;
mod legacy;
pub mod local_apic;

pub use local_apic::acknowledge_interrupt;

use crate::{log, logln};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InterruptIndex {
    Ps2Keyboard = 0x31,
    Ps2Mouse = 0x3C,
    SpuriousInterrupt = 0xFF,
}

impl InterruptIndex {
    fn as_u8(&self) -> u8 {
        *self as u8
    }
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
        idt.general_protection_fault.set_handler_fn(general_protection_handler);
        idt[0x30].set_handler_fn(debug_handler);
        idt[InterruptIndex::Ps2Keyboard.as_u8()].set_handler_fn(ps2_keyboard_handler);
        idt[InterruptIndex::Ps2Mouse.as_u8()].set_handler_fn(ps2_mouse_handler);
        idt[InterruptIndex::SpuriousInterrupt.as_u8()].set_handler_fn(spurious_interrupt_handler);
        idt
    };
}

pub extern "x86-interrupt" fn debug_handler(_stack_frame: x86_64::structures::idt::InterruptStackFrame) {
    // log!(".");

    crate::interrupts::local_apic::acknowledge_interrupt();
}

pub fn init() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        legacy::initialize_legacy_pics();
    });
    IDT.load();
}
