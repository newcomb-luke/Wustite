use common::{
    DRIVE_NUM_ADDR, MEMORY_REGIONS_DESCRIPTOR_ADDR, MEMORY_REGIONS_START_ADDR,
    PHYS_MAP_VIRTUAL_OFFSET,
};

use crate::{entry::memory::MemoryRegion, hlt_loop, keprintln};
use core::panic::PanicInfo;

#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    crate::init();

    let num_memory_regions =
        unsafe { (MEMORY_REGIONS_DESCRIPTOR_ADDR as *const u8).read() as usize };
    let boot_drive = unsafe { (DRIVE_NUM_ADDR as *const u8).read() };

    let mut memory_regions = [MemoryRegion::zero(); 256];

    for i in 0..num_memory_regions {
        let entry_ptr = unsafe { (MEMORY_REGIONS_START_ADDR as *const u64).add(i * 2) };

        memory_regions[i] = MemoryRegion::read_from(entry_ptr);
    }

    let memory_regions_slice =
        core::slice::from_raw_parts(memory_regions.as_ptr(), num_memory_regions);

    let boot_info = BootInfo {
        memory_regions: memory_regions_slice,
        boot_drive,
        physical_memory_offset: PHYS_MAP_VIRTUAL_OFFSET,
    };

    crate::main(&boot_info);

    hlt_loop();
}

pub struct BootInfo {
    pub memory_regions: &'static [MemoryRegion],
    pub boot_drive: u8,
    pub physical_memory_offset: u64,
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    keprintln!("{}", info);
    hlt_loop();
}
