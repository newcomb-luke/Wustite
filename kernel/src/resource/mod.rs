use kernel::SystemError;
use ports::PORTS_TABLE;

use crate::{
    interrupts::{
        ErasedIrqHandler, GSI, HandlerEntry, IrqHandler, LOGICAL_IRQ_MAPPING_TABLE,
        assign_irq_vector, create_irq_mapping,
    },
    kprintln,
};

mod ports;

pub fn request_port(port: u16) -> Result<(), SystemError> {
    PORTS_TABLE.request_port(port)
}

pub fn request_irq<T>(gsi: GSI, context: &'static T, handler: IrqHandler<T>) -> Result<(), SystemError> {
    let logical_irq = create_irq_mapping(gsi)?;

    assign_irq_vector(logical_irq)?;

    let handler_ptr = handler as *const ();

    let erased_handler: ErasedIrqHandler = unsafe { core::mem::transmute(handler_ptr) };

    let context = context as *const T as usize;

    let entry = HandlerEntry::new(erased_handler, context);

    LOGICAL_IRQ_MAPPING_TABLE.set_entry(logical_irq, entry);

    kprintln!(
        "Assigned interrupt handler to logical IRQ {}",
        logical_irq.as_u8()
    );

    Ok(())
}
