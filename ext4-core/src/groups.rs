use bin_tools::read_u32_le;

use crate::inode::BlockNumber;


#[derive(Debug, Clone, Copy)]
pub struct GroupDescriptor {
    /// offset 0x00
    block_bitmap: BlockNumber,
    /// offset 0x04
    inode_bitmap: BlockNumber,
    /// offset 0x08
    inode_table: BlockNumber,
    /// offset 0x0c
    free_blocks_count: u32,
    /// offset 0x10
    free_inodes_count: u32,
    /// offset 0x14
    used_dirs_count: u32,
    // padding and reserved bytes
}

impl GroupDescriptor {
    pub fn read(buffer: &[u8]) -> Self {
        Self {
            block_bitmap: BlockNumber::from(read_u32_le(buffer, 0x00)),
            inode_bitmap: BlockNumber::from(read_u32_le(buffer, 0x04)),
            inode_table: BlockNumber::from(read_u32_le(buffer, 0x08)),
            free_blocks_count: read_u32_le(buffer, 0x0c),
            free_inodes_count: read_u32_le(buffer, 0x10),
            used_dirs_count: read_u32_le(buffer, 0x14),
        }
    }

    pub fn block_bitmap_block(&self) -> BlockNumber {
        self.block_bitmap
    }

    pub fn inode_bitmap_block(&self) -> BlockNumber {
        self.inode_bitmap
    }

    pub fn inode_table_block(&self) -> BlockNumber {
        self.inode_table
    }

    pub fn free_blocks(&self) -> u32 {
        self.free_blocks_count
    }

    pub fn free_inodes(&self) -> u32 {
        self.free_inodes_count
    }

    pub fn used_dirs(&self) -> u32 {
        self.used_dirs_count
    }
}
