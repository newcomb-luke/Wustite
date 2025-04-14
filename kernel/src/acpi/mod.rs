use core::ptr::NonNull;

use acpi::{
    AcpiHandler, AcpiTables, InterruptModel, PhysicalMapping, platform::interrupt::NmiProcessor,
};
use alloc::boxed::Box;
use aml::{AmlContext, Handler};
use common::BootInfo;
use x86_64::{PhysAddr, structures::paging::PageTableFlags};

use crate::{
    drivers::{
        pci::{PCI_SUBSYSTEM, PCIAddress},
        read_io_port_u8, read_io_port_u16, read_io_port_u32, write_io_port_u8, write_io_port_u16,
        write_io_port_u32,
    },
    interrupts::{
        io_apic::{DeliveryMode, Destination, PinPolarity, TriggerMode},
        local_apic::LocalInterrupt,
    },
    logln,
    memory::MEMORY_MAPPER,
};

pub fn init_acpi(boot_info: &BootInfo) {
    logln!("[info] Initializaing ACPI");

    let tables = unsafe {
        AcpiTables::from_rsdp(KernelAcpiHandler, boot_info.acpi_rsdp_address as usize).unwrap()
    };

    let dsdt_table = tables.dsdt().unwrap();

    let mut aml = AmlContext::new(Box::new(KernelAmlHandler), aml::DebugVerbosity::All);

    let dsdt_address = unsafe {
        MEMORY_MAPPER
            .phys_to_virt(PhysAddr::new(dsdt_table.address as u64))
            .unwrap()
    };

    let dsdt_table_slice =
        unsafe { core::slice::from_raw_parts(dsdt_address.as_ptr(), dsdt_table.length as usize) };

    aml.parse_table(dsdt_table_slice).unwrap();
    aml.initialize_objects().unwrap();

    logln!("[info] ACPI initialized");

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
                0x30 + source_override.isa_source,
                DeliveryMode::Fixed,
                trigger_mode,
                PinPolarity::ActiveHigh,
                false,
                Destination::Physical(0),
            );
        }
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

    //     if let Ok(adr) = aml.invoke_method(&adr_path, aml::value::Args::EMPTY) {
    //         logln!("{}: {:016x}", path, adr.as_integer(&mut aml).unwrap());
    //     } else if let Ok(hid) = aml.invoke_method(&hid_path, aml::value::Args::EMPTY) {
    //         logln!("{}: {:?}", path, hid);
    //     } else {
    //         logln!("{}", path);
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

    // let resources = resource_descriptor_list(&linka_crs).unwrap();

    // logln!("{:#?}", resources);
}

struct KernelAmlHandler;

impl Handler for KernelAmlHandler {
    fn read_u8(&self, address: usize) -> u8 {
        read_memory(address)
    }

    fn read_u16(&self, address: usize) -> u16 {
        read_memory(address)
    }

    fn read_u32(&self, address: usize) -> u32 {
        read_memory(address)
    }

    fn read_u64(&self, address: usize) -> u64 {
        read_memory(address)
    }

    fn write_u8(&mut self, address: usize, value: u8) {
        unimplemented!()
    }

    fn write_u16(&mut self, address: usize, value: u16) {
        unimplemented!()
    }

    fn write_u32(&mut self, address: usize, value: u32) {
        unimplemented!()
    }

    fn write_u64(&mut self, address: usize, value: u64) {
        unimplemented!()
    }

    fn read_io_u8(&self, port: u16) -> u8 {
        unsafe { read_io_port_u8(port) }
    }

    fn read_io_u16(&self, port: u16) -> u16 {
        unsafe { read_io_port_u16(port) }
    }

    fn read_io_u32(&self, port: u16) -> u32 {
        unsafe { read_io_port_u32(port) }
    }

    fn write_io_u8(&self, port: u16, value: u8) {
        unsafe {
            write_io_port_u8(port, value);
        }
    }

    fn write_io_u16(&self, port: u16, value: u16) {
        unsafe {
            write_io_port_u16(port, value);
        }
    }

    fn write_io_u32(&self, port: u16, value: u32) {
        unsafe {
            write_io_port_u32(port, value);
        }
    }

    fn read_pci_u8(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u8 {
        PCI_SUBSYSTEM.pci_config_read_u8(PCIAddress::function(bus, device, function), offset as u8)
    }

    fn read_pci_u16(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u16 {
        PCI_SUBSYSTEM.pci_config_read_u16(PCIAddress::function(bus, device, function), offset as u8)
    }

    fn read_pci_u32(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u32 {
        PCI_SUBSYSTEM.pci_config_read_u32(PCIAddress::function(bus, device, function), offset as u8)
    }

    fn write_pci_u8(
        &self,
        segment: u16,
        bus: u8,
        device: u8,
        function: u8,
        offset: u16,
        value: u8,
    ) {
        unimplemented!()
    }

    fn write_pci_u16(
        &self,
        segment: u16,
        bus: u8,
        device: u8,
        function: u8,
        offset: u16,
        value: u16,
    ) {
        unimplemented!()
    }

    fn write_pci_u32(
        &self,
        segment: u16,
        bus: u8,
        device: u8,
        function: u8,
        offset: u16,
        value: u32,
    ) {
        unimplemented!()
    }

    fn sleep(&self, milliseconds: u64) {
        unimplemented!()
    }

    fn stall(&self, microseconds: u64) {
        unimplemented!()
    }
}

fn read_memory<T>(address: usize) -> T
where
    T: Copy,
{
    unsafe {
        let virt_addr = MEMORY_MAPPER
            .phys_to_virt(PhysAddr::new(address as u64))
            .unwrap();

        virt_addr.as_ptr::<T>().read()
    }
}

#[derive(Clone)]
struct KernelAcpiHandler;

impl AcpiHandler for KernelAcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> ::acpi::PhysicalMapping<Self, T> {
        let num_pages = size.div_ceil(4096);

        unsafe {
            let virt_addr = MEMORY_MAPPER
                .map_virt_page(
                    PhysAddr::new(physical_address as u64),
                    PageTableFlags::PRESENT,
                )
                .unwrap();

            if num_pages > 1 {
                for page in 1..num_pages {
                    MEMORY_MAPPER
                        .map_virt_page(
                            PhysAddr::new((physical_address + 4096 * page) as u64),
                            PageTableFlags::PRESENT,
                        )
                        .unwrap();
                }
            }

            PhysicalMapping::new(
                physical_address,
                NonNull::new_unchecked(virt_addr.as_u64() as *mut T),
                size,
                num_pages * 4096,
                KernelAcpiHandler,
            )
        }
    }

    fn unmap_physical_region<T>(region: &::acpi::PhysicalMapping<Self, T>) {}
}
