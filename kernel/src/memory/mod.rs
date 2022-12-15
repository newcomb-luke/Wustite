use core::slice;

use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        page_table::PageTableEntry, FrameAllocator, Mapper, OffsetPageTable, Page, PageTable,
        PageTableFlags, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

use crate::entry::memory::MemoryRegion;

// 0x00000500 - 0x00002d00 - Secondary bootloader load location (assuming size of 10 KiB)
// 0x00002d00 - 0x00007BFF - Used for bootloader (stage 1 + stage 2) stack
// 0x00007C00 - 0x00007DFF - 512 bytes - OS Boot Sector
// 0x00007E00 - 0x00008000 - 512 bytes - FAT12 Driver Boot Record Buffer
// 0x00008000 - 0x00009200 - 5 KiB - FAT12 Driver Directory Entry Buffer
// 0x00009200 - 0x0000A600 - 5 KiB - FAT12 Driver File Allocation Table Buffer
// 0x0000A600 - 0x0000F600 - 20 KiB - FAT12 Driver File Read Buffer
// 0x0000F600 - 0x00010000 - Empty space
// 0x00010000 - 0x00020000 - Bootloader initialized page table area
// 0x00020000 - 0x00070000 - Kernel read location (maximum size of 320 KiB)
// 0x00070000 - 0x0007FFFF - Stage2->Kernel Data Area
// 0x00080000 - 0x0009FFFF - 128 KiB - Extended BIOS Data Area
// 0x000A0000 - 0x000BFFFF - 128 KiB - Video Display Memory
// 0x000C0000 - 0x000C7FFF - 32 KiB - Video BIOS
// 0x000C8000 - 0x000EFFFF - 160 KiB - BIOS Expansions
// 0x000F0000 - 0x000FFFFF - 64 KiB - Motherboard BIOS
// 0x00100000 - 0x00200000 - Kernel Location
// 0x00200000 - 0x00300000 - Kernel Stack Region
// 0x00300000 - 0x00301000 - Kernel PML4T
// 0x00301000 - 0x00302000 - Kernel Page Directory Pointer Table
// 0x00302000 - 0x00303000 - Kernel Page Directory Table
// 0x00303000 - 0x0030b000 - Kernel Page Tables
// 0x0030b000 - 0x0030c000 - Unified memory map location
// 0x0030c000 - 0x00EFFFFF - 12 MiB - RAM free for use
// 0x00F00000 - 0x00FFFFFF - 1 MiB - Possibly memory-mapped hardware

const PAGE_TABLE_NUM_ENTRIES: usize = 512;
const PAGE_TABLE_BYTES_PER_ENTRY: usize = 8;
const BYTES_PER_PAGE_TABLE: usize = PAGE_TABLE_NUM_ENTRIES * PAGE_TABLE_BYTES_PER_ENTRY;

/// A FrameAllocator that always returns `None`.
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}

pub struct UnifiedMemoryMap {
    pub regions: &'static [UnifiedMemoryRegion],
}

impl UnifiedMemoryMap {
    pub unsafe fn new(memory_map: &mut [MemoryRegion]) -> Self {
        const BEGIN: *mut UnifiedMemoryRegion = 0x0030b000 as *mut UnifiedMemoryRegion;
        const MAX_NUM: usize = 512;

        let mut num_regions = 0;

        let mut current_memory_region = BEGIN;

        Self {
            regions: unsafe { slice::from_raw_parts(BEGIN, num_regions) },
        }
    }
}

pub struct UnifiedMemoryRegion {
    pub start: usize,
    pub end: usize,
}

/// Initialize a new OffsetPageTable.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    init_kernel_page_tables();

    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

unsafe fn init_kernel_page_tables() {
    const KERNEL_PML4T: *mut u8 = 0x00300000 as *mut u8;
    const KERNEL_PDPT: *mut u8 = 0x00301000 as *mut u8;
    const KERNEL_PDT: *mut u8 = 0x00302000 as *mut u8;
    const KERNEL_PAGE_TABLES_START: *mut u8 = 0x00303000 as *mut u8;
    const INITIAL_NUM_REGULAR_PAGE_TABLES: usize = 8;

    zero_page_table_memory(KERNEL_PML4T);
    zero_page_table_memory(KERNEL_PDPT);
    zero_page_table_memory(KERNEL_PDT);

    for i in 0..INITIAL_NUM_REGULAR_PAGE_TABLES {
        let current_page_table_ptr = KERNEL_PAGE_TABLES_START.offset(
            (i * PAGE_TABLE_NUM_ENTRIES * PAGE_TABLE_BYTES_PER_ENTRY)
                .try_into()
                .unwrap(),
        );
        zero_page_table_memory(current_page_table_ptr);
    }

    let pml4t_ptr: &mut PageTable = unsafe { &mut *(KERNEL_PML4T as *mut PageTable) };
    let pdpt_ptr: &mut PageTable = unsafe { &mut *(KERNEL_PDPT as *mut PageTable) };
    let pdt_ptr: &mut PageTable = unsafe { &mut *(KERNEL_PDT as *mut PageTable) };

    pml4t_ptr[0] = PageTableEntry::new();
    pml4t_ptr[0].set_addr(
        PhysAddr::new(KERNEL_PDPT as u64),
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
    );

    pdpt_ptr[0] = PageTableEntry::new();
    pdpt_ptr[0].set_addr(
        PhysAddr::new(KERNEL_PDT as u64),
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
    );

    let mut current_phys_addr = 0;
    let mut current_page_table_addr = KERNEL_PAGE_TABLES_START as u64;

    for i in 0..INITIAL_NUM_REGULAR_PAGE_TABLES {
        let current_page_table_ptr = &mut *((current_page_table_addr as *mut u8) as *mut PageTable);

        pdt_ptr[i] = PageTableEntry::new();
        pdt_ptr[i].set_addr(
            PhysAddr::new(current_page_table_addr),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        );

        for entry_idx in 0..PAGE_TABLE_NUM_ENTRIES {
            // Skip the very very first page of memory, that way dereferencing
            // a null pointer results in a page fault
            if i == 0 && entry_idx == 0 {
                current_page_table_ptr[entry_idx] = PageTableEntry::new();
                current_page_table_ptr[entry_idx]
                    .set_addr(PhysAddr::new(current_phys_addr), PageTableFlags::empty());

                current_phys_addr += 0x1000;

                continue;
            }

            current_page_table_ptr[entry_idx] = PageTableEntry::new();
            current_page_table_ptr[entry_idx].set_addr(
                PhysAddr::new(current_phys_addr),
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            );

            current_phys_addr += 0x1000;
        }

        current_page_table_addr += 0x1000;
    }

    let (_, flags) = Cr3::read();

    Cr3::write(
        PhysFrame::from_start_address(PhysAddr::new(KERNEL_PML4T as u64)).unwrap(),
        flags,
    );
}

unsafe fn zero_page_table_memory(page_table: *mut u8) {
    for i in 0..(BYTES_PER_PAGE_TABLE as isize) {
        unsafe {
            page_table.offset(i).write_volatile(0);
        }
    }
}

/// Creates an example mapping for the given page to frame `0xb8000`.
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        // FIXME: this is not safe, we do it only for testing
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}
