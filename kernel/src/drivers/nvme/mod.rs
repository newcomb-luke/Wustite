use x86_64::{
    PhysAddr,
    structures::paging::{PageTableFlags, PhysFrame},
};

use crate::{logln, memory::MEMORY_MAPPER};

use super::pci::PCIGeneralDevice;

const VERSION_REG: u64 = 0x08;
const ADMIN_SUBMISSION_REG: u64 = 0x28;
const ADMIN_COMPLETION_REG: u64 = 0x30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NVMEDriverError {
    MemoryMappingFailed,
}

struct NVMEQueue {
    address: u64,
    size: u64,
}

impl NVMEQueue {
    fn new(address: u64, size: u64) -> Self {
        Self { address, size }
    }
}

pub struct NVMEDriver {
    device: PCIGeneralDevice,
    base_address: u64,
    capability_stride: u8,
    admin_submission_queue: NVMEQueue,
    admin_completion_queue: NVMEQueue,
}

impl NVMEDriver {
    pub fn new(device: PCIGeneralDevice) -> Result<Self, NVMEDriverError> {
        let base_address = ((device.bar1() as u64) << 32) | (device.bar0() & 0xFFFFFFF0) as u64;
        let capability_stride = ((base_address >> 12) & 0xF) as u8;

        logln!("NVMe base address: {:016x}", base_address);
        logln!("NVMe capability stride: {:02x}", capability_stride);

        unsafe {
            MEMORY_MAPPER
                .identity_map(
                    PhysFrame::from_start_address(PhysAddr::new(base_address)).unwrap(),
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                )
                .map_err(|_| NVMEDriverError::MemoryMappingFailed)?;
        }

        logln!("Successfully mapped NVMe base address");

        todo!();

        // let mut driver = Self {
        //     device,
        //     base_address,
        //     capability_stride,
        // };

        // Ok(driver)
    }

    fn read_version(&mut self) -> u32 {
        unsafe { self.read_reg(VERSION_REG) }
    }

    unsafe fn write_reg(&mut self, offset: u64, value: u32) {
        unsafe {
            Self::write_nvme_reg(self.base_address, offset, value)
        }
    }

    unsafe fn read_reg(&mut self, offset: u64) -> u32 {
        unsafe {
            Self::read_nvme_reg(self.base_address, offset)
        }
    }

    fn create_admin_submission_queue() -> Result<NVMEQueue, NVMEDriverError> {
        todo!()
    }

    unsafe fn write_nvme_reg(base_address: u64, offset: u64, value: u32) {
        let register_ptr = (base_address + offset) as *mut u32;
        unsafe {
            register_ptr.write_volatile(value);
        }
    }

    unsafe fn read_nvme_reg(base_address: u64, offset: u64) -> u32 {
        let register_ptr = (base_address + offset) as *mut u32;
        unsafe { register_ptr.read_volatile() }
    }
}
