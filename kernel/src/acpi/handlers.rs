#![allow(unused_variables)]

use core::ptr::NonNull;

use acpi::{AcpiHandler, PhysicalMapping};
use aml::Handler;
use x86_64::{PhysAddr, structures::paging::PageTableFlags};

use crate::{
    drivers::{
        pci::{PCI_SUBSYSTEM, PCIAddress},
        read_io_port_u8, read_io_port_u16, read_io_port_u32, write_io_port_u8, write_io_port_u16,
        write_io_port_u32,
    },
    memory::MEMORY_MAPPER,
};

pub struct KernelAmlHandler;

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

    fn read_pci_u8(&self, _segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u8 {
        PCI_SUBSYSTEM.pci_config_read_u8(PCIAddress::function(bus, device, function), offset as u8)
    }

    fn read_pci_u16(&self, _segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u16 {
        PCI_SUBSYSTEM.pci_config_read_u16(PCIAddress::function(bus, device, function), offset as u8)
    }

    fn read_pci_u32(&self, _segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u32 {
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
pub struct KernelAcpiHandler;

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
