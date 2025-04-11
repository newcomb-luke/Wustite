use core::{alloc::Layout, ptr::NonNull};

use x86_64::{
    PhysAddr,
    structures::paging::{PageSize, PageTableFlags, PhysFrame},
};

use crate::{allocator::ALLOCATOR, logln, memory::MEMORY_MAPPER};

use super::pci::PCIGeneralDevice;

const VERSION_REG: u64 = 0x08;
const CONTROLLER_CONFIG_REG: u64 = 0x14;
const ADMIN_SUBMISSION_REG: u64 = 0x28;
const ADMIN_COMPLETION_REG: u64 = 0x30;

const QUEUE_SIZE: u64 = 4096;

type DriverResult<T> = Result<T, NVMEDriverError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NVMEDriverError {
    MemoryMappingFailed,
}

#[derive(Debug, Clone, Copy)]
struct NVMEQueue {
    address: u64,
    size: u64,
}

impl NVMEQueue {
    fn new(address: u64, size: u64) -> Self {
        Self { address, size }
    }
}

#[derive(Debug, Clone, Copy)]
struct AdminSubmissionQueueEntry {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AdminOpcode {
    CreateIOSubmissionQueue,
    CreateIOCompletionQueue,
    Identify,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IOOpcode {
    Read,
    Write,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FusedOperation {
    Normal,
    FirstCommand,
    SecondCommand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Location {
    PRP,
    SGL,
}

#[derive(Debug, Clone, Copy)]
struct AdminCommand {
    opcode: AdminOpcode,
    fused: FusedOperation,
    location: Location,
}

#[derive(Debug)]
pub struct NVMEDriver {
    device: PCIGeneralDevice,
    base_address: u64,
    capability_stride: u8,
    admin_submission_queue: NVMEQueue,
    admin_completion_queue: NVMEQueue,
}

impl NVMEDriver {
    pub fn new(device: PCIGeneralDevice) -> DriverResult<Self> {
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

        // Reset the controller
        Self::set_controller_enabled(base_address, false);

        let admin_submission_queue = Self::create_admin_submission_queue(base_address)?;
        let admin_completion_queue = Self::create_admin_completion_queue(base_address)?;

        logln!(
            "Admin submission queue created: addr {:08x}, size {}",
            admin_submission_queue.address,
            admin_submission_queue.size
        );
        logln!(
            "Admin completion queue created: addr {:08x}, size {}",
            admin_completion_queue.address,
            admin_completion_queue.size
        );

        // Start the controller
        Self::set_controller_enabled(base_address, true);

        logln!("Interrupt pin: {}", device.interrupt_pin());
        logln!("Interrupt line: {}", device.interrupt_line());

        let driver = Self {
            device,
            base_address,
            capability_stride,
            admin_submission_queue,
            admin_completion_queue,
        };

        Ok(driver)
    }

    fn read_version(&mut self) -> u32 {
        unsafe { self.read_reg(VERSION_REG) }
    }

    unsafe fn write_reg(&mut self, offset: u64, value: u32) {
        unsafe { Self::write_nvme_reg(self.base_address, offset, value) }
    }

    unsafe fn read_reg(&mut self, offset: u64) -> u32 {
        unsafe { Self::read_nvme_reg(self.base_address, offset) }
    }

    fn set_controller_enabled(base_address: u64, enabled: bool) {
        let data = if enabled { 1 } else { 0 };
        unsafe {
            Self::write_nvme_reg(base_address, CONTROLLER_CONFIG_REG, data);
        }
    }

    fn allocate_nvme_page(size: usize) -> DriverResult<NonNull<u8>> {
        ALLOCATOR
            .lock()
            .allocate_first_fit(
                Layout::from_size_align(size, 32)
                    .map_err(|_| NVMEDriverError::MemoryMappingFailed)?,
            )
            .map_err(|_| NVMEDriverError::MemoryMappingFailed)
    }

    fn create_admin_submission_queue(base_address: u64) -> DriverResult<NVMEQueue> {
        let address = Self::allocate_nvme_page(QUEUE_SIZE as usize)?.as_ptr() as u64;

        unsafe {
            Self::write_nvme_reg(base_address, ADMIN_SUBMISSION_REG, address as u32);
        }

        Ok(NVMEQueue::new(address, QUEUE_SIZE))
    }

    fn create_admin_completion_queue(base_address: u64) -> DriverResult<NVMEQueue> {
        let address = Self::allocate_nvme_page(QUEUE_SIZE as usize)?.as_ptr() as u64;

        unsafe {
            Self::write_nvme_reg(base_address, ADMIN_COMPLETION_REG, address as u32);
        }

        Ok(NVMEQueue::new(address, QUEUE_SIZE))
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
