use kernel::SystemError;
use ports::PORTS_TABLE;

use crate::{
    interrupts::{
        ErasedIrqHandler, HandlerEntry, IrqHandler, VIRTUAL_IRQ_MAPPING_TABLE, VirtualIrq,
        assign_irq_vector, create_irq_mapping,
    },
    kprintln,
};

mod ports;

pub fn request_port(port: u16) -> Result<(), SystemError> {
    PORTS_TABLE.request_port(port)
}

pub fn request_irq<T>(
    virtual_irq: VirtualIrq,
    context: &'static T,
    handler: IrqHandler<T>,
) -> Result<(), SystemError> {
    assign_irq_vector(virtual_irq)?;

    let handler_ptr = handler as *const ();

    let erased_handler: ErasedIrqHandler = unsafe { core::mem::transmute(handler_ptr) };

    let context = context as *const T as usize;

    let entry = HandlerEntry::new(erased_handler, context);

    VIRTUAL_IRQ_MAPPING_TABLE.set_entry(virtual_irq, entry);

    kprintln!(
        "Assigned interrupt handler to virtual IRQ {}",
        virtual_irq.as_u8()
    );

    Ok(())
}
