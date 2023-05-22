use core::ops::{Index, IndexMut};

use common::{
    MAXIMUM_SUPPORTED_MEMORY, NUM_ENTRIES_PER_TABLE, NUM_INITIAL_PAGE_TABLES,
    PAGE_DIRECTORY_POINTER_TABLE_START, PAGE_DIRECTORY_TABLE_START, PAGE_MAP_LEVEL_4_TABLE_START,
    PAGE_SIZE, PAGE_TABLES_START, PAGE_TABLE_SIZE, PHYS_PAGE_DIRECTORY_POINTER_TABLE_START,
};

const PRESENT: u64 = 1;
const WRITABLE: u64 = 2;
const LARGE_PAGE: u64 = 1 << 7;

const PHYS_MEM_PML4T_ENTRY: usize = 3;

#[repr(transparent)]
struct PageTableEntry {
    value: u64,
}

impl PageTableEntry {
    fn set_present(&mut self) {
        self.value |= PRESENT
    }

    fn set_writable(&mut self) {
        self.value |= WRITABLE
    }

    fn set_address(&mut self, addr: *mut u64) {
        self.value |= addr as u64
    }

    fn set_page_size(&mut self) {
        self.value |= LARGE_PAGE
    }
}

#[repr(align(4096))]
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

pub fn identity_map_mem(max_usable_addr: u64) {
    if max_usable_addr > MAXIMUM_SUPPORTED_MEMORY {
        panic!("More memory than bootloader is set up for.");
    }

    // Zero the PML4T
    let (pml4t_first, pml4t_phys) = unsafe {
        PAGE_MAP_LEVEL_4_TABLE_START.write_bytes(0, NUM_ENTRIES_PER_TABLE);

        let pml4t = PAGE_MAP_LEVEL_4_TABLE_START as *mut PageTableEntry;

        (
            pml4t.as_mut().unwrap(),
            pml4t.add(PHYS_MEM_PML4T_ENTRY).as_mut().unwrap(),
        )
    };

    // Zero the PDPT
    let pdpt_first = unsafe {
        PAGE_DIRECTORY_POINTER_TABLE_START.write_bytes(0, NUM_ENTRIES_PER_TABLE);

        (PAGE_DIRECTORY_POINTER_TABLE_START as *mut PageTableEntry)
            .as_mut()
            .unwrap_unchecked()
    };

    let phys_pdpt = unsafe {
        PHYS_PAGE_DIRECTORY_POINTER_TABLE_START.write_bytes(0, NUM_ENTRIES_PER_TABLE);

        (PHYS_PAGE_DIRECTORY_POINTER_TABLE_START as *mut PageTable)
            .as_mut()
            .unwrap_unchecked()
    };

    // Zero the PDT
    let pdt = unsafe {
        PAGE_DIRECTORY_TABLE_START.write_bytes(0, NUM_ENTRIES_PER_TABLE);

        (PAGE_DIRECTORY_TABLE_START as *mut PageTable)
            .as_mut()
            .unwrap_unchecked()
    };

    // Zero each of the regular page tables
    unsafe {
        PAGE_TABLES_START.write_bytes(0, NUM_ENTRIES_PER_TABLE * NUM_INITIAL_PAGE_TABLES);
    }

    // Set up the first entry of the PML4T, only one entry is all we will need
    // This stores a "pointer" to the page directory pointer table, and sets the entry
    // as present and writable
    pml4t_first.set_present();
    pml4t_first.set_writable();
    pml4t_first.set_address(PAGE_DIRECTORY_POINTER_TABLE_START);

    // Set up the 3rd entry of the PML4T, which maps virtual memory from
    // PHYS_MAP_VIRTUAL_OFFSET to all of physical memory by pointing to
    // phys_pdpt
    pml4t_phys.set_present();
    pml4t_phys.set_writable();
    pml4t_phys.set_address(PHYS_PAGE_DIRECTORY_POINTER_TABLE_START);

    // Set up the first entry of the PDPT, only one entry is all we need
    // Same value as above for the same reasons
    pdpt_first.set_present();
    pdpt_first.set_writable();
    pdpt_first.set_address(PAGE_DIRECTORY_TABLE_START);

    // Set up the physical PDPT, this will use HUGE (4 GiB) pages to map
    // virtual memory from PHYS_MAP_VIRTUAL_OFFSET to all of physical memory
    for i in 0..NUM_ENTRIES_PER_TABLE {
        let entry = &mut phys_pdpt[i];

        entry.set_present();
        entry.set_writable();
        entry.set_page_size();
        entry.set_address(core::ptr::null_mut());
    }

    // Here is where it gets a little fun. We need as many entries as NUM_PAGE_TABLES.
    // This points NUM_PAGE_TABLES page directory table entries to pointers
    // to the corresponding page tables. Each one resides at PAGE_TABLES_START + PAGE_TABLE_SIZE * i.
    // That is to say, they all start at PAGE_TABLES_START, offset by however many previous page tables
    // we have already set.
    for i in 0..NUM_INITIAL_PAGE_TABLES {
        let entry = &mut pdt[i];

        entry.set_present();
        entry.set_writable();
        entry.set_address(unsafe { PAGE_TABLES_START.add(PAGE_TABLE_SIZE * i) });

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
            let addr = ((i * NUM_ENTRIES_PER_TABLE + entry_idx) * PAGE_SIZE) as *mut u64;

            if !(i == 0 && entry_idx == 0) {
                entry.set_present();
                entry.set_writable();
                entry.set_address(addr);
            }
        }
    }

    // Now we are done! Hopefully it works
}
