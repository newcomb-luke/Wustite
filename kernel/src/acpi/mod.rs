use core::str::FromStr;

use acpi::{platform::interrupt::NmiProcessor, AcpiTables, AmlTable, InterruptModel, PciConfigRegions};
use alloc::{boxed::Box, vec::Vec};
use aml::{AmlContext, AmlName, AmlValue, LevelType};
use common::BootInfo;
use devices::{AcpiDevice, acpi_device_from_hid};
use handlers::{KernelAcpiHandler, KernelAmlHandler};
use spin::{Mutex, Once};
use x86_64::PhysAddr;

use crate::{
    interrupts::{
        io_apic::{DeliveryMode, Destination, PinPolarity, TriggerMode},
        local_apic::LocalInterrupt,
    },
    logln,
    memory::MEMORY_MAPPER,
};

mod devices;
mod handlers;

pub static INTERPRETER: Interpreter = Interpreter::new();

pub struct Interpreter {
    aml: Mutex<Once<AmlContext>>
}

impl Interpreter {
    const fn new() -> Self {
        Self {
            aml: Mutex::new(Once::new())
        }
    }

    fn initialize(&self, dsdt_table: AmlTable) {
        logln!("[info] ACPI: Initializing interpreter");

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

        let dsdt_table_slice =
            unsafe { core::slice::from_raw_parts(dsdt_address.as_ptr(), dsdt_table.length as usize) };

        context.parse_table(dsdt_table_slice).unwrap();
        context.initialize_objects().unwrap();

        aml.call_once(|| {
            context
        });

        logln!("[info] ACPI: Interpreter initialized");
    }
}

pub fn init_acpi(boot_info: &BootInfo) {
    logln!("[info] ACPI: Initializing");

    let tables = unsafe {
        AcpiTables::from_rsdp(KernelAcpiHandler, boot_info.acpi_rsdp_address as usize).unwrap()
    };

    INTERPRETER.initialize(tables.dsdt().unwrap());

    logln!("[info] ACPI: Initializing interrupt controllers");

    if let InterruptModel::Apic(interrupt_model) = tables.platform_info().unwrap().interrupt_model {
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
            let trigger_mode = match source_override.trigger_mode {
                acpi::platform::interrupt::TriggerMode::SameAsBus => TriggerMode::Edge,
                acpi::platform::interrupt::TriggerMode::Edge => TriggerMode::Edge,
                acpi::platform::interrupt::TriggerMode::Level => TriggerMode::Level,
            };

            crate::interrupts::io_apic::IO_APIC.set_redirect(
                source_override.global_system_interrupt,
                0x20 + source_override.isa_source,
                DeliveryMode::Fixed,
                trigger_mode,
                PinPolarity::ActiveHigh,
                false,
                Destination::Physical(0),
            );
        }
    }

    if let Ok(_pci_config) = PciConfigRegions::new(&tables) {
        logln!("[info] ACPI: Found MCFG - Root bus is PCIe");
        unimplemented!()
    } else {
        logln!("[info] ACPI: Could not find MCFG - Falling back to legacy PCI");
    }

    // let mut paths = Vec::new();

    // aml.namespace
    //     .traverse(|name, level| {
    //         if level.typ == LevelType::Device {
    //             paths.push(name.clone());
    //         }
    //         Ok(true)
    //     })
    //     .unwrap();

    // for path in paths {
    //     let adr_path = AmlName::from_str("_ADR").unwrap().resolve(&path).unwrap();
    //     let hid_path = AmlName::from_str("_HID").unwrap().resolve(&path).unwrap();

    //     logln!("{}:", path);

    //     if let Ok(adr) = aml.invoke_method(&adr_path, aml::value::Args::EMPTY) {
    //         logln!("   ADR {:016x}", adr.as_integer(&mut aml).unwrap());
    //     }
    //     if let Ok(hid) = aml.invoke_method(&hid_path, aml::value::Args::EMPTY) {
    //         match hid {
    //             aml::AmlValue::Integer(int_val) => {
    //                 if let Some(acpi_device) = acpi_device_from_hid(int_val) {
    //                     logln!("   Device: {:?}", acpi_device);
    //                     logln!("   HID {:016x}", int_val);

    //                     let status_path =
    //                         AmlName::from_str("_STA").unwrap().resolve(&path).unwrap();

    //                     if acpi_device == AcpiDevice::PS2Keyboard
    //                         || acpi_device == AcpiDevice::PS2Mouse
    //                     {
    //                         let status = aml
    //                             .namespace
    //                             .get_by_path(&status_path)
    //                             .unwrap()
    //                             .as_status()
    //                             .unwrap();

    //                         if status.present && status.enabled {
    //                             let crs_path =
    //                                 AmlName::from_str("_CRS").unwrap().resolve(&path).unwrap();

    //                             let value = aml
    //                                 .invoke_method(&crs_path, aml::value::Args::EMPTY)
    //                                 .unwrap();

    //                             let crs = aml::resource::resource_descriptor_list(&value).unwrap();

    //                             for resource in crs {
    //                                 match resource {
    //                                     aml::resource::Resource::Irq(irq) => {
    //                                         let irq_mask = irq.irq;

    //                                         if irq_mask.count_ones() == 1 {
    //                                             let irq_num = irq_mask.trailing_zeros();

    //                                             if crate::interrupts::io_apic::IO_APIC
    //                                                 .is_redirect_set(irq_num)
    //                                             {
    //                                                 panic!(
    //                                                     "ACPI: IO APIC already has redirect entry set for GSI {}",
    //                                                     irq_num
    //                                                 );
    //                                             }

    //                                             // logln!("   Real IRQ: {}", irq_num);
    //                                             // logln!("   Descriptor: {:?}", irq);

    //                                             let trigger = match irq.trigger {
    //                                                 aml::resource::InterruptTrigger::Edge => {
    //                                                     TriggerMode::Edge
    //                                                 }
    //                                                 aml::resource::InterruptTrigger::Level => {
    //                                                     TriggerMode::Level
    //                                                 }
    //                                             };

    //                                             let polarity = match irq.polarity {
    //                                                 aml::resource::InterruptPolarity::ActiveHigh => PinPolarity::ActiveHigh,
    //                                                 aml::resource::InterruptPolarity::ActiveLow => PinPolarity::ActiveLow,
    //                                             };

    //                                             crate::interrupts::io_apic::IO_APIC.set_redirect(
    //                                                 irq_num,
    //                                                 0x20 + (irq_num as u8),
    //                                                 DeliveryMode::Fixed,
    //                                                 trigger,
    //                                                 polarity,
    //                                                 false,
    //                                                 Destination::Physical(0),
    //                                             );
    //                                         } else {
    //                                             // Multiple IRQs are possible
    //                                             unimplemented!();
    //                                         }
    //                                     }
    //                                     _ => {
    //                                         // logln!("{:#?}", resource);
    //                                     }
    //                                 }
    //                             }
    //                         }
    //                     } else if acpi_device == AcpiDevice::PCIInterruptLinkDevice {
    //                         let crs_path =
    //                             AmlName::from_str("_CRS").unwrap().resolve(&path).unwrap();

    //                         let value = aml
    //                             .invoke_method(&crs_path, aml::value::Args::EMPTY)
    //                             .unwrap();

    //                         let crs = aml::resource::resource_descriptor_list(&value).unwrap();

    //                         logln!("{:#?}", crs);
    //                     }
    //                 } else {
    //                     logln!("   HID {:016x}", int_val);
    //                 }
    //             }
    //             aml::AmlValue::String(s_val) => {
    //                 logln!("   HID {}", s_val);
    //             }
    //             _ => unimplemented!(),
    //         }
    //     }
    // }

    // let root_prt = aml
    //     .invoke_method(
    //         &AmlName::from_str("\\_SB.PCI0._PRT").unwrap(),
    //         aml::value::Args::EMPTY,
    //     )
    //     .unwrap();

    // if let AmlValue::Package(entries) = root_prt {
    //     for entry in entries {
    //         if let AmlValue::Package(elements) = entry {
    //             let pci_address = match &elements[0] {
    //                 AmlValue::Integer(v) => v,
    //                 _ => unimplemented!(),
    //             };
    //             let int_pin = match &elements[1] {
    //                 AmlValue::Integer(v) => v,
    //                 _ => unimplemented!(),
    //             };
    //             let source = &elements[2];
    //             let gsi = match &elements[3] {
    //                 AmlValue::Integer(v) => v,
    //                 _ => unimplemented!(),
    //             };

    //             logln!("PCI address: {:016x}", pci_address);
    //             logln!("Interrupt pin: {:016x}", int_pin);
    //             logln!("Source: {:?}", source);
    //             logln!("GSI: {:016x}", gsi);
    //             logln!("----------------------------------------")
    //         }
    //     }
    // }

    // if let Ok((name, handle)) = aml.namespace.search(
    //     &AmlName::from_str("_CRS").unwrap(),
    //     &AmlName::from_str("\\_SB.LNKA").unwrap(),
    // ) {
    //     let value = aml.namespace.get(handle).unwrap();
    //     logln!("{:?}", value);
    //     let crs = aml::resource::resource_descriptor_list(&value);
    //     logln!("{}: {:X?}", name, crs);
    // }

    // let linka_crs = aml
    //     .invoke_method(
    //         &AmlName::from_str("\\_SB.LNKA._CRS").unwrap(),
    //         aml::value::Args::EMPTY,
    //     )
    //     .unwrap();

    // let resources = aml::resource::resource_descriptor_list(&linka_crs).unwrap();

    // logln!("{:#?}", resources);
}
