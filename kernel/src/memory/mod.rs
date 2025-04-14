use core::alloc::Layout;

use common::{BootInfo, memory::MemoryRegion};
use spin::Mutex;
use x86_64::{
    PhysAddr, VirtAddr,
    registers::control::Cr3,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageSize, PageTable, PageTableFlags,
        PhysFrame, Size4KiB, Translate,
    },
};

use crate::{allocator::ALLOCATOR, logln};

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    memory_map: &'static [MemoryRegion],
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames contained in it
    /// are really unused
    pub unsafe fn init(memory_map: &'static [MemoryRegion]) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // get usable regions from memory map
        let regions = self.memory_map.iter();
        // map each region to its address range
        let addr_ranges = regions.map(|r| r.start_addr..r.end_addr);
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

static mut PHYS_MEM_OFFSET: u64 = 0;

pub fn initialize_memory(boot_info: &BootInfo) {
    logln!("[info] Starting memory initialization");

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { init(phys_mem_offset) };

    let memory_regions = unsafe {
        core::slice::from_raw_parts(
            boot_info.memory_regions_start,
            boot_info.memory_regions_count as usize,
        )
    };

    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(memory_regions) };

    crate::allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Kernel heap initialization failed");

    MEMORY_MAPPER.init(mapper, frame_allocator);

    logln!("[info] Memory initialized");
}

/// Initialize a new OffsetPageTable.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    unsafe {
        PHYS_MEM_OFFSET = physical_memory_offset.as_u64();

        let level_4_table = active_level_4_table(physical_memory_offset);
        OffsetPageTable::new(level_4_table, physical_memory_offset)
    }
}

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}

pub struct KernelMapper {
    inner: Mutex<Option<InnerKernelMapper>>,
}

const PURE_VIRTUAL_START: usize = 0x1000_0000_0000;

struct InnerKernelMapper {
    virtual_offset: usize,
    mapper: OffsetPageTable<'static>,
    frame_allocator: BootInfoFrameAllocator,
}

impl KernelMapper {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    pub fn init(&self, mapper: OffsetPageTable<'static>, frame_allocator: BootInfoFrameAllocator) {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut inner = self.inner.lock();
            *inner = Some(InnerKernelMapper::new(mapper, frame_allocator));
        });
    }

    pub unsafe fn identity_map(&self, frame: PhysFrame, flags: PageTableFlags) -> Result<(), ()> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            if let Some(inner) = self.inner.lock().as_mut() {
                unsafe {
                    inner
                        .mapper
                        .identity_map(frame, flags, &mut inner.frame_allocator)
                        .map_err(|_| ())?
                        .flush();
                }

                Ok(())
            } else {
                Err(())
            }
        })
    }

    /// Maps a virtual page which has a mapping to the specified physical address
    pub unsafe fn map_virt_page(
        &self,
        address: PhysAddr,
        flags: PageTableFlags,
    ) -> Result<VirtAddr, ()> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            if let Some(inner) = self.inner.lock().as_mut() {
                let virt_address = VirtAddr::new(inner.next_virtual_page() as u64);

                unsafe {
                    inner
                        .mapper
                        .map_to(
                            Page::<Size4KiB>::containing_address(virt_address),
                            PhysFrame::containing_address(address),
                            flags,
                            &mut inner.frame_allocator,
                        )
                        .map_err(|_| ())?
                        .flush();
                }

                Ok(virt_address)
            } else {
                Err(())
            }
        })
    }

    pub unsafe fn virt_to_phys(&self, address: VirtAddr) -> Result<PhysAddr, ()> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            if let Some(inner) = self.inner.lock().as_mut() {
                inner.mapper.translate_addr(address).ok_or(())
            } else {
                Err(())
            }
        })
    }

    pub unsafe fn phys_to_virt(&self, address: PhysAddr) -> Result<VirtAddr, ()> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            if let Some(inner) = self.inner.lock().as_mut() {
                Ok(inner.mapper.phys_offset() + address.as_u64())
            } else {
                Err(())
            }
        })
    }
}

impl InnerKernelMapper {
    pub fn new(mapper: OffsetPageTable<'static>, frame_allocator: BootInfoFrameAllocator) -> Self {
        Self {
            virtual_offset: PURE_VIRTUAL_START,
            mapper,
            frame_allocator,
        }
    }

    fn next_virtual_page(&mut self) -> usize {
        let offset = self.virtual_offset;

        self.virtual_offset += 4096;

        offset
    }
}

pub static MEMORY_MAPPER: KernelMapper = KernelMapper::new();
