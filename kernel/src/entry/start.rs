use crate::{
    entry::memory::{MemoryRegion, MemoryRegionKind},
    keprintln,
};
use core::panic::PanicInfo;

const BOOT_DRIVE_NUMBER_PTR: *const u32 = 0x00070000 as *const u32;
const NUM_SMAP_ENTRIES_PTR: *const u32 = 0x00070004 as *const u32;
const START_SMAP_ENTRIES_PTR: *const RawMemoryRegion = 0x00070008 as *const RawMemoryRegion;
const MAX_SMAP_ENTRIES: usize = 50;

#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    let num_reported_memory_regions = *NUM_SMAP_ENTRIES_PTR;
    let boot_drive = *BOOT_DRIVE_NUMBER_PTR;

    let mut num_memory_regions = 0;
    let memory_regions_start = 0x00000004 as *mut MemoryRegion;
    let mut next_memory_region = memory_regions_start;
    let mut next_raw_memory_region = START_SMAP_ENTRIES_PTR;

    // This is actually funnily enough just where we are storing this table...
    // so reserve it!
    add_memory_region(
        &mut num_memory_regions,
        &mut next_memory_region,
        MemoryRegion {
            start: 0x0000,
            end: 0x500,
            kind: MemoryRegionKind::Reserved,
        },
    );
    // The place where the bootloader has placed the GDT
    // In reality this is just the entire second stage bootloader
    add_memory_region(
        &mut num_memory_regions,
        &mut next_memory_region,
        MemoryRegion {
            start: 0x500,
            end: 0x2d00,
            kind: MemoryRegionKind::BootloaderGDT,
        },
    );
    // The region where our boot sector was, and where the stacks were for
    // the primary and secondary bootloader stages
    add_memory_region(
        &mut num_memory_regions,
        &mut next_memory_region,
        MemoryRegion {
            start: 0x2d00,
            end: 0x10000,
            kind: MemoryRegionKind::Usable,
        },
    );
    // The place where the bootloader stored the current page tables
    add_memory_region(
        &mut num_memory_regions,
        &mut next_memory_region,
        MemoryRegion {
            start: 0x10000,
            end: 0x20000,
            kind: MemoryRegionKind::BootloaderPageTables,
        },
    );
    // The place the kernel is loaded, lol
    add_memory_region(
        &mut num_memory_regions,
        &mut next_memory_region,
        MemoryRegion {
            start: 0x20000,
            end: 0x70000,
            kind: MemoryRegionKind::Reserved,
        },
    );
    // This is a little tricky, but this is the place where
    // the secondary bootloader stored all of this information
    // which we are going to store in kernel space. The entire idea
    // of doing this translation is that we are moving it from raw memory in this area
    // into variables that the kernel is using. So it should be free memory
    // by the time we are done, which is when this map will be read
    add_memory_region(
        &mut num_memory_regions,
        &mut next_memory_region,
        MemoryRegion {
            start: 0x70000,
            end: 0x7FFFF,
            kind: MemoryRegionKind::Usable,
        },
    );
    // From here on, this should be reported by the hardware. It knows where the
    // EBDA begins and ends, and it knows where all of video memory and everything
    // above 1 MiB is.

    for i in 0..num_reported_memory_regions {
        let raw_region = *next_raw_memory_region;
        next_raw_memory_region = next_raw_memory_region.offset(1);

        add_memory_region(
            &mut num_memory_regions,
            &mut next_memory_region,
            MemoryRegion {
                start: raw_region.start,
                end: raw_region.start + raw_region.size,
                kind: MemoryRegionKind::from(raw_region.kind),
            },
        );
    }

    let memory_regions_slice =
        core::slice::from_raw_parts(memory_regions_start, num_memory_regions);

    let boot_info = BootInfo {
        memory_regions: memory_regions_slice,
        boot_drive,
    };

    crate::interrupts::init_idt();

    crate::main(&boot_info);

    loop {}
}

unsafe fn add_memory_region(
    regions_count: &mut usize,
    next_memory_region_ptr: &mut *mut MemoryRegion,
    next: MemoryRegion,
) {
    **next_memory_region_ptr = next;
    *next_memory_region_ptr = next_memory_region_ptr.offset(1);
    *regions_count += 1;
}

pub struct BootInfo<'a> {
    pub memory_regions: &'a [MemoryRegion],
    pub boot_drive: u32,
}

#[repr(packed)]
#[derive(Copy, Clone)]
struct RawMemoryRegion {
    start: u64,
    size: u64,
    kind: RawMemoryRegionKind,
    acpi: u32,
}

#[repr(u32)]
#[derive(Copy, Clone)]
enum RawMemoryRegionKind {
    SMAPUsable = 1,
    SMAPReserved = 2,
    SMAPACPIReclaimable = 3,
    SMAPACPINVS = 4,
    SMAPBadMemory = 5,
}

impl From<RawMemoryRegionKind> for MemoryRegionKind {
    fn from(raw: RawMemoryRegionKind) -> Self {
        match raw {
            RawMemoryRegionKind::SMAPUsable => MemoryRegionKind::Usable,
            RawMemoryRegionKind::SMAPReserved => MemoryRegionKind::Reserved,
            RawMemoryRegionKind::SMAPACPIReclaimable => MemoryRegionKind::ACPIReclaimable,
            RawMemoryRegionKind::SMAPACPINVS => MemoryRegionKind::ACPINonVolatileStorage,
            RawMemoryRegionKind::SMAPBadMemory => MemoryRegionKind::BadMemory,
        }
    }
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    keprintln!("{}", info);
    loop {}
}
