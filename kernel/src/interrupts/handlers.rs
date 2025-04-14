use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};

use crate::{drivers::keyboard::handle_keyboard_interrupt, logln};

pub extern "x86-interrupt" fn ps2_keyboard_handler(_stack_frame: InterruptStackFrame) {
    handle_keyboard_interrupt();

    crate::interrupts::acknowledge_interrupt();
}

pub extern "x86-interrupt" fn spurious_interrupt_handler(stack_frame: InterruptStackFrame) {
    panic!("SPURIOUS INTERRUPT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn nmi_handler(stack_frame: InterruptStackFrame) {
    panic!("NON-MASKABLE INTERRUPT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    logln!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    logln!("EXCEPTION: PAGE FAULT");
    logln!("Accessed Address: {:?}", Cr2::read());
    logln!("Error Code: {:?}", error_code);
    logln!("{:#?}", stack_frame);

    kernel::hlt_loop();
}
