#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod arch;
mod drivers;
mod entry;
mod gdt;
mod interrupts;
mod memory;
mod std;
use common::PHYS_PAGE_DIRECTORY_POINTER_TABLE_START_ADDR;
use x86_64::{
    structures::paging::{Page, PageTable, Translate},
    VirtAddr,
};

use crate::{entry::BootInfo, memory::active_level_4_table};

fn init() {
    crate::gdt::init();
    crate::interrupts::init_idt();
    unsafe { crate::interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

fn main(boot_info: &BootInfo) {
    kprintln!("Hello!");

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };

    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(boot_info.memory_regions) };

    // map an unused page
    let page = Page::containing_address(VirtAddr::new(0xdeadbeef));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // write the string `New!` to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    kprintln!("Didn't crash.");
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
