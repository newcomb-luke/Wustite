use core::ops::{Index, IndexMut};

use crate::println;

const PAGE_TABLE_SIZE: usize = 0x1000;
pub const PAGE_MAP_LEVEL_4_TABLE_START: *mut u8 = 0x00400000 as *mut u8;
const PAGE_DIRECTORY_POINTER_TABLE_START: *mut u8 = 0x00401000 as *mut u8;
const PAGE_DIRECTORY_TABLE_START: *mut u8 = 0x00402000 as *mut u8;
const PAGE_TABLES_START: *mut u8 = 0x00403000 as *mut u8;
const NUM_PAGE_TABLES: usize = 16;
const NUM_ENTRIES_PER_TABLE: usize = 512;
const PAGE_SIZE: usize = 4096;
pub const PAGE_TABLES_LENGTH: u64 = (PAGE_TABLE_SIZE * (NUM_PAGE_TABLES + 3)) as u64;

#[repr(transparent)]
struct PageTableEntry {
    value: u64,
}

impl PageTableEntry {
    fn set_present(&mut self) {
        self.value |= 1
    }

    fn set_writable(&mut self) {
        self.value |= 2
    }

    fn set_address(&mut self, addr: *mut u8) {
        self.value |= addr as u64
    }
}

#[repr(transparent)]
struct PageTable {
    entries: [PageTableEntry; NUM_ENTRIES_PER_TABLE],
}

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        self.entries.index(index)
    }
}

impl IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.entries.index_mut(index)
    }
}

pub fn identity_map_mem() {
    // Zero the PML4T
    unsafe { PAGE_MAP_LEVEL_4_TABLE_START.write_bytes(0, PAGE_TABLE_SIZE) };
    let pml4t_first = unsafe {
        (PAGE_MAP_LEVEL_4_TABLE_START as *mut PageTableEntry)
            .as_mut()
            .unwrap_unchecked()
    };

    // Zero the PDPT
    unsafe { PAGE_DIRECTORY_POINTER_TABLE_START.write_bytes(0, PAGE_TABLE_SIZE) };
    let pdpt_first = unsafe {
        (PAGE_DIRECTORY_POINTER_TABLE_START as *mut PageTableEntry)
            .as_mut()
            .unwrap_unchecked()
    };

    // Zero the PDT
    unsafe { PAGE_DIRECTORY_TABLE_START.write_bytes(0, PAGE_TABLE_SIZE) };
    let pdt = unsafe {
        (PAGE_DIRECTORY_TABLE_START as *mut PageTable)
            .as_mut()
            .unwrap_unchecked()
    };

    // Zero each of the regular page tables
    unsafe {
        PAGE_TABLES_START.write_bytes(0, PAGE_TABLE_SIZE * NUM_PAGE_TABLES);
    }

    // Set up the first entry of the PML4T, only one entry is all we will need
    // This stores a "pointer" to the page directory pointer table, and sets the entry
    // as present and writable
    pml4t_first.set_present();
    pml4t_first.set_writable();
    pml4t_first.set_address(PAGE_DIRECTORY_POINTER_TABLE_START);

    // Set up the first entry of the PDPT, only one entry is all we need
    // Same value as above for the same reasons
    pdpt_first.set_present();
    pdpt_first.set_writable();
    pdpt_first.set_address(PAGE_DIRECTORY_TABLE_START);

    // Here is where it gets a little fun. We need as many entries as NUM_PAGE_TABLES.
    // This points NUM_PAGE_TABLES page directory table entries to pointers
    // to the corresponding page tables. Each one resides at PAGE_TABLES_START + PAGE_TABLE_SIZE * i.
    // That is to say, they all start at PAGE_TABLES_START, offset by however many previous page tables
    // we have already set.
    for i in 0..NUM_PAGE_TABLES {
        let entry = &mut pdt[i];

        entry.set_present();
        entry.set_writable();
        entry.set_address(unsafe { PAGE_TABLES_START.offset((PAGE_TABLE_SIZE * i) as isize) });

        // Now we need to actually set up our page tables
        // We will start at address 0, and work up by increments of the page size. We
        // use the default size of 4KiB pages.
        let table = unsafe {
            (PAGE_TABLES_START.add(PAGE_TABLE_SIZE * i) as *mut PageTable)
                .as_mut()
                .unwrap_unchecked()
        };

        for entry_idx in 0..NUM_ENTRIES_PER_TABLE {
            let entry = &mut table[entry_idx];
            let addr = ((i * NUM_ENTRIES_PER_TABLE + entry_idx) * PAGE_SIZE) as *mut u8;

            if !(i == 0 && entry_idx == 0) {
                entry.set_present();
            }
            entry.set_writable();
            entry.set_address(addr);
        }
    }

    // Now we are done! Hopefully it works
}
