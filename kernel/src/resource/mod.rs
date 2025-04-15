use ports::{PortError, PORTS_TABLE};

use crate::interrupts::{ErasedIrqHandler, HandlerEntry, IrqHandler, LOGICAL_IRQ_MAPPING_TABLE};

mod ports;

pub fn request_port(port: u16) -> Result<(), PortError> {
    PORTS_TABLE.request_port(port)
}

pub fn request_irq<T>(irq: u8, context: &'static T, handler: IrqHandler<T>) -> Result<(), ()> {
    let logical_irq = LOGICAL_IRQ_MAPPING_TABLE.next_free_irq().ok_or(())?;

    let handler_ptr = handler as *const ();

    let erased_handler: ErasedIrqHandler = unsafe {
        core::mem::transmute(handler_ptr)
    };

    let context = context as *const T as usize;

    let entry = HandlerEntry::new(erased_handler, context);

    LOGICAL_IRQ_MAPPING_TABLE.set_entry(logical_irq, entry);
    Err(())
}
