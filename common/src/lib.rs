#![no_std]

use memory::MemoryRegion;

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
    pub initramfs_address: *const u8,
    pub initramfs_length: u64,
    pub acpi_rsdp_address: *const u8,
    pub physical_memory_offset: u64,
}

pub type KernelEntry = unsafe extern "C" fn(*const BootInfo);
