#![no_std]

pub const DRIVE_NUM_ADDR: u32 = 0x7c24;
pub const MEMORY_REGIONS_DESCRIPTOR_ADDR: u32 = 0x1000;
pub const MEMORY_REGIONS_START_ADDR: u32 = 0x1010;

pub const PAGE_TABLE_SIZE: usize = 0x1000;
pub const PAGE_SIZE: usize = 0x1000;
pub const NUM_ENTRIES_PER_TABLE: usize = 512;

pub const NUM_INITIAL_PAGE_TABLES: usize = 16;

pub const PAGE_MAP_LEVEL_4_TABLE_START_ADDR: u64 = 0x00400000;
pub const PAGE_MAP_LEVEL_4_TABLE_START: *mut u64 = PAGE_MAP_LEVEL_4_TABLE_START_ADDR as *mut u64;
pub const PAGE_DIRECTORY_POINTER_TABLE_START: *mut u64 = 0x00401000 as *mut u64;
pub const PAGE_DIRECTORY_TABLE_START: *mut u64 = 0x00402000 as *mut u64;
pub const PAGE_TABLES_START_ADDR: u64 = 0x00403000;
pub const PAGE_TABLES_START: *mut u64 = PAGE_TABLES_START_ADDR as *mut u64;
pub const PAGE_TABLES_END_ADDR: u64 =
    PAGE_TABLES_START_ADDR + NUM_INITIAL_PAGE_TABLES as u64 * PAGE_TABLE_SIZE as u64;

pub const PHYS_PAGE_DIRECTORY_POINTER_TABLE_START_ADDR: u64 =
    PAGE_TABLES_END_ADDR + PAGE_TABLE_SIZE as u64;
pub const PHYS_PAGE_DIRECTORY_POINTER_TABLE_START: *mut u64 =
    PHYS_PAGE_DIRECTORY_POINTER_TABLE_START_ADDR as *mut u64;
pub const ALL_PAGE_TABLES_END_ADDR: u64 =
    PHYS_PAGE_DIRECTORY_POINTER_TABLE_START_ADDR + PAGE_TABLE_SIZE as u64;

pub const MAXIMUM_SUPPORTED_MEMORY: u64 = 0x200000000; // 8 GiB

pub const PHYS_MAP_VIRTUAL_OFFSET: u64 = 0x18000000000;
