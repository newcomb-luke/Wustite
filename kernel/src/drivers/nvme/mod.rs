use core::alloc::Layout;

use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{PageTableFlags, PhysFrame},
};

use crate::{allocator::ALLOCATOR, drivers::pci::PCI_SUBSYSTEM, logln, memory::MEMORY_MAPPER};

use super::pci::PCIGeneralDevice;

const VERSION_REG: u64 = 0x08;
const CONTROLLER_CONFIG_REG: u64 = 0x14;
const ADMIN_QUEUE_REG: u64 = 0x24;
const ADMIN_SUBMISSION_REG: u64 = 0x28;
const ADMIN_COMPLETION_REG: u64 = 0x30;

const QUEUE_SIZE_BYTES: u64 = 4096;
const SUBMISSION_QUEUE_ENTRY_SIZE: u64 = 64;
const COMPLETION_QUEUE_ENTRY_SIZE: u64 = 16;
const SUBMISSION_QUEUE_SIZE_ENTRIES: u64 = QUEUE_SIZE_BYTES / SUBMISSION_QUEUE_ENTRY_SIZE;
const COMPLETION_QUEUE_SIZE_ENTRIES: u64 = QUEUE_SIZE_BYTES / COMPLETION_QUEUE_ENTRY_SIZE;

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
    pub fn new(mut device: PCIGeneralDevice) -> DriverResult<Self> {
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

        // Enable bus mastering, memory space, and I/O space
        PCI_SUBSYSTEM.send_command(&mut device, 0b111);

        // Reset the controller
        Self::set_controller_enabled(base_address, false);

        let (admin_submission_queue, admin_completion_queue) =
            Self::create_admin_queues(base_address)?;

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

    fn allocate_nvme_page(size: usize) -> DriverResult<(VirtAddr, PhysAddr)> {
        let virt_ptr = ALLOCATOR
            .lock()
            .allocate_first_fit(
                Layout::from_size_align(size, 32)
                    .map_err(|_| NVMEDriverError::MemoryMappingFailed)?,
            )
            .map_err(|_| NVMEDriverError::MemoryMappingFailed)?;

        let virt_addr = VirtAddr::from_ptr(virt_ptr.as_ptr());

        let phys_addr = unsafe {
            MEMORY_MAPPER
                .virt_to_phys(VirtAddr::new(virt_ptr.as_ptr() as u64))
                .map_err(|_| NVMEDriverError::MemoryMappingFailed)
        }?;

        Ok((virt_addr, phys_addr))
    }

    fn create_admin_queues(base_address: u64) -> DriverResult<(NVMEQueue, NVMEQueue)> {
        unsafe {
            let submission_queue = Self::create_admin_submission_queue(base_address)?;
            let completion_queue = Self::create_admin_completion_queue(base_address)?;

            Self::write_admin_queue_attributes(base_address);

            Ok((submission_queue, completion_queue))
        }
    }

    unsafe fn create_admin_submission_queue(base_address: u64) -> DriverResult<NVMEQueue> {
        let (virt_addr, phys_addr) = Self::allocate_nvme_page(QUEUE_SIZE_BYTES as usize)?;

        unsafe {
            Self::write_nvme_reg(base_address, ADMIN_SUBMISSION_REG, phys_addr.as_u64() as u32);
        }

        Ok(NVMEQueue::new(virt_addr.as_u64(), QUEUE_SIZE_BYTES))
    }

    unsafe fn create_admin_completion_queue(base_address: u64) -> DriverResult<NVMEQueue> {
        let (virt_addr, phys_addr) = Self::allocate_nvme_page(QUEUE_SIZE_BYTES as usize)?;

        unsafe {
            Self::write_nvme_reg(base_address, ADMIN_COMPLETION_REG, phys_addr.as_u64() as u32);
        }

        Ok(NVMEQueue::new(virt_addr.as_u64(), QUEUE_SIZE_BYTES))
    }

    unsafe fn write_admin_queue_attributes(base_address: u64) {
        let value =
            ((COMPLETION_QUEUE_SIZE_ENTRIES - 1) << 16 | (SUBMISSION_QUEUE_ENTRY_SIZE - 1)) as u32;

        unsafe {
            Self::write_nvme_reg(base_address, ADMIN_QUEUE_REG, value);
        }
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
