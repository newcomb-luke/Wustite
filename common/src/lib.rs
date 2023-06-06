#![no_std]

use memory::MemoryRegion;

pub const MAXIMUM_SUPPORTED_MEMORY: u64 = 0x200000000; // 8 GiB

pub const PHYS_MAP_VIRTUAL_OFFSET: u64 = 0x18000000000;

pub mod elf;
pub mod memory;

pub fn u16_from_slice(bytes: &[u8]) -> u16 {
    let mut u16_bytes = [0u8; 2];
    u16_bytes.copy_from_slice(bytes);
    u16::from_ne_bytes(u16_bytes)
}

pub fn u32_from_slice(bytes: &[u8]) -> u32 {
    let mut u32_bytes = [0u8; 4];
    u32_bytes.copy_from_slice(bytes);
    u32::from_ne_bytes(u32_bytes)
}

pub fn u64_from_slice(bytes: &[u8]) -> u64 {
    let mut u64_bytes = [0u8; 8];
    u64_bytes.copy_from_slice(bytes);
    u64::from_ne_bytes(u64_bytes)
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct BootInfo {
    pub memory_regions_start: *const MemoryRegion,
    pub memory_regions_count: u64,
    pub initramfs_location: *const u8,
    pub initramfs_length: u64,
    pub physical_memory_offset: u64,
}
