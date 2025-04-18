use core::alloc::Layout;

use alloc::boxed::Box;
use commands::{AdminCommand, CreateIOCompletionQueueCommand, CreateIOSubmissionQueueCommand};
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{PageTableFlags, PhysFrame},
};

use crate::{
    allocator::ALLOCATOR,
    drivers::pci::{BUS_MASTER_ENABLE, IO_SPACE_ENABLE, MEMORY_SPACE_ENABLE, PCI_SUBSYSTEM},
    kprintln,
    memory::MEMORY_MAPPER,
};

use super::{
    DriverResult,
    pci::{PCIDevice, PCIDeviceClass, PCIDeviceSetup},
    register::{PCIDriver, PCIDriverCreator},
};

mod commands;

const VERSION_REG: u64 = 0x08;
const CONTROLLER_CAPABILITIES_REG: u64 = 0x00;
const CONTROLLER_CONFIG_REG: u64 = 0x14;
const CONTROLLER_STATUS_REG: u64 = 0x1C;
const ADMIN_QUEUE_REG: u64 = 0x24;
const ADMIN_SUBMISSION_REG: u64 = 0x28;
const ADMIN_COMPLETION_REG: u64 = 0x30;

const QUEUE_SIZE_BYTES: u64 = 4096;
const SUBMISSION_QUEUE_ENTRY_SIZE: u64 = 64;
const SUBMISSION_QUEUE_ENTRY_SIZE_POW_2: u8 = SUBMISSION_QUEUE_ENTRY_SIZE.ilog2() as u8;
const COMPLETION_QUEUE_ENTRY_SIZE: u64 = 16;
const COMPLETION_QUEUE_ENTRY_SIZE_POW_2: u8 = COMPLETION_QUEUE_ENTRY_SIZE.ilog2() as u8;
const SUBMISSION_QUEUE_SIZE_ENTRIES: u64 = QUEUE_SIZE_BYTES / SUBMISSION_QUEUE_ENTRY_SIZE;
const COMPLETION_QUEUE_SIZE_ENTRIES: u64 = QUEUE_SIZE_BYTES / COMPLETION_QUEUE_ENTRY_SIZE;

#[derive(Debug, Clone, Copy)]
struct NVMEQueue {
    id: u16,
    address: u64,
    size: u64,
    tail: u32,
    doorbell_address: u64,
}

impl NVMEQueue {
    fn new(id: u16, address: u64, size: u64, doorbell_address: u64) -> Self {
        Self {
            id,
            address,
            size,
            tail: 0,
            doorbell_address,
        }
    }

    fn increment_tail(&mut self) {
        self.tail += 1;

        unsafe {
            let doorbell_ptr = self.doorbell_address as *mut u32;
            doorbell_ptr.write(self.tail);
        }
    }
}

#[derive(Debug)]
struct AdminChannel {
    submission_queue: NVMEQueue,
    completion_queue: NVMEQueue,
}

impl AdminChannel {
    fn send_command(&mut self, command: AdminCommand) {
        let command_base_address = self.submission_queue.address;
        let ptr = command_base_address as *mut u8;
        let command_slice =
            unsafe { core::slice::from_raw_parts_mut(ptr, SUBMISSION_QUEUE_ENTRY_SIZE as usize) };

        command.write(command_slice);

        self.submission_queue.increment_tail();
    }
}

struct ControllerConfiguration {
    io_completion_queue_entry_size: u8,
    io_submission_queue_entry_size: u8,
    enabled: bool,
}

impl ControllerConfiguration {
    fn new(enabled: bool) -> Self {
        Self {
            io_completion_queue_entry_size: COMPLETION_QUEUE_ENTRY_SIZE_POW_2,
            io_submission_queue_entry_size: SUBMISSION_QUEUE_ENTRY_SIZE_POW_2,
            enabled,
        }
    }

    fn as_u32(&self) -> u32 {
        let upper = ((self.io_completion_queue_entry_size as u32) << 20)
            | ((self.io_submission_queue_entry_size as u32) << 16);
        let lower = if self.enabled { 1 } else { 0 };

        upper | lower
    }
}

pub struct NVMEDriverCreator;

impl PCIDriverCreator for NVMEDriverCreator {
    fn detect(&self, device: &super::pci::PCIDevice) -> bool {
        let PCIDevice::General(device) = device;

        device.device_class
            == PCIDeviceClass::MassStorage(super::pci::MassStorageController::NVMController)
    }

    fn create(
        &self,
        setup: super::pci::PCIDeviceSetup,
    ) -> Result<alloc::boxed::Box<dyn super::register::PCIDriver>, kernel::SystemError> {
        Ok(Box::new(NVMEDriver::new(setup)?))
    }
}

pub struct NVMEDriver {
    setup: PCIDeviceSetup,
    base_address: u64,
    capability_stride: u8,
    admin_channel: AdminChannel,
    io_submission_queue: NVMEQueue,
    io_completion_queue: NVMEQueue,
}

impl PCIDriver for NVMEDriver {
    fn name(&self) -> &'static str {
        "NVMe Driver"
    }
}

impl NVMEDriver {
    pub fn new(mut setup: PCIDeviceSetup) -> DriverResult<Self> {
        let PCIDevice::General(device) = &mut setup.device;

        let base_address = ((device.bar1() as u64) << 32) | (device.bar0() & 0xFFFFFFF0) as u64;
        let capability_stride = ((base_address >> 12) & 0xF) as u8;

        kprintln!("NVMe base address: {:016x}", base_address);
        kprintln!("NVMe capability stride: {:02x}", capability_stride);

        unsafe {
            // First page
            MEMORY_MAPPER
                .identity_map(
                    PhysFrame::from_start_address(PhysAddr::new(base_address)).unwrap(),
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE,
                )
                .map_err(|_| kernel::SystemError::ResourceInvalid)?;
            // Second page
            MEMORY_MAPPER
                .identity_map(
                    PhysFrame::from_start_address(PhysAddr::new(base_address + 0x1000)).unwrap(),
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE,
                )
                .map_err(|_| kernel::SystemError::ResourceInvalid)?;
        }

        kprintln!("Successfully mapped NVMe base address");

        let controller_capabilities =
            unsafe { Self::read_nvme_reg_64(base_address, CONTROLLER_CAPABILITIES_REG) };

        let max_queue_size = (controller_capabilities & 0xFFFF) as u16;

        kprintln!("Maximum queue entries supported: {}", max_queue_size);

        // Enable bus mastering, memory space, and I/O space
        PCI_SUBSYSTEM.send_command(
            device,
            BUS_MASTER_ENABLE | MEMORY_SPACE_ENABLE | IO_SPACE_ENABLE,
        );

        // Reset the controller
        Self::set_controller_configuration(base_address, ControllerConfiguration::new(false));

        let mut admin_channel = Self::create_admin_queues(base_address, capability_stride)?;

        kprintln!(
            "Admin submission queue created: addr {:08x}, size {}",
            admin_channel.submission_queue.address,
            admin_channel.completion_queue.size
        );
        kprintln!(
            "Admin completion queue created: addr {:08x}, size {}",
            admin_channel.completion_queue.address,
            admin_channel.submission_queue.size
        );

        // Start the controller
        Self::set_controller_configuration(base_address, ControllerConfiguration::new(true));

        Self::wait_until_controller_ready(base_address);

        kprintln!("Interrupt pin: {:?}", device.interrupt_pin());
        kprintln!("Interrupt line: {}", device.interrupt_line());

        let (io_submission_queue, io_completion_queue) =
            Self::create_io_queues(base_address, capability_stride, &mut admin_channel)?;

        let driver = Self {
            setup,
            base_address,
            capability_stride,
            admin_channel,
            io_submission_queue,
            io_completion_queue,
        };

        Ok(driver)
    }

    fn read_version(&mut self) -> u32 {
        unsafe { self.read_reg_32(VERSION_REG) }
    }

    unsafe fn write_reg_32(&mut self, offset: u64, value: u32) {
        unsafe { Self::write_nvme_reg_32(self.base_address, offset, value) }
    }

    unsafe fn read_reg_32(&mut self, offset: u64) -> u32 {
        unsafe { Self::read_nvme_reg_32(self.base_address, offset) }
    }

    fn wait_until_controller_ready(base_address: u64) {
        while Self::read_controller_status(base_address) & 1 == 0 {}
    }

    fn read_controller_status(base_address: u64) -> u32 {
        unsafe { Self::read_nvme_reg_32(base_address, CONTROLLER_STATUS_REG) }
    }

    fn set_controller_configuration(base_address: u64, config: ControllerConfiguration) {
        let value = config.as_u32();
        unsafe {
            Self::write_nvme_reg_32(base_address, CONTROLLER_CONFIG_REG, value);
        }
    }

    fn allocate_nvme_page(size: usize) -> DriverResult<(VirtAddr, PhysAddr)> {
        let virt_ptr = ALLOCATOR
            .lock()
            .allocate_first_fit(
                Layout::from_size_align(size, 4096)
                    .map_err(|_| kernel::SystemError::ResourceInvalid)?,
            )
            .map_err(|_| kernel::SystemError::ResourceInvalid)?;

        let virt_addr = VirtAddr::from_ptr(virt_ptr.as_ptr());

        let phys_addr = unsafe {
            MEMORY_MAPPER
                .virt_to_phys(VirtAddr::new(virt_ptr.as_ptr() as u64))
                .map_err(|_| kernel::SystemError::ResourceInvalid)
        }?;

        Ok((virt_addr, phys_addr))
    }

    fn calculate_doorbell_address(base_address: u64, capability_stride: u8, queue_id: u16) -> u64 {
        let queue_offset = (queue_id as u64) * (4 << (capability_stride as u64));
        base_address + 0x1000 + queue_offset
    }

    fn create_admin_queues(base_address: u64, capability_stride: u8) -> DriverResult<AdminChannel> {
        unsafe {
            let submission_queue =
                Self::create_admin_submission_queue(base_address, capability_stride)?;
            let completion_queue =
                Self::create_admin_completion_queue(base_address, capability_stride)?;

            Self::write_admin_queue_attributes(base_address);

            let channel = AdminChannel {
                submission_queue,
                completion_queue,
            };

            Ok(channel)
        }
    }

    fn create_io_queues(
        base_address: u64,
        capability_stride: u8,
        admin_channel: &mut AdminChannel,
    ) -> DriverResult<(NVMEQueue, NVMEQueue)> {
        unsafe {
            let completion_queue =
                Self::create_io_completion_queue(base_address, capability_stride, admin_channel)?;

            kernel::hlt_loop();

            let completion_queue_id = completion_queue.id;

            let submission_queue = Self::create_io_submission_queue(
                base_address,
                capability_stride,
                completion_queue_id,
                admin_channel,
            )?;
            Ok((submission_queue, completion_queue))
        }
    }

    unsafe fn create_io_submission_queue(
        base_address: u64,
        capability_stride: u8,
        completion_queue_id: u16,
        admin_channel: &mut AdminChannel,
    ) -> DriverResult<NVMEQueue> {
        let (virt_addr, phys_addr) = Self::allocate_nvme_page(QUEUE_SIZE_BYTES as usize)?;

        let queue_id = 1;
        let doorbell_address =
            Self::calculate_doorbell_address(base_address, capability_stride, queue_id);

        let command = CreateIOSubmissionQueueCommand::new(
            phys_addr.as_u64(),
            (SUBMISSION_QUEUE_SIZE_ENTRIES - 1) as u16,
            queue_id,
            completion_queue_id,
        )
        .into_command(2);

        admin_channel.send_command(command);

        Ok(NVMEQueue::new(
            queue_id,
            virt_addr.as_u64(),
            QUEUE_SIZE_BYTES,
            doorbell_address,
        ))
    }

    unsafe fn create_io_completion_queue(
        base_address: u64,
        capability_stride: u8,
        admin_channel: &mut AdminChannel,
    ) -> DriverResult<NVMEQueue> {
        let (virt_addr, phys_addr) = Self::allocate_nvme_page(QUEUE_SIZE_BYTES as usize)?;

        let queue_id = 1;
        let doorbell_address =
            Self::calculate_doorbell_address(base_address, capability_stride, queue_id);

        let command = CreateIOCompletionQueueCommand::new(
            phys_addr.as_u64(),
            (COMPLETION_QUEUE_SIZE_ENTRIES - 1) as u16,
            queue_id,
            true,
        )
        .into_command(1);

        admin_channel.send_command(command);

        Ok(NVMEQueue::new(
            queue_id,
            virt_addr.as_u64(),
            QUEUE_SIZE_BYTES,
            doorbell_address,
        ))
    }

    unsafe fn create_admin_submission_queue(
        base_address: u64,
        capability_stride: u8,
    ) -> DriverResult<NVMEQueue> {
        let (virt_addr, phys_addr) = Self::allocate_nvme_page(QUEUE_SIZE_BYTES as usize)?;

        let queue_id = 0;
        let doorbell_address =
            Self::calculate_doorbell_address(base_address, capability_stride, queue_id);

        unsafe {
            Self::write_nvme_reg_32(
                base_address,
                ADMIN_SUBMISSION_REG,
                phys_addr.as_u64() as u32,
            );
        }

        Ok(NVMEQueue::new(
            0,
            virt_addr.as_u64(),
            QUEUE_SIZE_BYTES,
            doorbell_address,
        ))
    }

    unsafe fn create_admin_completion_queue(
        base_address: u64,
        capability_stride: u8,
    ) -> DriverResult<NVMEQueue> {
        let (virt_addr, phys_addr) = Self::allocate_nvme_page(QUEUE_SIZE_BYTES as usize)?;

        let queue_id = 0;
        let doorbell_address =
            Self::calculate_doorbell_address(base_address, capability_stride, queue_id);

        unsafe {
            Self::write_nvme_reg_32(
                base_address,
                ADMIN_COMPLETION_REG,
                phys_addr.as_u64() as u32,
            );
        }

        Ok(NVMEQueue::new(
            queue_id,
            virt_addr.as_u64(),
            QUEUE_SIZE_BYTES,
            doorbell_address,
        ))
    }

    unsafe fn write_admin_queue_attributes(base_address: u64) {
        let value = ((COMPLETION_QUEUE_SIZE_ENTRIES - 1) << 16
            | (SUBMISSION_QUEUE_SIZE_ENTRIES - 1)) as u32;

        unsafe {
            Self::write_nvme_reg_32(base_address, ADMIN_QUEUE_REG, value);
        }
    }

    unsafe fn write_nvme_reg_32(base_address: u64, offset: u64, value: u32) {
        let register_ptr = (base_address + offset) as *mut u32;
        unsafe {
            register_ptr.write_volatile(value);
        }
    }

    unsafe fn read_nvme_reg_32(base_address: u64, offset: u64) -> u32 {
        let register_ptr = (base_address + offset) as *mut u32;
        unsafe { register_ptr.read_volatile() }
    }

    unsafe fn read_nvme_reg_64(base_address: u64, offset: u64) -> u64 {
        let register_ptr = (base_address + offset) as *mut u64;
        unsafe { register_ptr.read_volatile() }
    }
}
