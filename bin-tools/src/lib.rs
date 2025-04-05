#![no_std]

pub fn read_u64_le(input: &[u8], offset: usize) -> u64 {
    let mut buffer: [u8; 8] = [0; 8];
    buffer.copy_from_slice(&input[offset..offset+8]);
    u64::from_le_bytes(buffer)
}

pub fn read_i64_le(input: &[u8], offset: usize) -> i64 {
    let mut buffer: [u8; 8] = [0; 8];
    buffer.copy_from_slice(&input[offset..offset+8]);
    i64::from_le_bytes(buffer)
}

pub fn read_u32_le(input: &[u8], offset: usize) -> u32 {
    let mut buffer: [u8; 4] = [0; 4];
    buffer.copy_from_slice(&input[offset..offset+4]);
    u32::from_le_bytes(buffer)
}

pub fn read_i32_le(input: &[u8], offset: usize) -> i32 {
    let mut buffer: [u8; 4] = [0; 4];
    buffer.copy_from_slice(&input[offset..offset+4]);
    i32::from_le_bytes(buffer)
}

pub fn read_u16_le(input: &[u8], offset: usize) -> u16 {
    let mut buffer: [u8; 2] = [0; 2];
    buffer.copy_from_slice(&input[offset..offset+2]);
    u16::from_le_bytes(buffer)
}

pub fn read_i16_le(input: &[u8], offset: usize) -> i16 {
    let mut buffer: [u8; 2] = [0; 2];
    buffer.copy_from_slice(&input[offset..offset+2]);
    i16::from_le_bytes(buffer)
}

pub fn read_u32_be(input: &[u8], offset: usize) -> u32 {
    let mut buffer: [u8; 4] = [0; 4];
    buffer.copy_from_slice(&input[offset..offset+4]);
    u32::from_be_bytes(buffer)
}

pub fn read_u16_be(input: &[u8], offset: usize) -> u16 {
    let mut buffer: [u8; 2] = [0; 2];
    buffer.copy_from_slice(&input[offset..offset+2]);
    u16::from_be_bytes(buffer)
}
