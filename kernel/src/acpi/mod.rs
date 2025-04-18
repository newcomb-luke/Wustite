use core::{f64::INFINITY, str::FromStr};

use acpi::{AcpiTables, AmlTable, InterruptModel, platform::interrupt::NmiProcessor};
use alloc::{boxed::Box, format, string::String};
use aml::{AmlContext, AmlName, AmlValue, resource::Resource, value::Args};
use common::BootInfo;
use handlers::{KernelAcpiHandler, KernelAmlHandler};
use kernel::SystemError;
use spin::{Mutex, Once};
use x86_64::PhysAddr;

use crate::{
    drivers::pci::{InterruptPin, PCIAddress},
    interrupts::{
        GSI, GSI_OVERRIDE_TABLE, GSIOverrideEntry, IrqHandler, Vector, VirtualIrq,
        assign_irq_vector, create_irq_mapping,
        io_apic::{DeliveryMode, Destination, IO_APIC, PinPolarity, TriggerMode},
        local_apic::LocalInterrupt,
    },
    kprintln,
    memory::MEMORY_MAPPER,
    resource::request_irq,
};

mod devices;
mod handlers;

pub static INTERPRETER: Interpreter = Interpreter::new();
pub static ACPI_TABLES: Mutex<Once<AcpiTables<KernelAcpiHandler>>> = Mutex::new(Once::new());

pub struct Interpreter {
    aml: Mutex<Once<AmlContext>>,
}

impl Interpreter {
    const fn new() -> Self {
        Self {
            aml: Mutex::new(Once::new()),
        }
    }

    fn initialize(&self, dsdt_table: AmlTable) {
        kprintln!("ACPI: Initializing interpreter");

        let aml = self.aml.lock();

        if aml.is_completed() {
            panic!("Attempted to initialize ACPI interpreter more than once");
        }

        let mut context = AmlContext::new(Box::new(KernelAmlHandler), aml::DebugVerbosity::All);

        let dsdt_address = unsafe {
            MEMORY_MAPPER
                .phys_to_virt(PhysAddr::new(dsdt_table.address as u64))
                .unwrap()
        };

        let dsdt_table_slice = unsafe {
            core::slice::from_raw_parts(dsdt_address.as_ptr(), dsdt_table.length as usize)
        };

        context.parse_table(dsdt_table_slice).unwrap();
        context.initialize_objects().unwrap();

        aml.call_once(|| context);

        kprintln!("ACPI: Interpreter initialized");
    }
}

pub fn init_acpi(boot_info: &BootInfo) {
    kprintln!("ACPI: Initializing tables");

    let acpi_tables = unsafe {
        AcpiTables::from_rsdp(KernelAcpiHandler, boot_info.acpi_rsdp_address as usize).unwrap()
    };

    INTERPRETER.initialize(acpi_tables.dsdt().unwrap());

    kprintln!("ACPI: Initializing interrupt controllers");

    if let InterruptModel::Apic(interrupt_model) =
        acpi_tables.platform_info().unwrap().interrupt_model
    {
        // We know that this is safe because we get the local apic address straight from the ACPI tables
        unsafe {
            crate::interrupts::local_apic::initialize_local_apic(
                interrupt_model.local_apic_address,
            );
        }

        // The ACPI tables can contain information about sources of NMIs (Non-Maskable Interrupts)
        // We must configure the LAPIC to use that source
        for nmi_config in interrupt_model.local_apic_nmi_lines.iter() {
            // Our boot processor
            if nmi_config.processor == NmiProcessor::All
                || nmi_config.processor == NmiProcessor::ProcessorUid(0)
            {
                // Map to our type
                let local_interrupt = match nmi_config.line {
                    acpi::platform::interrupt::LocalInterruptLine::Lint0 => LocalInterrupt::LInt0,
                    acpi::platform::interrupt::LocalInterruptLine::Lint1 => LocalInterrupt::LInt1,
                };

                // Here in the execution path, the local ACPI must have already been initialized
                unsafe {
                    crate::interrupts::local_apic::configure_nmi(local_interrupt);
                }
            }
        }

        // Get the information about almost definitely the only IOAPIC in the system
        let io_apic = interrupt_model.io_apics.get(0).unwrap();

        crate::interrupts::io_apic::IO_APIC.init(
            io_apic.address as u64,
            io_apic.id,
            io_apic.global_system_interrupt_base,
        );

        // Explicit hard-coded interrupt source overrides for the IO APIC
        // Other device-specific ones can be found elsewhere in the ACPI tables
        for source_override in interrupt_model.interrupt_source_overrides.iter() {
            kprintln!(
                "ACPI: Found interrupt source override for ISA {} to GSI {}",
                source_override.isa_source,
                source_override.global_system_interrupt
            );

            let trigger_mode = match source_override.trigger_mode {
                acpi::platform::interrupt::TriggerMode::SameAsBus => TriggerMode::Edge,
                acpi::platform::interrupt::TriggerMode::Edge => TriggerMode::Edge,
                acpi::platform::interrupt::TriggerMode::Level => TriggerMode::Level,
            };

            let isa = GSI::from_u8(source_override.isa_source);
            let gsi = GSI::from_u8(source_override.global_system_interrupt as u8);

            register_gsi_override(isa, gsi, trigger_mode, PinPolarity::ActiveHigh);
        }
    }

    {
        let tables = ACPI_TABLES.lock();
        tables.call_once(move || acpi_tables);
    }
}

pub fn acpi_pci_get_routing(
    address: PCIAddress,
    interrupt_pin: InterruptPin,
) -> Result<VirtualIrq, SystemError> {
    kprintln!("ACPI: Finding routing for PCI device");

    let gsi = acpi_get_gsi_from_prt(address, interrupt_pin)?;

    let (gsi, vector, trigger, polarity, was_overridden) = get_legacy_irq_parameters(gsi)?;

    if !was_overridden {
        kprintln!(
            "ACPI: Warning! Using Edge/ActiveHigh on GSI {} for a PCI device. This may cause problems.",
            gsi.as_u8()
        );
    }

    IO_APIC.set_redirect(
        gsi,
        vector,
        DeliveryMode::Fixed,
        trigger,
        polarity,
        false,
        Destination::Physical(0),
    )?;

    create_irq_mapping(gsi)
}

fn acpi_get_gsi_from_prt(
    address: PCIAddress,
    interrupt_pin: InterruptPin,
) -> Result<GSI, SystemError> {
    let mut lock = INTERPRETER.aml.lock();
    let interpreter = lock.get_mut().unwrap();

    let root_prt = interpreter
        .invoke_method(
            &AmlName::from_str("\\_SB.PCI0._PRT").map_err(|_| SystemError::ResourceNotFound)?,
            Args::EMPTY,
        )
        .map_err(|_| SystemError::ResourceNotFound)?;

    if let AmlValue::Package(prt_entries) = root_prt {
        for entry in prt_entries {
            if let AmlValue::Package(elements) = entry {
                let address_mask = match &elements[0] {
                    AmlValue::Integer(v) => *v,
                    _ => unimplemented!(),
                };
                let prt_pin = match &elements[1] {
                    AmlValue::Integer(v) => *v,
                    _ => unimplemented!(),
                };

                if !match_pci_device_to_prt(address, interrupt_pin, address_mask, prt_pin) {
                    continue;
                }

                kprintln!("ACPI: Found _PRT entry for PCI device");

                let source = &elements[2];

                let parameters = match &source {
                    AmlValue::String(link_name) => acpi_resolve_link_irq(interpreter, link_name)?,
                    AmlValue::Integer(gsi) => {
                        kprintln!("ACPI: Found GSI {} for PCI device directly in PRT", gsi);

                        GSI::from_u8(*gsi as u8)
                    }
                    _ => {
                        panic!(
                            "ACPI: Some logic is wrong here. AML PRT entry source was neither String nor Integer"
                        );
                    }
                };

                return Ok(parameters);
            } else {
                panic!("ACPI: Some logic is wrong here. AML PRT entry did not contain a Package");
            }
        }
    } else {
        panic!("ACPI: Some logic is wrong here. AML PRT table did not contain a Package");
    }

    Err(SystemError::ResourceNotFound)
}

fn match_pci_device_to_prt(
    address: PCIAddress,
    interrupt_pin: InterruptPin,
    address_mask: u64,
    prt_pin: u64,
) -> bool {
    if interrupt_pin.as_u8() != prt_pin as u8 {
        return false;
    }

    let prt_device = ((address_mask >> 16) & 0xFF) as u8;
    let function_mask = ((address_mask >> 8) & 0xFF) as u8;

    if address.device != prt_device {
        return false;
    }

    // 0xFF is a wildcard which matches any function on a device
    function_mask == 0xFF || address.function == function_mask
}

fn acpi_resolve_link_irq(
    interpreter: &mut AmlContext,
    source: &String,
) -> Result<GSI, SystemError> {
    kprintln!("ACPI: Looking for GSI for PCI link device {}", source);

    let crs_path = AmlName::from_str(&format!("\\_SB.{}._CRS", source)).unwrap();

    kprintln!("Got here");

    let test_path = AmlName::from_str("\\_SB.LNKC._HID").unwrap();
    let test_val = interpreter.invoke_method(&test_path, aml::value::Args::EMPTY);
    kprintln!("TEST _HID result: {:?}", test_val);

    let link_crs = interpreter
        .invoke_method(&crs_path, aml::value::Args::EMPTY)
        .map_err(|_| SystemError::ResourceNotFound)?;

    kprintln!("And got here");

    let resources = aml::resource::resource_descriptor_list(&link_crs).unwrap();

    for resource in resources {
        kprintln!("ACPI: Found resource {:?}", resource);

        match resource {
            Resource::Irq(descriptor) => {
                // let trigger = match descriptor.trigger {
                //     aml::resource::InterruptTrigger::Edge => TriggerMode::Edge,
                //     aml::resource::InterruptTrigger::Level => TriggerMode::Level,
                // };
                // let polarity = match descriptor.polarity {
                //     aml::resource::InterruptPolarity::ActiveHigh => PinPolarity::ActiveHigh,
                //     aml::resource::InterruptPolarity::ActiveLow => PinPolarity::ActiveLow,
                // };

                kprintln!(
                    "ACPI: Found GSI {} for PCI device directly in PCI link {}",
                    descriptor.irq,
                    source
                );

                if descriptor.irq == 0 {
                    panic!("ACPI: Will need to assign IRQ");
                }

                return Ok(GSI::from_u8(descriptor.irq as u8));
            }
            _ => {
                kprintln!("ACPI: Found other: {:?}", resource);
                todo!();
                continue;
            }
        }
    }

    Err(SystemError::ResourceNotFound)
}

pub fn acpi_request_irq<T>(
    isa: GSI,
    context: &'static T,
    handler: IrqHandler<T>,
) -> Result<(), SystemError> {
    let (gsi, vector, trigger, polarity, _) = get_legacy_irq_parameters(isa)?;

    IO_APIC.set_redirect(
        gsi,
        vector,
        DeliveryMode::Fixed,
        trigger,
        polarity,
        false,
        Destination::Physical(0),
    )?;

    let irq = create_irq_mapping(gsi)?;

    request_irq(irq, context, handler)
}

fn get_legacy_irq_parameters(
    isa: GSI,
) -> Result<(GSI, Vector, TriggerMode, PinPolarity, bool), SystemError> {
    Ok(
        if let Some(gsi_override) = GSI_OVERRIDE_TABLE.check_override(isa) {
            // An override existed in the ACPI tables

            let logical_irq = create_irq_mapping(gsi_override.gsi())?;
            let vector = assign_irq_vector(logical_irq)?;

            (
                gsi_override.gsi(),
                vector,
                gsi_override.trigger(),
                gsi_override.polarity(),
                true,
            )
        } else {
            // No override existed in the ACPI tables, just map it directly

            let logical_irq = create_irq_mapping(isa)?;
            let vector = assign_irq_vector(logical_irq)?;

            let (trigger, polarity) = if isa.as_u8() < 16 {
                // This is an ISA standard IRQ line, and should have those defaults
                (TriggerMode::Edge, PinPolarity::ActiveHigh)
            } else {
                // This is a non-ISA (probably PCI) IRQ line
                (TriggerMode::Level, PinPolarity::ActiveLow)
            };

            (isa, vector, trigger, polarity, false)
        },
    )
}

fn register_gsi_override(isa: GSI, gsi: GSI, trigger: TriggerMode, polarity: PinPolarity) {
    GSI_OVERRIDE_TABLE.add_override(isa, GSIOverrideEntry::new(gsi, trigger, polarity));
}
